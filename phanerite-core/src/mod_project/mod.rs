pub mod features;
pub mod modrinth;

use crate::download::task::DownloadTask;
use crate::error::Result;
use futures::Stream;
use std::fmt::Display;
use std::path::Path;

/// 具体的项目版本
pub trait ModVersion {
    fn version(&self) -> &str;
    fn change_log(&self) -> Option<impl Display + '_> {
        None::<&str>
    }
    fn download(self, dir: impl AsRef<Path>) -> Result<DownloadTask>;
}

/// 模组项目
pub trait ModProject {}

/// 模组仓库
#[allow(async_fn_in_trait)]
pub trait ModsRepository {
    /// 仓库名称
    const NAME: &str;
    const ATTRIBUTION: &str = "";
    const NOTICE: &str = "";
    /// 项目类型，用于筛选和信息展示
    type ModType: ModProject;
    /// 具体版本，用于下载
    type ModVersion: ModVersion;
    /// 搜索项目
    fn search(&self, keyword: &str) -> impl Stream<Item = Result<Self::ModType>>;
    /// 获取项目版本
    async fn versions(
        &self,
        project: &Self::ModType,
    ) -> Result<impl Iterator<Item = Self::ModVersion>>;
    /// 获取依赖
    async fn dependencies(
        &self,
        version: &Self::ModVersion,
    ) -> Result<impl Iterator<Item = Self::ModVersion>>;
}
