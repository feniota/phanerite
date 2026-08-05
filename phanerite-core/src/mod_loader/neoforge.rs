use crate::download::downloader::Downloader;
use crate::error::Result;
use crate::instance::Instance;
use crate::mod_loader::{LoaderInstall, LoaderMeta};
use crate::runtime::java::JavaRuntime;
use crate::utils::maven::MavenArtifact;
use crate::utils::version::{compare_versions, is_stable};
use futures::AsyncWriteExt;
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
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

pub struct NeoForge {
    group_id: String,
    artifact_id: String,
    list: Vec<NeoForgeVersion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MetaData<V> {
    pub(crate) group_id: String,
    pub(crate) artifact_id: String,
    pub(crate) versioning: Versioning<V>,
}

#[derive(Deserialize)]
pub(crate) struct Versioning<V> {
    // latest: ForgeVersion,
    // release: ForgeVersion,
    pub(crate) versions: Versions<V>,
    // last_updated: String,
}

#[derive(Deserialize)]
pub(crate) struct Versions<V> {
    pub(crate) version: Vec<V>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NeoForgeVersion {
    pub minecraft: String,
    pub neoforge: String,
}

impl LoaderInstall for NeoForge {
    type MetaInfo = NeoForgeVersion;
    type MetaList = std::vec::IntoIter<Self::MetaInfo>;
    async fn from_version(version: impl AsRef<str>, downloader: &Downloader<'_>) -> Result<Self> {
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
                    == version
                        .as_ref()
                        .strip_prefix("1.")
                        .unwrap_or(version.as_ref())
            })
            .collect();
        Ok(Self {
            group_id: xml.group_id,
            artifact_id: xml.artifact_id,
            list: filter,
        })
    }
    async fn install<C, S>(
        self,
        raw: &mut Instance<'_, JavaRuntime, C>,
        select: S,
        downloader: &Downloader<'_>,
    ) -> Result<()>
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
        let file = raw.storage.temp_path();
        let mut file = async_fs::File::create(file).await?;
        file.write_all(&body).await?;

        debug!("Build a virtual installation environment for NeoForge");

        Ok(())
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
