use crate::download::downloader::Downloader;
use crate::download::mod_loader::fabric::Fabric;
use crate::download::task::DownloadTask;
use crate::download::vanilla::version_index::Version;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::manifest::InstanceManifest;
use crate::storage::Storage;
use std::fmt::Display;

pub mod fabric;
pub mod forge;
pub mod neoforge;

impl<R, C> Instance<'_, R, C> {
    /// 由于模组加载器而存在的额外下载任务
    /// 需要手动注册
    pub(crate) async fn extra_downloads(
        &self,
        storage: &Storage,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        Fabric::extra_downloads(&self.manifest, storage).await
    }
}

impl Version {
    /// 获取有模组加载器的版本清单
    pub async fn install_loader<L: LoaderInstall>(
        &self,
        downloader: &Downloader,
        select: impl AsyncFnOnce(L::MetaList) -> Result<L::MetaInfo>,
    ) -> Result<InstanceManifest> {
        let install = L::from_version(self, downloader).await?;
        let raw = self.get_manifest(downloader).await?;
        install.install(raw.into(), select, downloader).await
    }
}

/// Loader 元信息
/// 根据版本大小全序
pub trait LoaderMeta: Ord {
    fn name(&self) -> impl Display;
    fn version(&self) -> impl Display;
    fn stable(&self) -> bool;
}

#[allow(async_fn_in_trait)]
pub trait LoaderInstall: Sized {
    /// 展示给用户的 Loader 元信息
    type MetaInfo: LoaderMeta;
    /// 当前版本可选的 Loader 列表
    type MetaList: Iterator<Item = Self::MetaInfo>;
    /// 根据已有版本查找合适的 Loader
    async fn from_version(version: &Version, downloader: &Downloader) -> Result<Self>;
    /// 选择版本并下载 Profile，合并出带有 Loader 的 `InstanceManifest`
    /// 留 AsyncFnOnce 给用户选择，警惕阻塞操作，不选返回 `crate::error::Error::Cancelled`
    async fn install<S>(
        self,
        raw: InstanceManifest,
        select: S,
        downloader: &Downloader,
    ) -> Result<InstanceManifest>
    where
        S: AsyncFnOnce(Self::MetaList) -> Result<Self::MetaInfo>;
    /// 从 `InstanceManifest` 里面找出无法被正常构建的下载任务
    /// 例如 `FabricLibrary`
    async fn extra_downloads(
        _manifest: &InstanceManifest,
        _storage: &Storage,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        Ok(std::iter::empty())
    }
}
