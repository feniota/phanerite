pub mod features;
pub mod modrinth;

use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::instance::Instance;
use futures::Stream;
use std::fmt::Display;

// 具体的项目版本
/// A concrete version of a project
pub trait ModVersion {
    fn version(&self) -> &str;
    fn change_log(&self) -> Option<impl Display + '_> {
        None::<&str>
    }
    fn download<'cx, R: Clone, C: Clone>(
        &self,
        instance: &Instance<'cx, R, C>,
    ) -> Result<DownloadTask<'cx>>;
}

// 模组项目
/// A mod project
#[allow(async_fn_in_trait)]
pub trait ModProject {
    // 具体版本，用于下载
    /// The concrete version, used for downloading
    type ModVersion: ModVersion;
    async fn versions(&self) -> Result<impl Iterator<Item = &Self::ModVersion>>;
}

// 模组仓库
/// A mod repository
#[allow(async_fn_in_trait)]
pub trait ModsRepository {
    // 仓库名称
    /// Name of the repository
    const NAME: &str;
    const ATTRIBUTION: &str = "";
    const NOTICE: &str = "";
    // 项目类型，用于筛选和信息展示
    /// The project type, used for filtering and for displaying information
    type ModType: ModProject;

    // 搜索项目
    /// Searches for projects
    fn search(&self, keyword: &str) -> impl Stream<Item = Result<Self::ModType>>;
}
