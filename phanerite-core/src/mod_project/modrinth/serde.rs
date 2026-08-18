use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use url::Url;

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

    pub(super) icon_url: Url,

    pub(super) date_created: DateTime<FixedOffset>,
    pub(super) date_modified: DateTime<FixedOffset>,

    // pub(super) latest_version: String,
    pub(super) license: String,

    // pub(super) client_side: SideSupport,
    // pub(super) server_side: SideSupport,
    // pub(super) environment: Vec<Environment>,
    pub(super) gallery: Vec<Url>,
    // pub(super) featured_gallery: Option<String>,
    pub(super) color: u32,
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

/// GET `/project/{id|slug}` 得到的信息
/// https://docs.modrinth.com/api/operations/getproject/
/// https://docs.modrinth.com/api/operations/getprojects/
#[derive(Deserialize)]
pub(super) struct DetailProject {
    // pub(super) id: String,
    // pub(super) team: String,
    // pub(super) title: String,
    // pub(super) description: String,
    pub(super) body: String,

    // pub(super) status: ProjectStatus,
    // pub(super) project_type: ProjectType,
    // pub(super) categories: Vec<String>,
    // pub(super) additional_categories: Vec<String>,
    // pub(super) environment: Vec<Environment>,
    // pub(super) game_versions: Vec<String>,
    pub(super) loaders: Vec<String>,
    pub(super) versions: Vec<String>,
    // pub(super) license: License,
    // pub(super) published: Option<DateTime<FixedOffset>>,
    // pub(super) updated: DateTime<FixedOffset>,
    // pub(super) downloads: u64,
    // pub(super) followers: u64,
    // pub(super) gallery: Vec<GalleryImage>,
    // pub(super) thread_id: String,
    // pub(super) monetization_status: MonetizationStatus,
    // pub(super) slug: Option<String>,
    // pub(super) organization: Option<String>,
    // pub(super) requested_status: Option<RequestedStatus>,
    // pub(super) queued: Option<DateTime<FixedOffset>>,
    // pub(super) icon_url: Option<Url>,
    // pub(super) raw_icon_url: Option<Url>,
    // pub(super) color: Option<u32>,
    // pub(super) issues_url: Option<Url>,
    // pub(super) source_url: Option<Url>,
    // pub(super) wiki_url: Option<Url>,
    // pub(super) discord_url: Option<Url>,
    // pub(super) donation_urls: Option<Vec<DonationUrl>>,

    // Deprecated
    // pub(super) client_side: SideSupport,
    // pub(super) server_side: SideSupport,

    // Deprecated，API 文档明确说总是 null
    // pub(super) body_url: Option<Url>,

    // Deprecated
    // pub(super) moderator_message: Option<ModeratorMessage>,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "lowercase")]
// pub(super) enum ProjectStatus {
//     Approved,
//     Archived,
//     Rejected,
//     Draft,
//     Unlisted,
//     Processing,
//     Withheld,
//     Scheduled,
//     Private,
//     #[serde(other)]
//     Unknown,
// }

// #[derive(Deserialize)]
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

// #[derive(Deserialize)]
// pub(super) struct License {
//     pub(super) id: String,
//     pub(super) name: String,
//     pub(super) url: Option<Url>,
// }

// #[derive(Deserialize)]
// pub(super) struct GalleryImage {
//     pub(super) url: Url,
//     pub(super) featured: bool,
//
//     pub(super) title: Option<String>,
//     pub(super) description: Option<String>,
//
//     pub(super) created: DateTime<FixedOffset>,
//     pub(super) ordering: i32,
//
//     pub(super) thread_id: String,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "kebab-case")]
// pub(super) enum MonetizationStatus {
//     Monetized,
//     Demonetized,
//     ForceDemonetized,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "lowercase")]
// pub(super) enum RequestedStatus {
//     Approved,
//     Archived,
//     Unlisted,
//     Private,
//     Draft,
// }

// #[derive(Deserialize)]
// pub(super) struct DonationUrl {
//     pub(super) id: String,
//     pub(super) platform: String,
//     pub(super) url: Url,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "lowercase")]
// pub(super) enum SideSupport {
//     Required,
//     Optional,
//     Unsupported,
//     #[serde(other)]
//     Unknown,
// }

// #[derive(Deserialize)]
// pub(super) struct ModeratorMessage {
//     pub(super) message: String,
//     pub(super) body: Option<String>,
// }

/// https://docs.modrinth.com/api/operations/getprojectversions/
#[derive(Deserialize)]
pub struct Version {
    // pub(super) name: String,
    pub(super) version_number: String,
    pub(super) changelog: Option<String>,
    // pub(super) dependencies: Vec<Dependency>,
    // pub(super) game_versions: Vec<String>,
    // pub(super) version_type: VersionType,
    // pub(super) loaders: Vec<String>,
    // pub(super) featured: bool,
    // pub(super) status: VersionStatus,
    // pub(super) requested_status: Option<RequestedStatus>,
    // pub(super) id: String,
    // pub(super) project_id: String,
    // pub(super) author_id: String,
    // pub(super) date_published: DateTime<FixedOffset>,
    // pub(super) downloads: u64,
    // pub(super) environment: Environment,
    pub(super) files: Vec<VersionFile>,
}

// #[derive(Deserialize)]
// pub(super) struct Dependency {
//     pub(super) version_id: Option<String>,
//     pub(super) project_id: Option<String>,
//     // pub(super) file_name: Option<String>,
//     pub(super) dependency_type: DependencyType,
// }

// #[derive(Deserialize, Eq, PartialEq)]
// #[serde(rename_all = "snake_case")]
// pub(super) enum DependencyType {
//     Required,
//     Optional,
//     Incompatible,
//     Embedded,
//
//     #[serde(other)]
//     Unknown,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub(super) enum VersionType {
//     Release,
//     Beta,
//     Alpha,
//
//     #[serde(other)]
//     Unknown,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub(super) enum VersionStatus {
//     Listed,
//     Archived,
//     Draft,
//     Unlisted,
//     Scheduled,
//
//     #[serde(other)]
//     Unknown,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub(super) enum RequestedStatus {
//     Listed,
//     Archived,
//     Draft,
//     Unlisted,
//
//     #[serde(other)]
//     Unknown,
// }

#[derive(Deserialize)]
pub(super) struct VersionFile {
    pub(super) hashes: Hashes,

    pub(super) url: Url,
    pub(super) filename: String,

    pub(super) primary: bool,

    pub(super) size: u64,
    // pub(super) file_type: Option<FileType>,
}

#[derive(Deserialize)]
pub(super) struct Hashes {
    pub(super) sha1: Sha1Hash,
    // sha512: String,
}

// #[derive(Deserialize)]
// #[serde(rename_all = "kebab-case")]
// pub(super) enum FileType {
//     RequiredResourcePack,
//     OptionalResourcePack,
//     SourcesJar,
//     DevJar,
//     JavadocJar,
//     Signature,
//
//     #[serde(other)]
//     Unknown,
// }
