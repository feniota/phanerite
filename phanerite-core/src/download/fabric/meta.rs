use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::download::vanilla::maven::MavenArtifact;
use crate::download::vanilla::version_index::Version;
use crate::error::Result;
use crate::storage::Storage;
use crate::utils::Sha256Hash;
use serde::Deserialize;
use url::Url;

const FABRIC_META: &str = "https://meta.fabricmc.net";

impl LoaderList {
    pub async fn get(version: &Version, downloader: &Downloader) -> Result<Self> {
        let body = downloader
            .fetch(
                format!("{}/v2/versions/loader/{}", FABRIC_META, version.id),
                None,
            )
            .await?;
        let json = serde_json::from_slice(&body)?;
        Ok(json)
    }
}

#[derive(Deserialize)]
pub struct LoaderList {
    pub list: Vec<LoaderMeta>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderMeta {
    pub loader: Loader,
    // intermediary: Intermediary,
    // launcher_meta: LauncherMeta,
}

#[derive(Deserialize)]
pub struct Loader {
    pub separator: String,
    pub build: usize,
    pub maven: MavenArtifact,
    pub version: String,
    pub stable: bool,
}

// #[derive(Deserialize)]
// struct Intermediary {
//     maven: MavenArtifact,
//     version: String,
//     stable: bool,
// }

// #[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// struct LauncherMeta {
//     version: usize,
//     min_java_version: usize,
//     libraries: Libraries,
//     main_class: MainClass,
// }

// #[derive(Deserialize)]
// struct Libraries {
//     client: Vec<Library>,
//     common: Vec<Library>,
//     server: Vec<Library>,
// }

#[derive(Deserialize)]
pub(super) struct Library {
    name: MavenArtifact,
    url: Url,
    sha256: Sha256Hash,
    size: u64,
}

impl Library {
    pub(super) fn into_download(self, storage: &Storage) -> Result<DownloadTask> {
        Ok(DownloadTask::builder()
            .url(self.name.url(&self.url)?)
            .to_library(self.name.path(), storage)
            .file_name(self.name)
            .file_size(self.size)
            .hash(self.sha256)
            .build())
    }
}

// #[derive(Deserialize)]
// struct MainClass {
//     client: String,
//     server: String,
// }
