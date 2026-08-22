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
