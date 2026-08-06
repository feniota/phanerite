use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::manifest::InstanceManifest;
use crate::instance::overlay::OverlayManifest;
use crate::mod_loader::{LoaderInstall, LoaderMeta};
use crate::runtime::java::JavaRuntime;
use crate::storage::Storage;
use crate::utils::Sha256Hash;
use crate::utils::maven::MavenArtifact;
use crate::utils::version::compare_versions;
use serde::Deserialize;
use std::cmp::Ordering;
use std::fmt::Display;
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
    async fn from_version(version: impl AsRef<str>, downloader: &impl Downloader) -> Result<Self> {
        let mut url = FABRIC_META.clone();
        url.path_segments_mut()
            .unwrap()
            .extend(["v2", "versions", "loader", version.as_ref()]);

        let body = downloader.fetch(url, None).await?;
        let json = serde_json::from_slice::<Vec<MetaData>>(&body)?;

        Ok(Fabric { list: json })
    }
    async fn install<C, S>(
        self,
        raw: &mut Instance<'_, JavaRuntime, C>,
        select: S,
        downloader: &impl Downloader,
    ) -> Result<()>
    where
        S: AsyncFnOnce(Self::MetaList) -> Result<Self::MetaInfo>,
    {
        let selected = select(self.list.into_iter()).await?;

        let mut url = FABRIC_META.clone();
        url.path_segments_mut().unwrap().extend([
            "v2",
            "versions",
            "loader",
            &raw.manifest.id,
            &selected.loader.version,
            "profile",
            "json",
        ]);

        let body = downloader.fetch(url, None).await?;
        let json = serde_json::from_slice::<OverlayManifest>(&body)?;
        raw.manifest.merge_overlay(json, 30000);
        Ok(())
    }
    async fn extra_downloads(
        manifest: &InstanceManifest,
        storage: &Storage,
    ) -> Result<impl Iterator<Item = DownloadTask>> {
        Ok(fabric_libraries(manifest).filter_map(|x| x.into_download(storage).ok()))
    }
}

impl LoaderMeta for MetaData {
    fn name(&self) -> impl Display {
        &self.loader.maven.artifact
    }

    fn version(&self) -> impl Display {
        &self.loader.version
    }

    fn stable(&self) -> bool {
        self.loader.stable
    }
}

impl PartialOrd<Self> for MetaData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetaData {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_versions(&self.loader.version, &other.loader.version)
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

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetaData {
    loader: Loader,
    // intermediary: Intermediary,
    // launcher_meta: LauncherMeta,
}

#[derive(Deserialize, PartialEq, Eq)]
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
