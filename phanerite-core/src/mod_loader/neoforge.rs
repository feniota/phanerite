use crate::download::Downloader;
use crate::error::{Error, Result};
use crate::instance::Instance;
use crate::instance::overlay::OverlayManifest;
use crate::mod_loader::{LoaderInstall, LoaderMeta};
use crate::runtime::java::JavaRuntime;
use crate::utils::maven::MavenArtifact;
use crate::utils::version::{compare_versions, is_stable};
use futures::{AsyncWriteExt, StreamExt};
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::io::Read;
use std::path::Path;
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
    async fn from_version(version: impl AsRef<str>, downloader: &impl Downloader) -> Result<Self> {
        let body = downloader.fetch(NEOFORGE_META.clone(), None).await?;
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
        downloader: &impl Downloader,
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
        let body = downloader.fetch(url, None).await?;
        let reader = std::io::Cursor::new(&body);
        let mut archive = zip::ZipArchive::new(reader)?;
        let mut manifest = Vec::new();
        archive
            .by_name("version.json")?
            .read_to_end(&mut manifest)?;
        drop(archive);
        let manifest = serde_json::from_slice::<OverlayManifest>(&manifest)?;
        let installer = raw.storage.temp_path();
        let mut file = async_fs::File::create(&installer).await?;
        file.write_all(&body).await?;
        drop(file);

        debug!("Build a virtual installation environment for NeoForge");
        let temp = raw.storage.temp_path();
        async_fs::create_dir_all(&temp).await?;
        // 假 launcher_profiles.json 骗 Installer 安装
        let mut profile = async_fs::File::create(&temp.join("launcher_profiles.json")).await?;
        profile.write_all(b"{\"profiles\":{}}").await?;
        drop(profile);
        // 运行安装器
        async_process::Command::new(&raw.runtime)
            .arg("-jar")
            .arg(installer)
            .arg("--installClient")
            .arg(&temp)
            .status()
            .await?
            .success()
            .ok_or(Error::other("The NeoForge installer exits on failure"))?;
        // 拿走安装结果
        merge_move(&temp.join("libraries"), raw.storage.libraries_dir()).await?;
        // 合并版本配置
        raw.manifest.merge_overlay(manifest, 30000);

        let _ = async_fs::remove_dir_all(temp).await;
        Ok(())
    }
}

/// 移动整个目录，并跳过已有文件
#[async_recursion::async_recursion]
async fn merge_move(src: &Path, dst: &Path) -> Result<()> {
    let mut entries = async_fs::read_dir(src).await?;
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let ty = entry.file_type().await?;
        if ty.is_dir() {
            if dst_path.exists() {
                merge_move(&src_path, &dst_path).await?;
            } else {
                async_fs::rename(&src_path, &dst_path).await?;
            }
        } else {
            if dst_path.exists() {
                continue;
            }
            async_fs::rename(&src_path, &dst_path).await?;
        }
    }
    Ok(())
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
