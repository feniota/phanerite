use crate::download::task::DownloadTask;
use crate::error::{Error, Result};
use crate::mod_project::ModVersion;
use crate::utils::Sha1Hash;
use serde::Deserialize;
use std::fmt::Display;
use std::path::Path;
use url::Url;

impl ModVersion for Version {
    fn version(&self) -> &str {
        &self.version_number
    }
    fn change_log(&self) -> Option<impl Display + '_> {
        self.changelog.as_ref()
    }
    fn download(self, dir: impl AsRef<Path>) -> Result<DownloadTask> {
        let file = self
            .files
            .iter()
            .find(|file| file.primary)
            .or_else(|| self.files.first())
            .ok_or(Error::other("No available file"))?;
        Ok(DownloadTask::builder()
            .url(file.url.clone())
            .to_path(dir.as_ref().join(file.filename.clone()))
            .hash(file.hashes.sha1.clone())
            .file_name(file.filename.clone())
            .file_size(file.size)
            .share()
            .build())
    }
}

#[derive(Deserialize)]
pub struct Version {
    // pub(super) name: String,
    pub(super) version_number: String,
    pub(super) changelog: Option<String>,
    pub(super) dependencies: Vec<Dependency>,
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

#[derive(Deserialize)]
pub(super) struct Dependency {
    pub(super) version_id: Option<String>,
    pub(super) project_id: Option<String>,
    // pub(super) file_name: Option<String>,
    pub(super) dependency_type: DependencyType,
}

#[derive(Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,

    #[serde(other)]
    Unknown,
}

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
    sha1: Sha1Hash,
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
