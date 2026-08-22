/*
use crate::download::downloader::Downloader;
use crate::download::vanilla::version_index::Version;
use crate::error::Result;
use crate::instance::manifest::InstanceManifest;
use crate::instance::overlay::OverlayManifest;
use crate::mod_loader::neoforge::MetaData;
use crate::mod_loader::{LoaderInstall, LoaderMeta};
use crate::utils::maven::MavenArtifact;
use crate::utils::version::{compare_versions, is_stable};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::io::Read;
use std::sync::LazyLock;
use tracing::debug;
use url::Url;

static FORGE_MAVEN: LazyLock<Url> =
    LazyLock::new(|| "https://maven.minecraftforge.net/".parse().unwrap());
static FORGE_META: LazyLock<Url> = LazyLock::new(|| {
    FORGE_MAVEN
        .join("net/minecraftforge/forge/maven-metadata.xml")
        .unwrap()
});

// Forge 可能不想让你从 Maven 下载 client
/// Forge probably does not want you to download the client from Maven
pub static UNAVAILABLE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://unavailable.invalid").unwrap());

pub struct Forge {
    group_id: String,
    artifact_id: String,
    list: Vec<ForgeVersion>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ForgeVersion {
    pub minecraft: String,
    pub forge: String,
}

impl LoaderInstall for Forge {
    type MetaInfo = ForgeVersion;
    type MetaList = std::vec::IntoIter<Self::MetaInfo>;
    async fn from_version(version: &Version, downloader: &Downloader) -> Result<Self> {
        let body = downloader.fetch(&FORGE_META, None).await?;
        let reader = std::io::Cursor::new(body);
        let xml = serde_xml_rs::from_reader::<MetaData<ForgeVersion>, _>(reader)?;
        let filter = xml
            .versioning
            .versions
            .version
            .into_iter()
            .filter(|x| x.minecraft == version.id)
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
        let url = maven.url(&FORGE_MAVEN)?;

        debug!("Downloading Forge Installer: {url}");

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

impl LoaderMeta for ForgeVersion {
    fn name(&self) -> impl Display {
        "forge"
    }

    fn version(&self) -> impl Display {
        &self.forge
    }

    fn stable(&self) -> bool {
        is_stable(&self.forge)
    }
}

impl Ord for ForgeVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_versions(&self.forge, &other.forge)
    }
}

impl PartialOrd<Self> for ForgeVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for ForgeVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.minecraft, self.forge)
    }
}

impl<'de> Deserialize<'de> for ForgeVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let (minecraft, forge) = s
            .split_once('-')
            .ok_or_else(|| serde::de::Error::custom("invalid Forge version"))?;

        Ok(Self {
            minecraft: minecraft.to_string(),
            forge: forge.to_string(),
        })
    }
}
*/
