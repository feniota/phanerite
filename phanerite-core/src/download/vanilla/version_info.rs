use crate::download::downloader::Downloader;
use crate::download::vanilla::version_index::{Version, VersionType};
use crate::error::Result;
use crate::instance::instance_info::{
    Arguments, DownloadInfo, InstanceManifest, JavaVersion, Logging, Patch,
};
use crate::instance::instance_info::{AssetIndex, Library};
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

impl Version {
    pub async fn get_manifest(&self, downloader: &Downloader) -> Result<VersionManifest> {
        let body = downloader
            .fetch(&self.url, Some(self.sha1.clone().into()))
            .await?;
        let json = serde_json::from_slice(&body)?;
        Ok(json)
    }
}

/// Build an instance manifest from the remote version metadata.
impl From<VersionManifest> for InstanceManifest {
    fn from(remote: VersionManifest) -> InstanceManifest {
        let assets = remote.asset_index.id.clone();
        let id = remote.id;
        let mut manifest = Self {
            id: id.clone(),
            arguments: remote.arguments,
            main_class: remote.main_class,
            jar: id,
            asset_index: remote.asset_index,
            assets,
            java_version: remote.java_version.unwrap_or(JavaVersion {
                component: "runtime-runtime-alpha".into(),
                major_version: 8,
            }),
            libraries: remote.libraries,
            downloads: crate::instance::instance_info::Downloads {
                client: remote.downloads.client.map(|d| DownloadInfo {
                    url: d.url,
                    sha1: d.sha1,
                    size: d.size,
                }),
                server: remote.downloads.server.map(|d| DownloadInfo {
                    url: d.url,
                    sha1: d.sha1,
                    size: d.size,
                }),
            },
            logging: remote.logging,
            version_type: remote.version_type,
            time: remote.time,
            release_time: remote.release_time,
            minimum_launcher_version: remote.minimum_launcher_version,
            root: Some(true),
            patches: vec![],
            minecraft_arguments: remote.minecraft_arguments,
            extra: filter_extra(remote.extra),
        };
        manifest.patches = manifest.arguments.clone().map_or(vec![], |args| {
            vec![Patch {
                id: "game".into(),
                version: manifest.id.clone(),
                priority: 0,
                arguments: args,
                main_class: manifest.main_class.clone(),
                asset_index: manifest.asset_index.clone(),
                assets: manifest.assets.clone(),
                java_version: manifest.java_version.clone(),
                libraries: manifest.libraries.clone(),
                downloads: manifest.downloads.clone(),
                logging: manifest.logging.clone(),
                version_type: manifest.version_type,
                time: manifest.time,
                release_time: manifest.release_time,
                minimum_launcher_version: manifest.minimum_launcher_version,
            }]
        });
        manifest
    }
}

/// Strip keys that `VersionManifest` already owns from the remote `extra` map,
/// so serde won't emit duplicate fields.
fn filter_extra(
    mut extra: HashMap<String, serde_json::Value>,
) -> HashMap<String, serde_json::Value> {
    extra.remove("assets");
    extra.remove("complianceLevel");
    extra.remove("id");
    extra.remove("type");
    extra.remove("time");
    extra.remove("releaseTime");
    extra.remove("minimumLauncherVersion");
    extra.remove("mainClass");
    extra.remove("arguments");
    extra.remove("assetIndex");
    extra.remove("javaVersion");
    extra.remove("libraries");
    extra.remove("downloads");
    extra.remove("logging");
    extra.remove("jar");
    extra.remove("patches");
    extra.remove("root");
    extra.remove("minecraftArguments");
    extra
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub id: String,

    #[serde(rename = "type")]
    pub version_type: VersionType,

    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,

    pub main_class: String,

    pub minimum_launcher_version: Option<u32>,

    pub arguments: Option<Arguments>,

    pub asset_index: AssetIndex,

    pub downloads: Downloads,

    pub java_version: Option<JavaVersion>,

    pub libraries: Vec<Library>,

    pub logging: Option<Logging>,

    // 旧版本字段
    pub minecraft_arguments: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// 游戏下载
#[derive(Clone, Deserialize, Serialize)]
pub struct Downloads {
    pub client: Option<Download>,
    pub server: Option<Download>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Download {
    pub sha1: Sha1Hash,
    pub size: u64,
    pub url: Url,
}
