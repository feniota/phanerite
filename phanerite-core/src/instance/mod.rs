use crate::auth::Authentication;
use crate::download::Downloader;
use crate::download::task::{DownloadTask, filter_existed, filter_hash};
use crate::download::vanilla::assets::AssetIndexList;
use crate::error::{Error, Result};
use crate::instance::manifest::InstanceManifest;
use crate::runtime::java::JavaRuntime;
use crate::storage::Storage;
use crate::storage::temp::TempGuard;
use futures::{AsyncReadExt, AsyncWriteExt};
use futures::{Stream, StreamExt};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

pub mod arguments;
pub mod manifest;
pub mod overlay;
pub mod variables;

#[derive(Clone)]
pub struct NotReady;
#[derive(Clone)]
pub struct Ready;

pub struct Instance<'storage, R: Clone, C: Clone> {
    pub instance_dir: PathBuf,
    pub manifest: InstanceManifest,

    pub storage: &'storage Storage,

    /// Runtime 的准备状态
    /// JavaRuntime 或 NotReady
    pub runtime: R,
    /// 游戏完整性状态
    /// Ready 或 NotReady
    pub completeness: C,
}

impl<'storage> Instance<'storage, NotReady, NotReady> {
    /// 创建实例
    pub async fn create(
        manifest: impl Into<InstanceManifest>,
        name: Option<impl AsRef<str>>,
        storage: &'storage Storage,
        downloader: &'storage impl Downloader,
    ) -> Result<Self> {
        let mut manifest = manifest.into();
        if let Some(name) = name {
            manifest.rename(name.as_ref())
        }

        // 准备实例目录
        let path = storage.versions_dir().join(&manifest.id);
        if path.exists() {
            return Err(Error::other("Instance exists"));
        }
        async_fs::create_dir_all(&path).await?;

        // versions/{name}/{name}.json
        let info_file = path.join(format!("{}.json", manifest.id));
        let mut info_file = async_fs::File::create(info_file).await?;
        let info_json = serde_json::to_vec_pretty(&manifest)?;
        info_file.write_all(&info_json).await?;

        // assets/indexes/{id}.json
        let index_file = storage
            .assets_indexes()
            .join(format!("{}.json", manifest.asset_index.id));
        let mut index_file = async_fs::File::create(index_file).await?;
        let assets_index = manifest.asset_index.get_list(downloader).await?;
        let index_json = serde_json::to_vec_pretty(&assets_index)?;
        index_file.write_all(&index_json).await?;

        Self::open(&manifest.id, storage).await
    }
    /// 打开本地实例
    pub async fn open(name: impl AsRef<str>, storage: &'storage Storage) -> Result<Self> {
        let instance_dir = storage.versions_dir().join(name.as_ref());
        let json = find_manifest(name.as_ref(), &instance_dir).await?;
        Ok(Self {
            instance_dir,
            manifest: serde_json::from_slice::<InstanceManifest>(&json)?,
            storage,
            runtime: NotReady,
            completeness: NotReady,
        })
    }
    /// 扫描实例
    pub fn scan(storage: &'storage Storage) -> impl Stream<Item = Result<Self>> + 'storage {
        futures::stream::try_unfold((storage, None), async |(storage, dir)| {
            let mut dir = match dir {
                Some(dir) => dir,
                None => async_fs::read_dir(storage.versions_dir()).await?,
            };
            match dir.next().await {
                Some(entry) => {
                    let entry = entry?;
                    let value = Self::open(entry.file_name().to_string_lossy(), storage).await?;
                    Ok(Some((value, (storage, Some(dir)))))
                }
                None => Ok(None),
            }
        })
    }
}

impl<R: Clone, C: Clone> Instance<'_, R, C> {
    /// 获取当前实例的游戏文件路径
    pub fn client_file(&self) -> PathBuf {
        self.instance_dir.join(format!("{}.jar", self.manifest.jar))
    }
    /// 重命名
    pub async fn rename(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        let path = self.storage.versions_dir().join(&name);
        if path.exists() {
            return Err(Error::other("Instance exists"));
        }
        async_fs::rename(&self.instance_dir, &path).await?;
        self.instance_dir = path;
        self.manifest.rename(name);
        self.save().await?;
        Ok(())
    }
    /// 复制
    pub async fn copy(&self, name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let path = self.storage.versions_dir().join(&name);
        if path.exists() {
            return Err(Error::other("Instance exists"));
        }
        async_fs::copy(&self.instance_dir, &path).await?;
        let mut manifest = self.manifest.clone();
        manifest.rename(name);
        let new = Instance {
            instance_dir: path,
            manifest,
            storage: self.storage,
            runtime: self.runtime.clone(),
            completeness: self.completeness.clone(),
        };
        Ok(new)
    }
    /// 删除
    pub async fn delete(self) -> Result<()> {
        async_fs::remove_dir_all(self.instance_dir).await?;
        self.storage.clean_hardlink().await?;
        Ok(())
    }
    /// 持久化版本清单
    pub async fn save(&self) -> Result<()> {
        let file = self.instance_dir.join(format!("{}.json", self.manifest.id));
        let mut file = async_fs::File::create(file).await?;
        let json = serde_json::to_vec_pretty(&self.manifest)?;
        file.write_all(&json).await?;
        Ok(())
    }
    /// 修复 Assets 索引（如果打开失败）
    pub async fn fix_assets_index(&self, downloader: &impl Downloader) -> Result<AssetIndexList> {
        let index_path = self
            .storage
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
                let assets_index = self.manifest.asset_index.get_list(downloader).await?;
                let index_json = serde_json::to_vec_pretty(&assets_index)?;
                index_file.write_all(&index_json).await?;
                Ok(open().await?)
            }
        }
    }
    /// 粗略检查游戏完整性，返回缺失文件
    /// 检查 Assets 索引，不检查压缩包，不校验 Hash
    pub async fn check_exist(
        &self,
        features: HashSet<&'static str>,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        let tasks = self.install(features).await?;
        let tasks = filter_existed(tasks, false);
        Ok(tasks)
    }
    /// 检查游戏完整性，返回缺失文件
    /// 检查 Assets 索引，重下压缩包，校验 Hash
    pub async fn check_full(
        &self,
        features: HashSet<&'static str>,
        downloader: &impl Downloader,
    ) -> Result<Vec<DownloadTask>> {
        self.fix_assets_index(downloader).await?;
        let tasks = futures::stream::iter(self.install(features).await?);
        let tasks = filter_hash(tasks, true).collect().await;
        Ok(tasks)
    }

    /// 构建下载任务，并减少下载量
    pub async fn install_less(
        &self,
        features: HashSet<&'static str>,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        let full = self.install(features).await?;
        Ok(filter_existed(full, true))
    }
    /// 构建完整下载任务
    pub async fn install(
        &self,
        features: HashSet<&'static str>,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        // Assets
        let assets_index = self
            .storage
            .assets_indexes()
            .join(format!("{}.json", self.manifest.assets));
        let mut assets_manifest = Vec::new();
        async_fs::File::open(assets_index)
            .await?
            .read_to_end(&mut assets_manifest)
            .await?;
        let assets_manifest = serde_json::from_slice::<AssetIndexList>(&assets_manifest)?;
        let assets_task = assets_manifest.build_assets_task(self.storage);

        // Libraries
        let native_dir = self.instance_dir.join("native");
        let libraries_task = self
            .manifest
            .libraries
            .iter()
            .scan((features, native_dir), |(f, n), x| {
                Some([x.to_task(self.storage, f), x.to_native_task(f, n)])
            })
            .flatten()
            .flatten();

        // Extra
        let extra = self.extra_downloads(self.storage).await?;

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

impl<'a, R: Clone> Instance<'a, R, NotReady> {
    /// 确定已经完成了完整性检查
    pub fn ensure_ready(self) -> Instance<'a, R, Ready> {
        Instance {
            instance_dir: self.instance_dir,
            manifest: self.manifest,
            storage: self.storage,
            runtime: self.runtime,
            completeness: Ready,
        }
    }
}

impl<'a, R: Clone, C: Clone> Instance<'a, R, C> {
    pub async fn bind_java(self, java: JavaRuntime) -> Result<Instance<'a, JavaRuntime, C>> {
        Ok(Instance {
            instance_dir: self.instance_dir,
            manifest: self.manifest,
            storage: self.storage,
            runtime: java,
            completeness: self.completeness,
        })
    }
}

impl Instance<'_, JavaRuntime, Ready> {
    pub async fn launch(&self, auth: &impl Authentication) -> Result<LaunchCommand<'_>> {
        let arg_path = self.storage.temp_file().await?;
        let mut arg_file = async_fs::File::create(&arg_path).await?;
        arg_file
            .write_all(auth.args(self).await?.to_string().as_bytes())
            .await?;
        arg_file.flush().await?;
        drop(arg_file);

        let mut cmd = async_process::Command::new(self.runtime.clone());
        cmd.arg(format!(
            "@{}",
            std::path::absolute(&arg_path)?.to_string_lossy()
        ))
        .current_dir(&self.instance_dir);
        Ok(LaunchCommand {
            cmd,
            _arg_temp: arg_path,
        })
    }
}

/// `async_process::Command`的包装，用于保证参数文件不会被提前删除
pub struct LaunchCommand<'a> {
    cmd: async_process::Command,
    _arg_temp: TempGuard<'a>,
}

impl Deref for LaunchCommand<'_> {
    type Target = async_process::Command;

    fn deref(&self) -> &Self::Target {
        &self.cmd
    }
}

impl DerefMut for LaunchCommand<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cmd
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
