use crate::download::downloader::Downloader;
use crate::download::task::{DownloadTask, filter_existed};
use crate::download::vanilla::assets::AssetIndexList;
use crate::download::vanilla::version_info::VersionManifest;
use crate::error::{Error, Result};
use crate::instance::instance_info::InstanceManifest;
use crate::storage::Storage;
use futures::StreamExt;
use futures::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::error;

pub mod arguments;
pub mod instance_info;
pub mod variables;

pub struct Instance {
    pub instance_dir: PathBuf,
    pub manifest: InstanceManifest,
}

impl Instance {
    pub fn client_file(&self) -> PathBuf {
        self.instance_dir.join(format!("{}.jar", self.manifest.jar))
    }
    pub async fn create(
        version: VersionManifest,
        name: &str,
        storage: &Storage,
        downloader: &Downloader,
    ) -> Result<()> {
        // 准备实例目录
        let path = storage.versions_dir().join(name);
        if path.exists() {
            return Err(Error::other("Instance exists"));
        }
        async_fs::create_dir_all(&path).await?;

        // 创建需要的文件
        let info_file = path.join(format!("{}.json", name));
        let mut info_file = async_fs::File::create(info_file).await?;
        let index_file = storage
            .assets_indexes()
            .join(format!("{}.json", version.asset_index.id));
        let mut index_file = async_fs::File::create(index_file).await?;

        // versions/{name}/{name}.json
        let manifest = InstanceManifest::from_remote(version.clone()).rename(name.to_string());
        let info_json = serde_json::to_vec_pretty(&manifest)?;
        info_file.write_all(&info_json).await?;
        drop(manifest);
        drop(info_json);

        // 构造下载任务
        let native_dir = path.join("native");
        let features = HashSet::new(); // 下载大概不需要启用 features
        let game_file = path.join(format!("{}.jar", name));
        let (downloads, assets_index) = version
            .build_all_task(game_file, &native_dir, &features, storage, downloader)
            .await?;
        let downloads = filter_existed(downloads);

        // assets/indexes/{id}.json
        let index_json = serde_json::to_vec_pretty(&assets_index)?;
        index_file.write_all(&index_json).await?;
        drop(index_json);

        // 执行下载
        let errors = downloader.download_concurrent(downloads).await;

        if errors.is_empty() {
            Ok(())
        } else {
            errors.iter().for_each(|e| error!("{e}"));
            Err(Error::other("download errors"))
        }
    }
    pub async fn open(instance_dir: impl AsRef<Path>) -> Result<Self> {
        let path = std::path::absolute(instance_dir)?;

        // 优先考虑 versions/{name}/{name}.json
        if let Some(parent) = path.parent().and_then(|t| t.file_name()) {
            let file = path.join(format!("{}.json", parent.to_string_lossy()));
            if file.is_file() {
                let mut json = Vec::new();
                async_fs::File::open(file)
                    .await?
                    .read_to_end(&mut json)
                    .await?;
                return Ok(Self {
                    instance_dir: path,
                    manifest: serde_json::from_slice(&json)?,
                });
            }
        }

        let jsons = async_fs::read_dir(&path)
            .await?
            .filter_map(|entry| async move {
                let entry = entry.ok()?;
                let name = entry.file_name();
                if name.to_string_lossy().ends_with(".json") && entry.path().is_file() {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .await;

        if jsons.len() == 1 {
            let mut json = Vec::new();
            async_fs::File::open(jsons.first().unwrap())
                .await?
                .read_to_end(&mut json)
                .await?;
            return Ok(Self {
                instance_dir: path,
                manifest: serde_json::from_slice(&json)?,
            });
        }

        Err(Error::other("No instance found"))
    }
    pub async fn check_exist(&self, storage: &Storage) -> Result<Vec<DownloadTask>> {
        let features = HashSet::new();
        let tasks = self.build_all_task(&features, storage).await?;
        let tasks = filter_existed(tasks);

        Ok(tasks.collect())
    }
    // TODO: Inelegant implementation, plan rewrite
    pub async fn check_hash(
        &self,
        storage: &Storage,
        downloader: &Downloader,
    ) -> Result<Vec<DownloadTask>> {
        let features = HashSet::new();
        let tasks = self.build_all_task(&features, storage).await?;
        let res = async_lock::Mutex::new(Vec::new());
        for task in tasks {
            if downloader.hash_file(&task).await.is_err() {
                res.lock().await.push(task)
            }
        }
        Ok(res.into_inner())
    }
    async fn build_all_task(
        &self,
        features: &HashSet<&'static str>,
        storage: &Storage,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        // Assets
        let assets_index = storage
            .assets_indexes()
            .join(format!("{}.json", self.manifest.assets));
        let mut assets_manifest = Vec::new();
        async_fs::File::open(assets_index)
            .await?
            .read_to_end(&mut assets_manifest)
            .await?;
        let assets_manifest = serde_json::from_slice::<AssetIndexList>(&assets_manifest)?;
        let assets_task = assets_manifest.build_assets_task(storage);

        // Libraries
        let libraries_task = self
            .manifest
            .libraries
            .iter()
            .flat_map(|x| {
                [
                    x.to_task(storage, features),
                    x.to_native_task(features, &self.instance_dir.join("native")),
                ]
            })
            .flatten();

        // Client
        let file_name = format!("{}.jar", self.manifest.id);
        let client_task = self.manifest.downloads.client.as_ref().map(|c| {
            DownloadTask::builder()
                .url(c.url.clone())
                .to_path(self.instance_dir.join(&file_name))
                .share()
                .file_name(file_name)
                .file_size(c.size)
                .hash(c.sha1.clone())
                .build()
        });

        Ok(client_task
            .into_iter()
            .chain(assets_task)
            .chain(libraries_task))
    }
}
