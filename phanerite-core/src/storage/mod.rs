pub mod bucket;
pub mod capability;

use crate::error::{Error, Result};
use crate::storage::capability::{DirCapability, probe_tree};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// `Storage` 包含了启动器需要持久储存数据的地址
/// 作为依赖注入到启动器的文件系统操作中
///
/// `Storage` 并不封装文件系统的 IO 操作，仅保存常用路径
/// 因此写入操作并不总是在需要注入 `Storage` 的调用
/// 例如 `DownloadTask` 在构建时可能需要注入 `Storage`
/// 但实际的写入操作是在执行下载时
pub struct Storage {
    /// 启动器数据的根目录，例如 `.minecraft`
    root_dir: PathBuf,
    /// 临时文件目录， `{root_dir}/cache`
    /// `Storage` 释放时删除
    cache_dir: PathBuf,
    /// 实例目录，`{root_dir}/versions`
    versions_dir: PathBuf,
    /// 运行时目录，`{root_dir}/runtime`
    /// 储存启动 Minecraft 需要的运行时
    /// 子目录命名需要满足 `RuntimePath`
    runtime_dir: PathBuf,
    /// 共享储存桶目录，`{root_dir}/share`
    /// 通过 hardlink 共享文件以节约磁盘空间，需要文件系统支持
    /// 所有文件以 `Blake3` Hash 值命名，并取前两位作为目录名
    share_dir: PathBuf,
    /// Library 目录，`{root_dir}/libraries`
    libraries_dir: PathBuf,
    /// Asset 目录，`{root_dir}/assets`
    assets_dir: PathBuf,
    /// Asset 对象目录，`{root_dir}/assets/objects`
    assets_objects: PathBuf,
    /// Asset 索引目录，`{root_dir}/assets/indexes`
    assets_indexes: PathBuf,
    /// `AuthlibInjector` 目录，`{root_dir]/authlib-injector`
    authlib_injector: PathBuf,

    /// 目录能力
    capability: DirCapability,
    /// 共享储存桶策略
    /// 根据目录能力已 Fallback
    share_strategy: SharePreference,
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
            return Err(Error::other("A non-writable directory exists"));
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
            authlib_injector: dir(&root_dir, "cache").await?,
            capability,
            share_strategy: SharePreference::Hardlink.fallback(capability),
            root_dir,
        })
    }
    /// 修改偏好
    pub fn share_preference(mut self, share_preference: SharePreference) -> Self {
        self.share_strategy = share_preference.fallback(self.capability);
        self
    }
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }
    pub fn temp_path(&self) -> PathBuf {
        self.cache_dir.join(Uuid::now_v7().to_string())
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

    pub(crate) fn linker(&self) -> impl Fn(&Path, &Path) -> Result<()> + 'static {
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
    pub(crate) fn linker_async(&self) -> impl AsyncFn(&Path, &Path) -> Result<()> + 'static {
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
}

/// 分平台的链接
async fn symlink_async(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    #[cfg(target_family = "unix")]
    let symlink = async_fs::unix::symlink(source.as_ref(), target.as_ref()).await?;

    #[cfg(target_os = "windows")]
    async_fs::windows::symlink_file(source.as_ref(), target.as_ref()).await?;

    Ok(())
}

/// 分平台的链接，用于阻塞线程
fn symlink(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    #[cfg(target_family = "unix")]
    let symlink = std::os::unix::fs::symlink(source.as_ref(), target.as_ref())?;

    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_file(source.as_ref(), target.as_ref())?;

    Ok(())
}

async fn dir(root: &Path, name: &str) -> Result<PathBuf> {
    let p = root.join(name);
    if !p.is_dir() {
        async_fs::create_dir_all(&p).await?;
    } else if p.exists() {
        return Err(Error::other(format!(
            "{} should be directory, but found file",
            p.to_string_lossy()
        )));
    }
    Ok(p)
}

/// 清理临时文件
impl Drop for Storage {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.cache_dir);
    }
}

/// 共享桶储存偏好
/// 自下往上 Fallback
#[derive(Clone, Copy)]
pub enum SharePreference {
    /// 下载后移动到目标，不共享
    /// 注意：从别的方案迁移至此方案会破坏原有共享文件
    Move,
    /// 下载后符号链接到目标
    /// 此方案目前会导致资源文件泄露
    Symlink,
    /// 下载后硬链接到目标
    /// 推荐使用的方案
    Hardlink,
}

impl SharePreference {
    /// 根据 capability 和 preference 计算 share_strategy
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
