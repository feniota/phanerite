use crate::download::downloader::Downloader;
use crate::download::task::{DownloadTask, filter_existed, filter_hash};
use crate::download::vanilla::assets::AssetIndexList;
use crate::error::{Error, Result};
use crate::instance::manifest::InstanceManifest;
use crate::storage::Storage;
use futures::StreamExt;
use futures::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashSet;
use std::path::PathBuf;

pub mod arguments;
pub mod manifest;
pub mod overlay;
pub mod simple_info;
pub mod variables;

pub struct Instance {
    pub instance_dir: PathBuf,
    pub manifest: InstanceManifest,
}

impl Instance {
    /// 获取当前实例的游戏文件路径
    pub fn client_file(&self) -> PathBuf {
        self.instance_dir.join(format!("{}.jar", self.manifest.jar))
    }
    /// 创建实例
    pub async fn create<'a>(
        manifest: impl Into<InstanceManifest>,
        name: impl AsRef<str>,
        storage: &'a Storage,
        downloader: &'a Downloader,
    ) -> Result<Self> {
        let manifest = manifest.into().rename(name.as_ref());

        // 准备实例目录
        let path = storage.versions_dir().join(name.as_ref());
        if path.exists() {
            return Err(Error::other("Instance exists"));
        }
        async_fs::create_dir_all(&path).await?;

        // versions/{name}/{name}.json
        let info_file = path.join(format!("{}.json", name.as_ref()));
        let mut info_file = async_fs::File::create(info_file).await?;
        let info_json = serde_json::to_vec_pretty(&manifest)?;
        info_file.write_all(&info_json).await?;

        // assets/indexes/{id}.json
        let index_file = storage
            .assets_indexes()
            .join(format!("{}.json", manifest.asset_index.id));
        let mut index_file = async_fs::File::create(index_file).await?;
        let assets_index = AssetIndexList::get(&manifest.asset_index, downloader).await?;
        let index_json = serde_json::to_vec_pretty(&assets_index)?;
        index_file.write_all(&index_json).await?;

        Self::open(name, storage).await
    }
    /// 打开本地实例
    pub async fn open(name: impl AsRef<str>, storage: &Storage) -> Result<Self> {
        let instance_dir = std::path::absolute(storage.versions_dir().join(name.as_ref()))?;
        let json = find_manifest(name.as_ref(), &instance_dir).await?;
        Ok(Self {
            instance_dir,
            manifest: serde_json::from_slice::<InstanceManifest>(&json)?,
        })
    }
    /// 粗略检查游戏完整性，返回缺失文件
    /// 检查 Assets 索引，不检查压缩包，不校验 Hash
    pub async fn check_exist(
        &self,
        storage: &Storage,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        let tasks = self.install(HashSet::new(), storage).await?;
        let tasks = filter_existed(tasks, false);
        Ok(tasks)
    }
    /// 检查游戏完整性，返回缺失文件
    /// 检查 Assets 索引，重下压缩包，校验 Hash
    pub async fn check_full(
        &self,
        features: HashSet<&'static str>,
        storage: &Storage,
        downloader: &Downloader,
    ) -> Result<Vec<DownloadTask>> {
        self.fix_assets_index(storage, downloader).await?;
        let tasks = futures::stream::iter(self.install(features, storage).await?);
        let tasks = filter_hash(tasks, true).collect().await;
        Ok(tasks)
    }
    /// 修复 Assets 索引（如果打开失败）
    pub async fn fix_assets_index(
        &self,
        storage: &Storage,
        downloader: &Downloader,
    ) -> Result<AssetIndexList> {
        let index_path = storage
            .assets_indexes()
            .join(format!("{}.json", self.manifest.assets));
        let open = async || {
            let mut assets_manifest = Vec::new();
            async_fs::File::open(&index_path)
                .await?
                .read_to_end(&mut assets_manifest)
                .await?;
            let assets_manifest = serde_json::from_slice::<AssetIndexList>(&assets_manifest)?;
            Ok::<AssetIndexList, Error>(assets_manifest)
        };
        match open().await {
            Ok(v) => Ok(v),
            Err(_) => {
                let _ = async_fs::remove_file(&index_path).await;
                let mut index_file = async_fs::File::create(&index_path).await?;
                let assets_index =
                    AssetIndexList::get(&self.manifest.asset_index, downloader).await?;
                let index_json = serde_json::to_vec_pretty(&assets_index)?;
                index_file.write_all(&index_json).await?;
                Ok(open().await?)
            }
        }
    }
    /// 构建下载任务，并减少下载量
    pub async fn install_less(
        &self,
        features: HashSet<&'static str>,
        storage: &Storage,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        let full = self.install(features, storage).await?;
        Ok(filter_existed(full, true))
    }
    /// 构建完整下载任务
    pub async fn install(
        &self,
        features: HashSet<&'static str>,
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
        let native_dir = self.instance_dir.join("native");
        let libraries_task = self
            .manifest
            .libraries
            .iter()
            .scan((features, native_dir), |(f, n), x| {
                Some([x.to_task(storage, f), x.to_native_task(f, n)])
            })
            .flatten()
            .flatten();

        // Extra
        let extra = self.extra_downloads(storage).await?;

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
            .chain(libraries_task)
            .chain(extra))
    }
}

/// 寻找并打开实例清单 JSON
async fn find_manifest(name: impl AsRef<str>, instance_dir: &PathBuf) -> Result<Vec<u8>> {
    // 优先考虑 versions/{name}/{name}.json
    let file = instance_dir.join(format!("{}.json", name.as_ref()));
    if file.is_file() {
        let mut json = Vec::new();
        async_fs::File::open(file)
            .await?
            .read_to_end(&mut json)
            .await?;
        return Ok(json);
    }

    // 寻找别的 json
    let jsons = async_fs::read_dir(&instance_dir)
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
        return Ok(json);
    }

    Err(Error::other("No instance found"))
}
