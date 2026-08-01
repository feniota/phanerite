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

static FABRIC_META: LazyLock<Url> = LazyLock::new(|| "https://meta.fabricmc.net/".parse().unwrap());
static FABRIC_MAVEN: LazyLock<Url> =
    LazyLock::new(|| "https://maven.fabricmc.net/".parse().unwrap());

impl Version {
    pub async fn install_fabric<'a>(
        &self,
        downloader: &'a Downloader,
    ) -> Result<LoaderInstall<'a>> {
        let mut url = FABRIC_META.clone();
        url.path_segments_mut()
            .unwrap()
            .extend(["v2", "versions", "loader", &self.id]);

        let body = downloader.fetch(&url, None).await?;
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
    /// 留 AsyncFn 给用户选择，警惕阻塞操作，不选返回 `crate::error::Error::Cancelled`
    pub async fn install(
        self,
        select: impl AsyncFnOnce(Vec<LoaderMeta>) -> Result<LoaderMeta>,
    ) -> Result<InstanceManifest> {
        let LoaderInstall {
            list,
            manifest,
            downloader,
        } = self;

        let selected = select(list).await?;

        let mut url = FABRIC_META.clone();
        url.path_segments_mut().unwrap().extend([
            "v2",
            "versions",
            "loader",
            &manifest.id,
            &selected.loader.version,
            "profile",
            "json",
        ]);

        let body = downloader.fetch(&url, None).await?;
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
    /// 下载 Fabric 库
    pub(super) fn fabric_downloads(&self, storage: &Storage) -> impl Iterator<Item = DownloadTask> {
        self.fabric_libraries()
            .inspect(|x| println!("{}", x.name))
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
    // url: Url,
    sha256: Option<Sha256Hash>,
    size: Option<u64>,
}

impl FabricLibrary {
    pub fn into_download(self, storage: &Storage) -> Result<DownloadTask> {
        let url = self.name.url(&FABRIC_MAVEN)?;
        // 本库优秀的泛型设计 + 傻逼的 Fabric Meta =
        let task = match (self.size, self.sha256) {
            (Some(size), Some(hash)) => DownloadTask::builder()
                .url(url)
                .to_library(self.name.path(), storage)
                .file_name(self.name)
                .file_size(size)
                .hash(hash)
                .build(),
            (Some(size), None) => DownloadTask::builder()
                .url(url)
                .to_library(self.name.path(), storage)
                .file_name(self.name)
                .file_size(size)
                .build(),
            (None, Some(hash)) => DownloadTask::builder()
                .url(url)
                .to_library(self.name.path(), storage)
                .file_name(self.name)
                .hash(hash)
                .build(),
            (None, None) => DownloadTask::builder()
                .url(url)
                .to_library(self.name.path(), storage)
                .file_name(self.name)
                .build(),
        };
        Ok(task)
    }
}

// #[derive(Deserialize)]
// struct MainClass {
//     client: String,
//     server: String,
// }
