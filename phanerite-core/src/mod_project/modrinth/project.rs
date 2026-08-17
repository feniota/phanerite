use crate::mod_project::ModProject;
use crate::mod_project::features::display::{ModProjectDisplayExt, Rgb};
use crate::mod_project::features::filter::ModsLoader::Other;
use crate::mod_project::features::filter::{FilterCriteria, ModProjectFilterExt};
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::fmt::Display;
use std::sync::OnceLock;
use url::Url;

pub struct Project {
    pub(super) search_project: SearchProject,
    pub(super) extended_project: ExtendedProject,
    pub(super) filter_criteria: OnceLock<FilterCriteria>,
}

impl ModProject for Project {}

impl ModProjectDisplayExt for Project {
    fn title(&self) -> impl Display + '_ {
        &self.search_project.title
    }
    fn description(&self) -> impl Display + '_ {
        &self.search_project.description
    }
    fn body(&self) -> impl Display + '_ {
        &self.extended_project.body
    }
    fn author(&self) -> impl Display + '_ {
        &self.search_project.author
    }
    fn created_time(&self) -> &DateTime<FixedOffset> {
        &self.search_project.date_created
    }
    fn updated_time(&self) -> &DateTime<FixedOffset> {
        &self.search_project.date_modified
    }
    fn license(&self) -> impl Iterator<Item = impl Display + '_> {
        self.search_project.license.split_ascii_whitespace()
    }
    fn categories(&self) -> impl Iterator<Item = impl Display + '_> {
        self.search_project.display_categories.iter()
    }
    fn icon(&self) -> Option<Url> {
        self.search_project.icon_url.parse().ok()
    }
    fn color(&self) -> Option<Rgb> {
        Some(Rgb {
            r: ((self.search_project.color >> 16) & 0xff) as u8,
            g: ((self.search_project.color >> 8) & 0xff) as u8,
            b: (self.search_project.color & 0xff) as u8,
        })
    }
    fn downloads(&self) -> Option<usize> {
        Some(self.search_project.downloads)
    }
    fn follows(&self) -> Option<usize> {
        Some(self.search_project.follows)
    }
    fn gallery(&self) -> Vec<Url> {
        self.search_project
            .gallery
            .iter()
            .filter_map(|x| x.parse().ok())
            .collect()
    }
}

impl ModProjectFilterExt for Project {
    fn filter_criteria(&self) -> &FilterCriteria {
        self.filter_criteria.get_or_init(|| FilterCriteria {
            project_type: Some(self.search_project.project_type.into()),
            mods_loader: self
                .extended_project
                .loaders
                .iter()
                .map(|x| x.parse().unwrap_or(Other))
                .collect(),
            game_version: self.search_project.versions.clone(),
            loader_version: self.extended_project.versions.clone(),
        })
    }
}

/// GET `/search` 得到的信息
/// https://docs.modrinth.com/api/operations/searchprojects/
#[derive(Deserialize)]
pub(super) struct SearchProject {
    pub(super) project_id: String,
    pub(super) project_type: ProjectType,
    // pub(super) all_project_types: Vec<ProjectType>,
    // pub(super) slug: String,
    pub(super) author: String,
    // pub(super) author_id: String,
    // pub(super) organization: Option<String>,
    // pub(super) organization_id: Option<String>,
    pub(super) title: String,
    pub(super) description: String,

    // pub(super) categories: Vec<String>,
    pub(super) display_categories: Vec<String>,
    pub(super) versions: Vec<String>,

    pub(super) downloads: usize,
    pub(super) follows: usize,

    pub(super) icon_url: String,

    pub(super) date_created: DateTime<FixedOffset>,
    pub(super) date_modified: DateTime<FixedOffset>,

    // pub(super) latest_version: String,
    pub(super) license: String,

    // pub(super) client_side: SideSupport,
    // pub(super) server_side: SideSupport,
    // pub(super) environment: Vec<Environment>,
    pub(super) gallery: Vec<String>,
    // pub(super) featured_gallery: Option<String>,
    pub(super) color: u32,
}

/// GET `/project/{id|slug}` 得到的信息
/// https://docs.modrinth.com/api/operations/getproject/
/// https://docs.modrinth.com/api/operations/getprojects/
#[derive(Deserialize)]
pub(super) struct ExtendedProject {
    pub(super) body: String,
    pub(super) loaders: Vec<String>,
    pub(super) versions: Vec<String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(super) enum ProjectType {
    Mod,
    Modpack,
    ResourcePack,
    Shader,
    Plugin,
    Datapack,
    #[serde(other)]
    Unknown,
}

impl From<ProjectType> for crate::mod_project::features::filter::ProjectType {
    fn from(value: ProjectType) -> Self {
        match value {
            ProjectType::Mod => Self::Mod,
            ProjectType::Modpack => Self::ModPack,
            ProjectType::ResourcePack => Self::ResourcePack,
            ProjectType::Shader => Self::Shader,
            _ => Self::Other,
        }
    }
}

// #[derive(Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub(super) enum SideSupport {
//     Required,
//     Optional,
//     Unsupported,
//     #[serde(other)]
//     Unknown,
// }

// #[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub(super) enum Environment {
//     ClientAndServer,
//     ClientOnly,
//     ClientOnlyServerOptional,
//     SingleplayerOnly,
//     ServerOnly,
//     ServerOnlyClientOptional,
//     DedicatedServerOnly,
//     ClientOrServer,
//     ClientOrServerPrefersBoth,
//     #[serde(other)]
//     Unknown,
// }
