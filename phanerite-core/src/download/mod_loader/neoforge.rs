use crate::download::downloader::Downloader;
use crate::download::mod_loader::{LoaderInstall, LoaderMeta};
use crate::download::vanilla::version_index::Version;
use crate::error::Result;
use crate::instance::manifest::InstanceManifest;
use crate::instance::overlay::OverlayManifest;
use crate::utils::maven::MavenArtifact;
use crate::utils::version::{compare_versions, is_stable};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::io::Read;
use std::sync::LazyLock;
use tracing::debug;
use url::Url;

static NEOFORGE_MAVEN: LazyLock<Url> =
    LazyLock::new(|| "https://maven.neoforged.net/releases/".parse().unwrap());
static NEOFORGE_META: LazyLock<Url> = LazyLock::new(|| {
    NEOFORGE_MAVEN
        .join("net/neoforged/neoforge/maven-metadata.xml")
        .unwrap()
});

// static FORGE_MAVEN: LazyLock<Url> =
//     LazyLock::new(|| "https://maven.minecraftforge.net/".parse().unwrap());
// static FORGE_META: LazyLock<Url> = LazyLock::new(|| {
//     FORGE_MAVEN
//         .join("net/minecraftforge/forge/maven-metadata.xml")
//         .unwrap()
// });

pub struct NeoForge {
    group_id: String,
    artifact_id: String,
    list: Vec<NeoForgeVersion>,
}

impl LoaderInstall for NeoForge {
    type MetaInfo = NeoForgeVersion;
    type MetaList = std::vec::IntoIter<Self::MetaInfo>;
    async fn from_version(version: &Version, downloader: &Downloader) -> Result<Self> {
        let body = downloader.fetch(&NEOFORGE_META, None).await?;
        let reader = std::io::Cursor::new(body);
        let xml = serde_xml_rs::from_reader::<MetaData<NeoForgeVersion>, _>(reader)?;
        let filter = xml
            .versioning
            .versions
            .version
            .into_iter()
            .filter(|x| {
                // NeoForge 去掉了 "1."
                x.minecraft.strip_prefix("1.").unwrap_or(&x.minecraft)
                    == version.id.strip_prefix("1.").unwrap_or(&version.id)
            })
            .collect();
        Ok(Self {
            group_id: xml.group_id,
            artifact_id: xml.artifact_id,
            list: filter,
        })
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
        let maven = MavenArtifact {
            group: self.group_id,
            artifact: self.artifact_id,
            version: selected.to_string(),
            classifier: Some("installer".to_string()),
            extension: "jar".to_string(),
        };
        let url = maven.url(&NEOFORGE_MAVEN)?;

        debug!("Downloading NeoForge Installer: {url}");

        let body = downloader.fetch(&url, None).await?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(body))?;
        let mut manifest = Vec::new();
        let mut file = archive.by_name("version.json")?;
        file.read_to_end(&mut manifest)?;
        let manifest = serde_json::from_slice::<OverlayManifest>(&manifest)?;

        raw.merge_overlay(manifest, 30000);
        Ok(raw)
    }
}

impl LoaderMeta for NeoForgeVersion {
    fn name(&self) -> impl Display {
        "neoforge"
    }

    fn version(&self) -> impl Display {
        &self.neoforge
    }

    fn stable(&self) -> bool {
        is_stable(&self.neoforge)
    }
}

impl PartialOrd<Self> for NeoForgeVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NeoForgeVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_versions(&self.neoforge, &other.neoforge)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetaData<V> {
    group_id: String,
    artifact_id: String,
    versioning: Versioning<V>,
}

#[derive(Deserialize)]
struct Versioning<V> {
    // latest: ForgeVersion,
    // release: ForgeVersion,
    versions: Versions<V>,
    // last_updated: String,
}

#[derive(Deserialize)]
struct Versions<V> {
    version: Vec<V>,
}

// #[derive(Debug, PartialEq, Eq)]
// pub struct ForgeVersion {
//     pub minecraft: String,
//     pub forge: String,
// }
//
// impl<'de> Deserialize<'de> for ForgeVersion {
//     fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = String::deserialize(deserializer)?;
//
//         let (minecraft, forge) = s
//             .split_once('-')
//             .ok_or_else(|| D::Error::custom("invalid Forge version"))?;
//
//         Ok(Self {
//             minecraft: minecraft.to_string(),
//             forge: forge.to_string(),
//         })
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub struct NeoForgeVersion {
    pub minecraft: String,
    pub neoforge: String,
}

impl Display for NeoForgeVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.minecraft, self.neoforge)
    }
}

impl<'de> Deserialize<'de> for NeoForgeVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let (minecraft, neoforge) = s
            .rsplit_once('.')
            .ok_or_else(|| serde::de::Error::custom("invalid NeoForge version"))?;

        Ok(Self {
            minecraft: minecraft.to_string(),
            neoforge: neoforge.to_string(),
        })
    }
}
