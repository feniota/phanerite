// 游戏实例
//
// 一个实例就是 `{root}/versions/{id}/` 加上其中的版本清单
// [`manifest::InstanceManifest`]。[`Instance`] 只描述实例、生成任务，
// 不替调用方执行下载，也不替调用方开进程。
//
// # 用类型记录准备状态
//
// `Instance<'_, R, C>` 的两个参数分别记录「Java 绑好了没」和「文件齐不
// 齐」：
//
// - `R`：[`NotReady`] → 绑定后变成 [`JavaRuntime`]
// - `C`：[`NotReady`] → [`Ready`]
//
// [`Instance::launch`] 只对 `Instance<'_, JavaRuntime, Ready>` 存在，
// [`Instance::install_loader`] 只要求 `R = JavaRuntime`。于是「没装 Java
// 就启动」「没检查完整性就启动」这两类错误在编译期就没了。
//
// 注意 [`Instance::ensure_ready`] 是一句断言而不是一次检查：它只把 `C`
// 换成 `Ready`，实际的完整性要调用方先用 [`Instance::check_exist`] 或
// [`Instance::check_full`] 自己确认。
//
// # 安装是「生成任务」而不是「执行下载」
//
// [`Instance::install`] 返回的是 [`DownloadTask`] 的迭代器；交给哪个
// [`Downloader`] 执行、并发多少、要不要监视进度，全部由调用方决定。
// [`Instance::install_less`] 是跳过已存在文件的版本。
//
// 由模组加载器带来的、无法从标准清单推导出来的下载任务，走
// [`LoaderInstall::extra_downloads`](crate::mod_loader::LoaderInstall::extra_downloads)。
//
// # 清单与 patch
//
// 模组加载器不重写版本清单，而是往
// [`InstanceManifest::patches`](manifest::InstanceManifest::patches) 里追加
// 一层 [`overlay::Patch`]，再由
// [`InstanceManifest::resolve`](manifest::InstanceManifest::resolve) 重放
// 全部 patch。累积型字段（`libraries`、`arguments`）在重放前会先清空，
// 所以 `resolve()` 可以反复调用而不会重复累加。
//
// # 启动参数
//
// [`variables::Variables`] 收集 `${...}` 变量并按 feature 过滤条件参数，
// 展开成 [`arguments::LaunchArguments`]。参数不直接放进命令行，而是写进
// 一个临时文件再用 `@file` 传给 java——绕开 Windows 的命令行长度上限，
// 同时也让访问令牌不出现在进程的命令行里。[`LaunchCommand`] 持有这个
// 临时文件的 guard，因此它必须活到进程启动之后。
//! Game instances
//!
//! An instance is `{root}/versions/{id}/` together with the version manifest
//! it contains, [`InstanceManifest`]. [`Instance`] only describes
//! the instance and produces tasks; it neither runs the downloads nor spawns
//! the process on the caller's behalf.
//!
//! # Readiness tracked in the type
//!
//! The two parameters of `Instance<'_, R, C>` record "is a Java bound" and
//! "are the files complete":
//!
//! - `R`: [`NotReady`] → becomes [`JavaRuntime`] once bound
//! - `C`: [`NotReady`] → [`Ready`]
//!
//! [`Instance::launch`] exists only for `Instance<'_, JavaRuntime, Ready>`,
//! and [`Instance::install_loader`] only requires `R = JavaRuntime`. Two
//! classes of mistake — launching without a Java, and launching without
//! having checked integrity — therefore cannot be written at all.
//!
//! Note that [`Instance::ensure_ready`] is an assertion rather than a check:
//! it only swaps `C` for `Ready`, and the caller is expected to have
//! established the actual integrity beforehand with [`Instance::check_exist`]
//! or [`Instance::check_full`].
//!
//! # Installing produces tasks, it does not download
//!
//! [`Instance::install`] returns an iterator of [`DownloadTask`]; which
//! [`Downloader`] runs them, at what concurrency, and whether progress is
//! monitored at all, is entirely up to the caller.
//! [`Instance::install_less`] is the variant that skips files that already
//! exist.
//!
//! Download tasks that a mod loader brings along and that cannot be derived
//! from the standard manifest go through
//! [`LoaderInstall::extra_downloads`](crate::mod_loader::LoaderInstall::extra_downloads).
//!
//! # Manifest and patches
//!
//! A mod loader does not rewrite the version manifest. It appends an
//! [`overlay::Patch`] to
//! [`InstanceManifest::patches`](manifest::InstanceManifest::patches), and
//! [`InstanceManifest::resolve`](InstanceManifest::resolve) then
//! replays every patch. The accumulating fields (`libraries`, `arguments`)
//! are cleared before the replay, so `resolve()` can be called repeatedly
//! without piling up duplicates.
//!
//! # Launch arguments
//!
//! [`variables::Variables`] collects the `${...}` variables and filters the
//! conditional arguments by feature, expanding into
//! [`arguments::LaunchArguments`]. The arguments do not go on the command
//! line directly: they are written to a temporary file that is handed to
//! java as `@file`. This sidesteps the Windows command-line length limit,
//! and incidentally keeps the access token out of the process's command
//! line. [`LaunchCommand`] holds the guard for that temporary file, so it
//! must stay alive until the process has been spawned.

use crate::auth::Authentication;
use crate::download::Downloader;
use crate::download::task::{DownloadTask, filter_existed, filter_hash};
use crate::download::vanilla::assets::AssetIndexList;
use crate::error::{Error, Result};
use crate::instance::manifest::InstanceManifest;
use crate::runtime::java::JavaRuntime;
use crate::storage::Storage;
use crate::storage::temp::TempGuard;
use crate::utils::state::{NotReady, Ready};
use futures::{AsyncReadExt, AsyncWriteExt};
use futures::{Stream, StreamExt};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

pub mod arguments;
pub mod manifest;
pub mod overlay;
pub mod variables;

pub struct Instance<'storage, R: Clone, C: Clone> {
    pub instance_dir: PathBuf,
    pub manifest: InstanceManifest,

    pub storage: &'storage Storage,

    // Runtime 的准备状态
    // JavaRuntime 或 NotReady
    /// Readiness state of the runtime
    /// Either `JavaRuntime` or `NotReady`
    pub runtime: R,
    // 游戏完整性状态
    // Ready 或 NotReady
    /// Integrity state of the game
    /// Either `Ready` or `NotReady`
    pub completeness: C,
}

impl<'storage> Instance<'storage, NotReady, NotReady> {
    // 创建实例
    /// Creates an instance
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
    // 打开本地实例
    /// Opens a local instance
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
    // 扫描实例
    /// Scans for instances
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
    // 获取当前实例的游戏文件路径
    /// Gets the path to this instance's game file
    pub fn client_file(&self) -> PathBuf {
        self.instance_dir.join(format!("{}.jar", self.manifest.jar))
    }
    // 获取当前实例 Java 版本
    /// Gets this instance's Java version
    pub fn java_major(&self) -> u32 {
        self.manifest.java_version.major_version
    }
    // 重命名
    /// Renames the instance
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
    // 复制
    /// Copies the instance
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
    // 删除
    /// Deletes the instance
    pub async fn delete(self) -> Result<()> {
        async_fs::remove_dir_all(self.instance_dir).await?;
        self.storage.clean_hardlink().await?;
        Ok(())
    }
    // 持久化版本清单
    /// Persists the version manifest
    pub async fn save(&self) -> Result<()> {
        let file = self.instance_dir.join(format!("{}.json", self.manifest.id));
        let mut file = async_fs::File::create(file).await?;
        let json = serde_json::to_vec_pretty(&self.manifest)?;
        file.write_all(&json).await?;
        Ok(())
    }
    // 修复 Assets 索引（如果打开失败）
    /// Repairs the asset index (if it fails to open)
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
    // 粗略检查游戏完整性，返回缺失文件
    // 检查 Assets 索引，不检查压缩包，不校验 Hash
    /// Roughly checks the game's integrity and returns the missing files
    /// Checks the asset index; does not check archives and does not verify
    /// hashes
    pub async fn check_exist(
        &self,
        features: HashSet<&'static str>,
    ) -> Result<impl Iterator<Item = DownloadTask<'_>>> {
        let tasks = self.install(features).await?;
        let tasks = filter_existed(tasks, false);
        Ok(tasks)
    }
    // 检查游戏完整性，返回缺失文件
    // 检查 Assets 索引，重下压缩包，校验 Hash
    /// Checks the game's integrity and returns the missing files
    /// Checks the asset index, re-downloads archives, and verifies hashes
    pub async fn check_full(
        &self,
        features: HashSet<&'static str>,
        downloader: &impl Downloader,
    ) -> Result<impl Stream<Item = DownloadTask<'_>>> {
        self.fix_assets_index(downloader).await?;
        let tasks = futures::stream::iter(self.install(features).await?);
        let tasks = filter_hash(tasks, true);
        Ok(tasks)
    }

    // 构建下载任务，并减少下载量
    /// Builds the download tasks, keeping the download volume down
    pub async fn install_less(
        &self,
        features: HashSet<&'static str>,
    ) -> Result<impl Iterator<Item = DownloadTask<'_>>> {
        let full = self.install(features).await?;
        Ok(filter_existed(full, true))
    }
    // 构建完整下载任务
    /// Builds the full set of download tasks
    pub async fn install(
        &self,
        features: HashSet<&'static str>,
    ) -> Result<impl Iterator<Item = DownloadTask<'_>>> {
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
                Some([
                    x.to_task(self.storage, f),
                    x.to_native_task(self.storage, f, n),
                ])
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
                .to_path(self.instance_dir.join(&file_name), self.storage)
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
    // 确定已经完成了完整性检查
    /// Asserts that the integrity check has been completed
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

// `async_process::Command`的包装，用于保证参数文件不会被提前删除
/// Wrapper around `async_process::Command` that keeps the argument file from
/// being deleted too early
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

// 寻找并打开实例清单 JSON
/// Finds and opens the instance manifest JSON
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
        .filter_map(async |entry| {
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
