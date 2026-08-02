use crate::download::downloader::Downloader;
use crate::download::mod_loader::{LoaderInstall, LoaderMeta};
use crate::download::task::DownloadTask;
use crate::download::vanilla::maven::MavenArtifact;
use crate::download::vanilla::version_index::Version;
use crate::error::Result;
use crate::instance::manifest::InstanceManifest;
use crate::instance::overlay::OverlayManifest;
use crate::storage::Storage;
use crate::utils::Sha256Hash;
use serde::Deserialize;
use std::sync::LazyLock;
use url::Url;

static FABRIC_META: LazyLock<Url> = LazyLock::new(|| "https://meta.fabricmc.net/".parse().unwrap());
static FABRIC_MAVEN: LazyLock<Url> =
    LazyLock::new(|| "https://maven.fabricmc.net/".parse().unwrap());

pub struct Fabric {
    list: Vec<MetaData>,
}

impl LoaderInstall for Fabric {
    type MetaInfo = MetaData;
    type MetaList = std::vec::IntoIter<Self::MetaInfo>;
    async fn from_version(version: &Version, downloader: &Downloader) -> Result<Self> {
        let mut url = FABRIC_META.clone();
        url.path_segments_mut()
            .unwrap()
            .extend(["v2", "versions", "loader", &version.id]);

        let body = downloader.fetch(&url, None).await?;
        let json = serde_json::from_slice::<Vec<MetaData>>(&body)?;

        Ok(Fabric { list: json })
    }
    async fn install<S>(
        self,
        mut raw: InstanceManifest,
        select: S,
        downloader: &Downloader,
    ) -> Result<InstanceManifest>
    where
        S: AsyncFnOnce(Self::MetaList) -> Result<Self::MetaInfo>,
    {
        let selected = select(self.list.into_iter()).await?;

        let mut url = FABRIC_META.clone();
        url.path_segments_mut().unwrap().extend([
            "v2",
            "versions",
            "loader",
            &raw.id,
            &selected.loader.version,
            "profile",
            "json",
        ]);

        let body = downloader.fetch(&url, None).await?;
        let json = serde_json::from_slice::<OverlayManifest>(&body)?;
        raw.merge_overlay(json, 30000);
        Ok(raw)
    }
    async fn extra_downloads(
        manifest: &InstanceManifest,
        storage: &Storage,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        Ok(fabric_libraries(manifest).filter_map(|x| x.into_download(storage).ok()))
    }
}

impl LoaderMeta for MetaData {
    fn name(&self) -> &str {
        &self.loader.maven.artifact
    }

    fn version(&self) -> &str {
        &self.loader.version
    }

    fn stable(&self) -> bool {
        self.loader.stable
    }
}

/// 查找 Fabric 库
fn fabric_libraries(manifest: &InstanceManifest) -> impl Iterator<Item = FabricLibrary> {
    manifest.libraries.iter().filter_map(|x| {
        (x.extra.get("url")? == "https://maven.fabricmc.net/").then_some(FabricLibrary {
            name: x.name.clone(),
            sha256: x
                .extra
                .get("sha256")
                .and_then(|t| serde_json::from_value(t.clone()).ok()?),
            size: x
                .extra
                .get("size")
                .and_then(|t| serde_json::from_value(t.clone()).ok()?),
        })
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    loader: Loader,
    // intermediary: Intermediary,
    // launcher_meta: LauncherMeta,
}

#[derive(Deserialize)]
struct Loader {
    // separator: String,
    // build: usize,
    maven: MavenArtifact,
    version: String,
    stable: bool,
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
struct FabricLibrary {
    name: MavenArtifact,
    // url: Url,
    sha256: Option<Sha256Hash>,
    size: Option<u64>,
}

impl FabricLibrary {
    fn into_download(self, storage: &Storage) -> Result<DownloadTask> {
        let mut builder = DownloadTask::builder()
            .url(self.name.url(&FABRIC_MAVEN)?)
            .to_library(self.name.path(), storage);

        if let Some(size) = self.size {
            builder = builder.file_size(size);
        }

        if let Some(hash) = self.sha256 {
            builder = builder.hash(hash);
        }

        Ok(builder.build())
    }
}

// #[derive(Deserialize)]
// struct MainClass {
//     client: String,
//     server: String,
// }
