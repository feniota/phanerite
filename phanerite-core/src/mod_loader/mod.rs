// 模组加载器
//
// 安装模组加载器不改写原来的版本清单，而是往
// [`InstanceManifest::patches`](crate::instance::manifest::InstanceManifest::patches)
// 追加一层 patch 再重放（见 [`crate::instance::overlay`]），所以同一个实例
// 可以反复安装、覆盖，原版信息始终留在最底层。
//
// # 两种安装方式
//
// [`LoaderInstall`] 抽象掉了各家加载器的差异，但差异本身很大：
//
// - [`fabric`] 只有元数据。拉一份 profile JSON 合并进清单就完事；额外的
//   库通过 [`LoaderInstall::extra_downloads`] 补上——Fabric 的库记在清单
//   的自定义字段里，标准解析路径认不出来。
// - [`neoforge`] 必须跑官方安装器。下载 installer jar，在临时目录里造一个
//   假的 `launcher_profiles.json` 骗过它，用实例绑定的 Java 执行，再把
//   产出的 `libraries` 合并回来。安装器自己发起的下载不经过本库的
//   [`Downloader`]，因此不受镜像、缓存和进度监视的管辖。
//
// 正因为 NeoForge 这条路要开 java 进程，
// [`Instance::install_loader`](crate::instance::Instance::install_loader)
// 只对 `Instance<'_, JavaRuntime, _>` 存在。
//
// [`forge`] 目前整个文件是注释掉的，尚未支持。
//
// # 版本选择
//
// [`LoaderInstall::from_version`] 只负责列出候选，选哪个由调用方的闭包
// 决定。闭包跑在异步上下文里，不要做阻塞操作；不想装就返回
// [`Error::Cancelled`](crate::error::Error::Cancelled)。
//
// [`LoaderMeta`] 要求 `Ord`，但排序依据是
// [`compare_versions`](crate::utils::version::compare_versions)——面向人类
// 可读性，不保证等于真实的发布顺序。需要精确顺序时请自己解析
// [`LoaderMeta::version`]。
//! Mod loaders
//!
//! Installing a mod loader does not rewrite the original version manifest.
//! It appends a patch to
//! [`InstanceManifest::patches`](crate::instance::manifest::InstanceManifest::patches)
//! and replays it (see [`crate::instance::overlay`]), so the same instance
//! can be installed into repeatedly and overwritten while the vanilla
//! information stays at the bottom of the stack.
//!
//! # Two ways to install
//!
//! [`LoaderInstall`] abstracts over the differences between loaders, but
//! those differences are considerable:
//!
//! - [`fabric`] is metadata only. Fetching a profile JSON and merging it into
//!   the manifest is the whole job; the extra libraries are supplied through
//!   [`LoaderInstall::extra_downloads`], because Fabric records them in a
//!   custom field of the manifest that the standard parse path does not
//!   recognise.
//! - [`neoforge`] has to run the official installer. It downloads the
//!   installer jar, fabricates a fake `launcher_profiles.json` in a temporary
//!   directory to satisfy it, runs it with the Java bound to the instance,
//!   and merges the resulting `libraries` back in. The downloads the
//!   installer itself issues do not go through this crate's [`Downloader`],
//!   so they are not subject to mirrors, caching or progress monitoring.
//!
//! Because the NeoForge path has to spawn a java process,
//! [`Instance::install_loader`](crate::instance::Instance::install_loader)
//! exists only for `Instance<'_, JavaRuntime, _>`.
//!
//! [`forge`] is currently commented out in its entirety and unsupported.
//!
//! # Choosing a version
//!
//! [`LoaderInstall::from_version`] only lists the candidates; which one to
//! take is decided by a closure the caller supplies. That closure runs in an
//! async context, so it must not block; to install nothing, return
//! [`Error::Cancelled`](crate::error::Error::Cancelled).
//!
//! [`LoaderMeta`] requires `Ord`, but the ordering comes from
//! [`compare_versions`](crate::utils::version::compare_versions), which aims
//! at human readability and is not guaranteed to match the real release
//! order. When the exact order matters, parse [`LoaderMeta::version`]
//! yourself.

use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::manifest::InstanceManifest;
use crate::mod_loader::fabric::Fabric;
use crate::runtime::java::JavaRuntime;
use crate::storage::Storage;
use std::fmt::Display;

pub mod fabric;
pub mod forge;
pub mod neoforge;

impl<R: Clone, C: Clone> Instance<'_, R, C> {
    // 由于模组加载器而存在的额外下载任务
    // 需要手动注册
    /// Extra download tasks that exist because of the mod loader
    /// They have to be registered by hand
    pub(crate) async fn extra_downloads<'cx>(
        &self,
        storage: &'cx Storage,
    ) -> Result<impl Iterator<Item = DownloadTask<'cx>>> {
        Fabric::extra_downloads(&self.manifest, storage).await
    }
}

impl<C: Clone> Instance<'_, JavaRuntime, C> {
    // 为实例安装模组加载器
    /// Installs a mod loader into the instance
    pub async fn install_loader<L: LoaderInstall>(
        &mut self,
        version: impl AsRef<str>,
        downloader: &impl Downloader,
        select: impl AsyncFnOnce(L::MetaList) -> Result<L::MetaInfo>,
    ) -> Result<()> {
        // 根据版本获取可用加载器列表
        let install = L::from_version(version, downloader).await?;
        // 执行安装
        install.install(self, select, downloader).await?;
        // 持久化修改过的 `InstanceManifest`
        self.save().await?;
        Ok(())
    }
}

// Loader 元信息
// 根据版本大小全序
/// Loader metadata
/// Totally ordered by version
pub trait LoaderMeta: Ord {
    fn name(&self) -> impl Display + '_;
    fn version(&self) -> impl Display + '_;
    fn stable(&self) -> bool;
}

#[allow(async_fn_in_trait)]
pub trait LoaderInstall: Sized {
    // 展示给用户的 Loader 元信息
    /// Loader metadata shown to the user
    type MetaInfo: LoaderMeta;
    // 当前版本可选的 Loader 列表
    /// List of loaders available for the current version
    type MetaList: IntoIterator<Item = Self::MetaInfo>;
    // 根据已有版本查找合适的 Loader
    /// Looks up a suitable loader for an existing version
    async fn from_version(version: impl AsRef<str>, downloader: &impl Downloader) -> Result<Self>;
    // 选择版本并下载 Profile，然后安装到 `Instance`
    // 留 AsyncFnOnce 给用户选择，警惕阻塞操作，不选返回 `crate::error::Error::Cancelled`
    /// Picks a version, downloads the profile and installs it into the
    /// `Instance`
    /// The `AsyncFnOnce` is left for the user to make the choice; beware of
    /// blocking operations, and return `crate::error::Error::Cancelled` when
    /// nothing is picked
    async fn install<C: Clone, S>(
        self,
        raw: &mut Instance<'_, JavaRuntime, C>,
        select: S,
        downloader: &impl Downloader,
    ) -> Result<()>
    where
        S: AsyncFnOnce(Self::MetaList) -> Result<Self::MetaInfo>;
    // 从 `InstanceManifest` 里面找出无法被正常构建的下载任务
    // 例如 `FabricLibrary` 是 Fabric 的自定义格式
    /// Finds the download tasks in an `InstanceManifest` that cannot be built
    /// the normal way
    /// For example `FabricLibrary` is Fabric's own custom format
    async fn extra_downloads<'cx>(
        manifest: &InstanceManifest,
        storage: &'cx Storage,
    ) -> Result<impl Iterator<Item = DownloadTask<'cx>>> {
        let _ = manifest;
        let _ = storage;
        Ok(std::iter::empty())
    }
}
