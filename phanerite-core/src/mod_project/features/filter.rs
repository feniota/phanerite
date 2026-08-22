use crate::error::Result;
use crate::mod_project::{ModProject, ModsRepository};
use futures::{Stream, StreamExt};
use strum::{AsRefStr, Display, EnumString};

// 项目类型
/// Project type
#[derive(Copy, Clone, PartialEq, Eq, AsRefStr, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ProjectType {
    Mod,
    ModPack,
    ResourcePack,
    Shader,
    Other,
}

// 模组加载器
/// Mod loader
#[derive(Clone, PartialEq, Eq, AsRefStr, EnumString, Display)]
pub enum ModsLoader {
    #[strum(serialize = "neoforge")]
    NeoForge,
    #[strum(serialize = "fabric")]
    Fabric,
    #[strum(default)]
    Other(String),
}

// 筛选条件
/// Filter criteria
#[derive(Eq)]
pub struct FilterCriteria {
    pub project_type: Option<ProjectType>,
    pub mods_loader: Vec<ModsLoader>,
    pub game_version: Vec<String>,
    pub loader_version: Vec<String>,
}

#[allow(async_fn_in_trait)]
pub trait ModFilter: ModProject {
    async fn filter_criteria(&self) -> Result<&FilterCriteria>;
}

pub struct FilteredModsRepository<'repo, R: ModsRepository<ModType: ModFilter>> {
    filter_criteria: FilterCriteria,
    repo: &'repo R,
}

pub trait ModsRepositoryFilterExt: ModsRepository<ModType: ModFilter>
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
    R: ModsRepository<ModType: ModFilter>,
{
    const NAME: &'static str = R::NAME;
    const ATTRIBUTION: &'static str = R::ATTRIBUTION;
    const NOTICE: &'static str = R::NOTICE;
    type ModType = R::ModType;
    fn search(&self, keyword: &str) -> impl Stream<Item = Result<Self::ModType>> {
        self.repo.search(keyword).filter_map(async |x| {
            let x = match x {
                Ok(x) => x,
                Err(e) => return Some(Err(e)),
            };
            let filter = match x.filter_criteria().await {
                Ok(filter) => filter,
                Err(e) => return Some(Err(e)),
            };
            if self.filter_criteria == *filter {
                Some(Ok(x))
            } else {
                None
            }
        })
    }
}
