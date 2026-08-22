// 模组仓库
//
// 三层 trait 对应三层概念：[`ModsRepository`] 是一个站点，[`ModProject`]
// 是站点上的一个项目，[`ModVersion`] 是项目的一个具体版本——只有到了版本
// 这一层才谈得上下载。
//
// [`ModVersion::download`] 返回的同样是
// [`DownloadTask`]，和本库其它地方
// 一致：生成任务，不执行。
//
// # 搜索是流
//
// [`ModsRepository::search`] 返回 [`Stream`] 而不是
// `Vec`。仓库都是分页的，用流可以边翻页边出结果，调用方取多少就翻多少。
//
// # 署名
//
// [`ModsRepository::ATTRIBUTION`] 和 [`ModsRepository::NOTICE`] 是仓库要求
// 的署名与声明。把它们放在 trait 上而不是写进文档，是因为展示它们通常是
// 使用该仓库 API 的前提，界面层需要能拿到。两者默认为空串，目前
// [`modrinth`] 还没有填。
//
// # 目前的实现
//
// [`modrinth`] 是唯一的实现。[`features`] 里是可选的能力扩展：
// [`features::filter`] 按项目类型、加载器和游戏版本过滤，
// [`features::display`] 是给界面用的展示接口。
//! Mod repositories
//!
//! Three traits mirror three concepts: [`ModsRepository`] is a site,
//! [`ModProject`] is a project on that site, and [`ModVersion`] is a
//! concrete version of that project — only at the version level does
//! downloading become meaningful.
//!
//! [`ModVersion::download`] likewise returns a
//! [`DownloadTask`], consistent with the
//! rest of this crate: it produces a task, it does not run one.
//!
//! # Search is a stream
//!
//! [`ModsRepository::search`] returns a [`Stream`] rather
//! than a `Vec`. Repositories are paginated, and a stream lets results come
//! out while paging continues, so the caller pages exactly as far as it
//! consumes.
//!
//! # Attribution
//!
//! [`ModsRepository::ATTRIBUTION`] and [`ModsRepository::NOTICE`] carry the
//! attribution and notices a repository requires. They sit on the trait
//! rather than in prose because displaying them is usually a precondition for
//! using that repository's API, and the UI layer has to be able to reach
//! them. Both default to the empty string, and [`modrinth`] does not fill
//! them in yet.
//!
//! # Current implementations
//!
//! [`modrinth`] is the only implementation. [`features`] holds optional
//! capability extensions: [`features::filter`] filters by project type,
//! loader and game version, and [`features::display`] is the presentation
//! interface meant for a UI.

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
