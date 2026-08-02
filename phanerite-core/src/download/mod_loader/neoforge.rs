use crate::download::downloader::Downloader;
use crate::download::mod_loader::{LoaderInstall, LoaderMeta};
use crate::download::vanilla::version_index::Version;
use crate::error::Result;
use crate::instance::manifest::InstanceManifest;
use serde::Deserialize;
use std::cmp::Ordering;
use std::sync::LazyLock;
use url::Url;

static NEOFORGE_META: LazyLock<Url> = LazyLock::new(|| {
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml"
        .parse()
        .unwrap()
});

// static FORGE_META: LazyLock<Url> = LazyLock::new(|| {
//     "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml"
//         .parse()
//         .unwrap()
// });

pub struct Forge {
    list: Vec<ForgeVersion>,
}

impl LoaderInstall for Forge {
    type MetaInfo = ForgeVersion;
    type MetaList = std::vec::IntoIter<Self::MetaInfo>;
    async fn from_version(version: &Version, downloader: &Downloader) -> Result<Self> {
        let body = downloader.fetch(&NEOFORGE_META, None).await?;
        let reader = std::io::Cursor::new(body);
        let xml = serde_xml_rs::from_reader::<MetaData, _>(reader)?;
        let filter = xml
            .versioning
            .versions
            .into_iter()
            .filter(|x| x.minecraft == version.id)
            .collect();
        Ok(Self { list: filter })
    }
    async fn install<S>(
        self,
        raw: InstanceManifest,
        select: S,
        downloader: &Downloader,
    ) -> Result<InstanceManifest>
    where
        S: AsyncFnOnce(Self::MetaList) -> Result<Self::MetaInfo>,
    {
        let selected = select(self.list.into_iter()).await?;

        todo!()
    }
}

impl LoaderMeta for ForgeVersion {
    fn name(&self) -> &str {
        "neoforge"
    }

    fn version(&self) -> &str {
        &self.forge
    }

    fn stable(&self) -> bool {
        true
    }
}

impl Eq for ForgeVersion {}

impl PartialEq<Self> for ForgeVersion {
    fn eq(&self, other: &Self) -> bool {
        self.forge.eq(&other.forge)
    }
}

impl PartialOrd<Self> for ForgeVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ForgeVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        // TODO: 更好的版本比较
        self.forge.cmp(&other.forge)
    }
}

#[derive(Deserialize)]
struct MetaData {
    // group_id: String,
    // artifact_id: String,
    versioning: Versioning,
}

#[derive(Deserialize)]
struct Versioning {
    // latest: ForgeVersion,
    // release: ForgeVersion,
    versions: Vec<ForgeVersion>,
    // last_updated: String,
}

pub struct ForgeVersion {
    minecraft: String,
    forge: String,
}

impl<'de> Deserialize<'de> for ForgeVersion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (minecraft, forge) = s
            .split_once('-')
            .ok_or_else(|| serde::de::Error::custom("invalid NeoForge version"))?;
        Ok(Self {
            minecraft: minecraft.to_owned(),
            forge: forge.to_owned(),
        })
    }
}
