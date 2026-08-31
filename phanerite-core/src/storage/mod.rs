// 启动器的文件布局
//
// [`Storage`] 是一张路径表，不是文件系统的封装：它只回答「某类数据应该
// 放在哪」，真正的读写发生在别处。所以需要写文件的调用往往在构造阶段就
// 注入 `Storage`，而落盘要等到任务被执行的时候。
//
// # 目录能力探测
//
// 一棵目录树可能横跨多个文件系统，硬链接和符号链接不保证处处可用。
// [`Storage::new`] 会遍历整棵树逐个目录实测（见 [`capability`]），把结果
// 取交集，再据此把共享策略从 `Hardlink` 逐级降到 `Symlink`、`Move`
// （见 [`SharePreference`]）。因此 [`Storage::share_preference`] 给出的是
// 偏好而不是保证。
//
// # 共享储存桶
//
// `{root}/share` 下的文件以内容的 Blake3 值命名，取前两位作为子目录。
// 声明了 `share()` 的下载任务会先落进桶里，再按上面选出的策略链接到目标
// 位置，于是多个实例引用同一份文件时不会重复占用磁盘。
//
// 桶不自动回收。删掉实例之后要调用 [`Storage::clean_hardlink`] 清理引用
// 计数已经归零的文件和空目录（见 [`bucket`]）。
//
// # 临时文件
//
// [`temp::TempGuard`] 在 Drop 时把删除操作交给 `Storage` 自带的执行器，
// 而不是同步阻塞地删。这个执行器需要有人推动：把 [`Storage::run_cleaner`]
// 返回的 future spawn 出去，并且不要丢掉一起返回的
// [`temp::ShutdownGuard`]。没有人推动时，临时文件只能等 `Storage` 自己
// Drop 的那一刻被尽力清理一次。
//
// # 多个 Storage
//
// [`multi::MultiStorage`] 用于同时管理多个游戏目录。
// [`multi::StorageWithPlugin`] 可以把生命周期必须与 `Storage` 一致的东西
// （清理任务的 `ShutdownGuard`、带缓存的下载器……）绑在一起，避免它们被
// 提前释放。
//! The launcher's file layout
//!
//! [`Storage`] is a table of paths, not a wrapper around the filesystem: it
//! only answers "where should this kind of data live", while the actual
//! reads and writes happen elsewhere. That is why a call that needs to write
//! a file often has `Storage` injected at construction time, with the write
//! itself deferred until the task runs.
//!
//! # Directory capability probing
//!
//! A directory tree may span several filesystems, so hard links and symlinks
//! are not guaranteed to work everywhere. [`Storage::new`] walks the whole
//! tree and tests each directory empirically (see [`capability`]), takes the
//! intersection of the results, and uses it to degrade the share strategy
//! from `Hardlink` down through `Symlink` to `Move` (see
//! [`SharePreference`]). [`Storage::share_preference`] therefore states a
//! preference, not a guarantee.
//!
//! # Share bucket
//!
//! Files under `{root}/share` are named after the Blake3 hash of their
//! contents, with the first two characters used as a subdirectory. A
//! download task that declares `share()` lands in the bucket first and is
//! then linked to its target location using the strategy chosen above, so
//! that several instances referencing the same file do not each take up
//! disk space.
//!
//! The bucket is not reclaimed automatically. After deleting an instance,
//! call [`Storage::clean_hardlink`] to clear out files whose reference count
//! has dropped to zero, along with the empty directories left behind (see
//! [`clean`]).
//!
//! # Temporary files
//!
//! On drop, [`temp::TempGuard`] hands the removal over to the executor that
//! `Storage` carries, rather than blocking on it synchronously. Something
//! has to drive that executor: spawn the future returned by
//! [`Storage::run_cleaner`], and do not drop the [`temp::ShutdownGuard`]
//! returned alongside it. With nothing driving it, temporary files only get
//! one best-effort sweep at the moment `Storage` itself is dropped.
//!
//! # Several storages
//!
//! [`multi::MultiStorage`] manages several game directories at once.
//! [`multi::StorageWithPlugin`] can tie objects whose lifetime must match
//! `Storage` (the cleaner's `ShutdownGuard`, a caching downloader, ...) to
//! it, so that they are not released early.

pub mod capability;
pub mod multi;
pub mod shared;
pub mod temp;

use crate::error::{Error, Result};
use crate::storage::capability::{probe_tree, DirCapability};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// `Storage` 包含了启动器需要持久储存数据的地址
// 作为依赖注入到启动器的文件系统操作中
//
// `Storage` 并不封装文件系统的 IO 操作，仅保存常用路径
// 因此写入操作并不总是在需要注入 `Storage` 的调用
// 例如 `DownloadTask` 在构建时可能需要注入 `Storage`
// 但实际的写入操作是在执行下载时
/// `Storage` holds the locations where the launcher persists its data,
/// and is injected as a dependency into the launcher's filesystem operations
///
/// `Storage` does not wrap filesystem IO, it only keeps the commonly used
/// paths. A write therefore does not always happen in the call that `Storage`
/// is injected into: `DownloadTask`, for example, may need `Storage` injected
/// at construction time while the actual write happens when the download runs
#[derive(Debug)]
pub struct Storage {
    // 启动器数据的根目录，例如 `.minecraft`
    /// Root directory of the launcher's data, e.g. `.minecraft`
    pub root_dir: PathBuf,
    // 临时文件目录， `{root_dir}/cache`
    // `Storage` 释放时删除
    /// Temporary file directory, `{root_dir}/cache`
    /// Deleted when `Storage` is dropped
    cache_dir: PathBuf,
    // 实例目录，`{root_dir}/versions`
    /// Instance directory, `{root_dir}/versions`
    versions_dir: PathBuf,
    // 运行时目录，`{root_dir}/runtime`
    // 储存启动 Minecraft 需要的运行时
    // 子目录命名需要满足 `RuntimePath`
    /// Runtime directory, `{root_dir}/runtime`
    /// Stores the runtimes needed to launch Minecraft
    /// Subdirectory names must conform to `RuntimePath`
    runtime_dir: PathBuf,
    // 共享储存桶目录，`{root_dir}/share`
    // 通过 hardlink 共享文件以节约磁盘空间，需要文件系统支持
    // 所有文件以 `Blake3` Hash 值命名，并取前两位作为目录名
    /// Shared bucket directory, `{root_dir}/share`
    /// Shares files through hard links to save disk space; requires
    /// filesystem support
    /// Every file is named after its `Blake3` hash, with the first two
    /// characters used as the directory name
    share_dir: PathBuf,
    // Library 目录，`{root_dir}/libraries`
    /// Library directory, `{root_dir}/libraries`
    libraries_dir: PathBuf,
    // Asset 目录，`{root_dir}/assets`
    /// Asset directory, `{root_dir}/assets`
    assets_dir: PathBuf,
    // Asset 对象目录，`{root_dir}/assets/objects`
    /// Asset object directory, `{root_dir}/assets/objects`
    assets_objects: PathBuf,
    // Asset 索引目录，`{root_dir}/assets/indexes`
    /// Asset index directory, `{root_dir}/assets/indexes`
    assets_indexes: PathBuf,
    // `AuthlibInjector` 目录，`{root_dir]/authlib-injector`
    /// `AuthlibInjector` directory, `{root_dir}/authlib-injector`
    authlib_injector: PathBuf,

    // 目录能力
    /// Directory capabilities
    capability: DirCapability,
    // 共享储存桶策略
    // 根据目录能力已 Fallback
    /// Shared bucket strategy
    /// Already fell back according to the directory capabilities
    share_strategy: SharePreference,
    // 临时文件清理器
    /// Temporary file cleaner
    cleaner: Arc<async_executor::Executor<'static>>,
}

impl Storage {
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root_dir = std::path::absolute(root.as_ref())?;
        if !root_dir.exists() {
            async_fs::create_dir_all(&root_dir).await?;
        }

        let capability = probe_tree(root_dir.clone()).await;
        if !capability.read {
            return Err(Error::other("An unreadable directory exists"));
        } else if !capability.write {
            return Err(Error::other("An unwritable directory exists"));
        }

        Ok(Self {
            cache_dir: dir(&root_dir, "cache").await?,
            versions_dir: dir(&root_dir, "versions").await?,
            runtime_dir: dir(&root_dir, "runtime").await?,
            share_dir: dir(&root_dir, "share").await?,
            libraries_dir: dir(&root_dir, "libraries").await?,
            assets_dir: dir(&root_dir, "assets").await?,
            assets_objects: dir(&root_dir, "assets/objects").await?,
            assets_indexes: dir(&root_dir, "assets/indexes").await?,
            authlib_injector: dir(&root_dir, "authlib-injector").await?,
            capability,
            share_strategy: SharePreference::Hardlink.fallback(capability),
            root_dir,
            cleaner: Default::default(),
        })
    }
    // 修改偏好
    /// Changes the preference
    pub fn share_preference(mut self, share_preference: SharePreference) -> Self {
        self.share_strategy = share_preference.fallback(self.capability);
        self
    }
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }
    pub fn versions_dir(&self) -> &Path {
        self.versions_dir.as_ref()
    }
    pub fn runtime_dir(&self) -> &Path {
        self.runtime_dir.as_ref()
    }
    pub fn share_dir(&self) -> &Path {
        self.share_dir.as_ref()
    }
    pub fn share_path(&self, blake3: &blake3::Hash) -> PathBuf {
        let file_name = blake3.to_string();
        self.share_dir.join(&file_name[..2]).join(file_name)
    }
    pub fn libraries_dir(&self) -> &Path {
        self.libraries_dir.as_ref()
    }
    pub fn assets_dir(&self) -> &Path {
        self.assets_dir.as_ref()
    }
    pub fn assets_objects(&self) -> &Path {
        self.assets_objects.as_ref()
    }
    pub fn assets_indexes(&self) -> &Path {
        self.assets_indexes.as_ref()
    }
    pub fn authlib_injector(&self) -> &Path {
        self.authlib_injector.as_ref()
    }

    // 生成用于创建链接的闭包（阻塞 IO）
    /// Builds the closure used to create links (blocking IO)
    pub fn linker_blocking(&self) -> impl Fn(&Path, &Path) -> Result<()> + 'static {
        let strategy = self.share_strategy;
        move |source, target| {
            match strategy {
                SharePreference::Move => std::fs::rename(source, target)?,
                SharePreference::Symlink => symlink(source, target)?,
                SharePreference::Hardlink => std::fs::hard_link(source, target)?,
            }
            Ok(())
        }
    }
    // 生成用于创建链接的闭包
    /// Builds the closure used to create links
    pub fn linker(&self) -> impl AsyncFn(&Path, &Path) -> Result<()> + 'static {
        let strategy = self.share_strategy;
        async move |source, target| {
            match strategy {
                SharePreference::Move => async_fs::rename(source, target).await?,
                SharePreference::Symlink => symlink_async(source, target).await?,
                SharePreference::Hardlink => async_fs::hard_link(source, target).await?,
            }
            Ok(())
        }
    }

    /// Get a [`StorageIdent`] representing this [`Storage`].
    pub fn identifier(&self) -> StorageIdent {
        self.into()
    }
}

// 分平台的链接
/// Platform-specific symlink
async fn symlink_async(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    #[cfg(target_family = "unix")]
    async_fs::unix::symlink(source.as_ref(), target.as_ref()).await?;

    #[cfg(target_os = "windows")]
    async_fs::windows::symlink_file(source.as_ref(), target.as_ref()).await?;

    Ok(())
}

// 分平台的链接，用于阻塞线程
/// Platform-specific symlink, for use on a blocking thread
fn symlink(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    #[cfg(target_family = "unix")]
    std::os::unix::fs::symlink(source.as_ref(), target.as_ref())?;

    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_file(source.as_ref(), target.as_ref())?;

    Ok(())
}

// 拼接和创建目录
/// Joins and creates a directory
async fn dir(root: &Path, name: &str) -> Result<PathBuf> {
    let p = root.join(name);
    if !p.is_dir() {
        async_fs::create_dir_all(&p).await?;
    }
    Ok(p)
}

// 尝试推进清理任务，不保证异步 IO 完成
/// Tries to drive the cleanup task forward; completion of the async IO is not
/// guaranteed
impl Drop for Storage {
    fn drop(&mut self) {
        // 不保证完全清理
        while self.cleaner.try_tick() {}
    }
}

impl PartialEq for Storage {
    fn eq(&self, other: &Self) -> bool {
        self.root_dir == other.root_dir
    }
}

impl Eq for Storage {}

// 共享桶储存偏好
// 自下往上 Fallback
/// Storage preference for the shared bucket
/// Falls back from the bottom upwards
#[derive(Clone, Copy, Debug)]
pub enum SharePreference {
    // 下载后移动到目标，不共享
    // 注意：从别的方案迁移至此方案会破坏原有共享文件
    /// Move to the target after downloading, no sharing
    /// Note: migrating to this strategy from another one breaks the existing
    /// shared files
    Move,
    // 下载后符号链接到目标
    // 此方案目前会导致资源文件泄露
    /// Symlink to the target after downloading
    /// This strategy currently leaks resource files
    Symlink,
    // 下载后硬链接到目标
    // 推荐使用的方案
    /// Hard link to the target after downloading
    /// The recommended strategy
    Hardlink,
}

impl SharePreference {
    // 根据 capability 和 preference 计算 share_strategy
    /// Computes `share_strategy` from the capabilities and the preference
    fn fallback(&self, dir_capability: DirCapability) -> SharePreference {
        match self {
            SharePreference::Move => SharePreference::Move,
            SharePreference::Symlink => {
                if dir_capability.symlink {
                    SharePreference::Symlink
                } else {
                    SharePreference::Move
                }
            }
            SharePreference::Hardlink => {
                if dir_capability.hardlink {
                    SharePreference::Hardlink
                } else if dir_capability.symlink {
                    SharePreference::Symlink
                } else {
                    SharePreference::Move
                }
            }
        }
    }
}

/// A type representing a [`Storage`] which is relatively cheap to clone.
///
/// [`Storage`] has 13 fields, making it expensive to clone. By converting
/// a [`Storage`] to this type, we can identify a [`Storage`] without cloning
/// all 13 fields.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StorageIdent {
    pub root_dir: PathBuf,
}

impl From<&Storage> for StorageIdent {
    fn from(value: &Storage) -> Self {
        Self {
            root_dir: value.root_dir.clone(),
        }
    }
}

impl PartialEq<Storage> for StorageIdent {
    fn eq(&self, other: &Storage) -> bool {
        self.root_dir == other.root_dir
    }
}

impl PartialEq<StorageIdent> for Storage {
    fn eq(&self, other: &StorageIdent) -> bool {
        self.root_dir == other.root_dir
    }
}
