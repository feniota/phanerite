use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::download::vanilla::maven::MavenArtifact;
use crate::download::vanilla::version_index::Version;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::instance_info::InstanceManifest;
use crate::instance::overlay::OverlayManifest;
use crate::storage::Storage;
use crate::utils::Sha256Hash;
use serde::Deserialize;
use std::sync::LazyLock;
use url::Url;

static FABRIC_META: LazyLock<Url> = LazyLock::new(|| "https://meta.fabricmc.net".parse().unwrap());

fn fabric_intermediary_url(game_version: &str) -> Url {
    let mut url = FABRIC_META.clone();

    url.path_segments_mut()
        .unwrap()
        .extend(["v2", "versions", "intermediary", game_version]);

    url
}

fn fabric_profile_url(game_version: &str, loader_version: &str) -> Url {
    let mut url = FABRIC_META.clone();

    url.path_segments_mut().unwrap().extend([
        "v2",
        "versions",
        "loader",
        game_version,
        loader_version,
        "profile",
        "json",
    ]);

    url
}

impl Version {
    pub async fn install_fabric<'a>(
        &self,
        downloader: &'a Downloader,
    ) -> Result<LoaderInstall<'a>> {
        let body = downloader
            .fetch(&fabric_intermediary_url(&self.id), None)
            .await?;
        let json = serde_json::from_slice::<Vec<LoaderMeta>>(&body)?;
        Ok(LoaderInstall {
            downloader,
            manifest: self.get_manifest(downloader).await?.into(),
            list: json,
        })
    }
}

impl<'a> LoaderInstall<'a> {
    /// 选择版本并下载 Profile
    pub async fn install<F>(
        self,
        mut select: impl AsyncFnMut(Vec<LoaderMeta>) -> Result<LoaderMeta>,
    ) -> Result<InstanceManifest> {
        let LoaderInstall {
            list,
            manifest,
            downloader,
        } = self;
        let selected = select(list).await?;
        let body = downloader
            .fetch(
                &fabric_profile_url(&manifest.id, &selected.loader.version),
                None,
            )
            .await?;
        let json = serde_json::from_slice::<OverlayManifest>(&body)?;
        let merged = json.merge(manifest);
        Ok(merged)
    }
}

impl Instance {
    /// 查找 Fabric 库
    fn fabric_libraries(&self) -> impl Iterator<Item = FabricLibrary> {
        let libraries = self.manifest.libraries.iter();
        libraries.filter_map(|x| {
            Some(FabricLibrary {
                name: x.name.clone(),
                url: serde_json::from_value(x.extra.get("url")?.clone()).ok()?,
                sha256: serde_json::from_value(x.extra.get("sha256")?.clone()).ok()?,
                size: serde_json::from_value(x.extra.get("size")?.clone()).ok()?,
            })
        })
    }
    /// 下载 Fabric 库
    pub(super) fn fabric_downloads(&self, storage: &Storage) -> impl Iterator<Item = DownloadTask> {
        self.fabric_libraries()
            .filter_map(|x| x.into_download(storage).ok()) // 不应该出现解析失败的 URL
    }
}

pub struct LoaderInstall<'a> {
    downloader: &'a Downloader,
    manifest: InstanceManifest,
    list: Vec<LoaderMeta>,
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
pub struct FabricLibrary {
    name: MavenArtifact,
    url: Url,
    sha256: Sha256Hash,
    size: u64,
}

impl FabricLibrary {
    pub fn into_download(self, storage: &Storage) -> Result<DownloadTask> {
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
