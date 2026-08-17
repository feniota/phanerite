use crate::error::Result;
use crate::mod_project::{ModProject, ModsRepository};
use futures::Stream;
use futures::StreamExt;
use strum::{AsRefStr, Display, EnumString};

/// 项目类型
#[derive(Copy, Clone, PartialEq, Eq, AsRefStr, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ProjectType {
    Mod,
    ModPack,
    ResourcePack,
    Shader,
    Other,
}

/// 模组加载器
#[derive(Copy, Clone, PartialEq, Eq, AsRefStr, EnumString, Display)]
pub enum ModsLoader {
    #[strum(serialize = "neoforge")]
    NeoForge,
    #[strum(serialize = "fabric")]
    Fabric,

    Other,
}

/// 筛选条件
#[derive(Eq)]
pub struct FilterCriteria {
    pub project_type: Option<ProjectType>,
    pub mods_loader: Vec<ModsLoader>,
    pub game_version: Vec<String>,
    pub loader_version: Vec<String>,
}

pub trait ModProjectFilterExt: ModProject {
    fn filter_criteria(&self) -> &FilterCriteria;
}

pub struct FilteredModsRepository<'repo, R: ModsRepository<ModType: ModProjectFilterExt>> {
    filter_criteria: FilterCriteria,
    repo: &'repo R,
}

pub impl(crate) trait ModsRepositoryFilterExt:
    ModsRepository<ModType: ModProjectFilterExt>
where
    Self: Sized,
{
    fn filtered(&self, filter_criteria: FilterCriteria) -> impl ModsRepository {
        FilteredModsRepository {
            filter_criteria,
            repo: self,
        }
    }
}

impl PartialEq for FilterCriteria {
    fn eq(&self, other: &Self) -> bool {
        self.project_type
            .is_none_or(|t| Some(t) == other.project_type)
            && self
                .mods_loader
                .iter()
                .any(|t| other.mods_loader.contains(t))
            && self
                .game_version
                .iter()
                .any(|t| other.game_version.contains(t))
            && self
                .loader_version
                .iter()
                .any(|t| other.loader_version.contains(t))
    }
}

impl<R> ModsRepository for FilteredModsRepository<'_, R>
where
    R: ModsRepository<ModType: ModProjectFilterExt>,
{
    const NAME: &'static str = R::NAME;
    const ATTRIBUTION: &'static str = R::ATTRIBUTION;
    const NOTICE: &'static str = R::NOTICE;
    type ModType = R::ModType;
    type ModVersion = R::ModVersion;
    fn search(&self, keyword: &str) -> impl Stream<Item = Result<Self::ModType>> {
        self.repo.search(keyword).filter(|x| {
            futures::future::ready(match x {
                Ok(v) => &self.filter_criteria == v.filter_criteria(),
                Err(_) => true,
            })
        })
    }
    async fn versions(
        &self,
        project: &Self::ModType,
    ) -> Result<impl Iterator<Item = Self::ModVersion>> {
        self.repo.versions(project).await
    }
    async fn dependencies(
        &self,
        version: &Self::ModVersion,
    ) -> Result<impl Iterator<Item = Self::ModVersion>> {
        self.repo.dependencies(version).await
    }
}
