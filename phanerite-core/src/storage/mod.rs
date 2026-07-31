pub mod bucket;

use crate::error::Result;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    cache_dir: OnceLock<PathBuf>,
    /// 实例目录，`{root_dir}/versions`
    versions_dir: OnceLock<PathBuf>,
    /// 运行时目录，`{root_dir}/runtime`
    /// 储存启动 Minecraft 需要的运行时
    /// 子目录命名需要满足 `RuntimePath`
    runtime_dir: OnceLock<PathBuf>,
    /// 共享储存桶目录，`{root_dir}/share`
    /// 通过 hardlink 共享文件以节约磁盘空间，需要文件系统支持
    /// 所有文件以 `Blake3` Hash 值命名，并取前两位作为目录名
    share_dir: OnceLock<PathBuf>,
    /// Library 目录，`{root_dir}/libraries`
    libraries_dir: OnceLock<PathBuf>,
    /// Asset 目录，`{root_dir}/assets`
    assets_dir: OnceLock<PathBuf>,
    /// Asset 对象目录，`{root_dir}/assets/objects`
    assets_objects: OnceLock<PathBuf>,
    /// Asset 索引目录，`{root_dir}/assets/indexes`
    assets_indexes: OnceLock<PathBuf>,
    /// `AuthlibInjector` 目录，`{root_dir]/authlib-injector`
    authlib_injector: OnceLock<PathBuf>,
    /// 共享储存桶策略
    pub(crate) share_strategy: ShareStrategy,
}

impl Storage {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root_dir = std::path::absolute(root.as_ref())?;
        if !root_dir.exists() {
            std::fs::create_dir_all(&root_dir).ok();
        }
        Ok(Self {
            root_dir,
            cache_dir: OnceLock::new(),
            versions_dir: OnceLock::new(),
            runtime_dir: OnceLock::new(),
            share_dir: OnceLock::new(),
            libraries_dir: OnceLock::new(),
            assets_dir: OnceLock::new(),
            assets_objects: OnceLock::new(),
            assets_indexes: OnceLock::new(),
            authlib_injector: OnceLock::new(),
            share_strategy: ShareStrategy::Prefer,
        })
    }
    pub fn share_strategy(mut self, strategy: ShareStrategy) -> Self {
        self.share_strategy = strategy;
        self
    }
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn cache_dir(&self) -> &Path {
        self.cache_dir.get_or_init(|| dir(&self.root_dir, "cache"))
    }
    pub fn versions_dir(&self) -> &Path {
        self.versions_dir
            .get_or_init(|| dir(&self.root_dir, "versions"))
    }
    pub fn runtime_dir(&self) -> &Path {
        self.runtime_dir
            .get_or_init(|| dir(&self.root_dir, "runtime"))
    }
    pub fn share_dir(&self) -> &Path {
        self.share_dir.get_or_init(|| dir(&self.root_dir, "share"))
    }
    pub fn libraries_dir(&self) -> &Path {
        self.libraries_dir
            .get_or_init(|| dir(&self.root_dir, "libraries"))
    }
    pub fn assets_dir(&self) -> &Path {
        self.assets_dir
            .get_or_init(|| dir(&self.root_dir, "assets"))
    }
    pub fn assets_objects(&self) -> &Path {
        self.assets_objects
            .get_or_init(|| dir(&self.root_dir, "assets/objects"))
    }
    pub fn assets_indexes(&self) -> &Path {
        self.assets_indexes
            .get_or_init(|| dir(&self.root_dir, "assets/indexes"))
    }
    pub fn authlib_injector(&self) -> &Path {
        self.authlib_injector
            .get_or_init(|| dir(&self.root_dir, "authlib-injector"))
    }
}

fn dir(root: &Path, name: &str) -> PathBuf {
    let p = root.join(name);
    let _ = std::fs::create_dir_all(&p);
    p
}

/// 清理临时文件
impl Drop for Storage {
    fn drop(&mut self) {
        if let Some(cache) = self.cache_dir.get() {
            let _ = std::fs::remove_dir_all(cache);
        }
    }
}

/// 共享储存桶策略，请勿在同一个储存桶混用，否则会导致文件缺失
#[derive(Clone, Copy)]
pub enum ShareStrategy {
    /// 完全关闭储存桶
    Off,
    /// 优先使用硬链接，否则不使用链接
    Prefer,
    /// 使用符号链接 fallback
    /// WIP: 暂未实现符号链接的计数机制
    Fallback,
    /// 强制使用硬链接
    Force,
}
