use crate::download::downloader::Downloader;
use crate::download::vanilla::version_index::{Version, VersionType};
use crate::error::Result;
use crate::instance::instance_info::{
    Arguments, AssetIndex, Downloads, InstanceManifest, JavaVersion, Library, Logging,
};
use crate::instance::overlay::{OptionalManifest, Patch};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    fn from(remote: VersionManifest) -> Self {
        let assets = remote.asset_index.id.clone();
        let id = remote.id;

        // Self-patch so the serialized JSON records the original state.
        let self_patch = Patch {
            priority: 0,
            manifest: OptionalManifest {
                id: Some(id.clone()),
                version: None,
                arguments: remote.arguments.clone(),
                main_class: Some(remote.main_class.clone()),
                jar: Some(id.clone()),
                asset_index: Some(remote.asset_index.clone()),
                assets: Some(assets.clone()),
                compliance_level: None,
                java_version: remote.java_version.clone(),
                downloads: Some(remote.downloads.clone()),
                logging: remote.logging.clone(),
                version_type: Some(remote.version_type),
                time: Some(remote.time),
                release_time: Some(remote.release_time),
                minimum_launcher_version: remote.minimum_launcher_version,
                minecraft_arguments: remote.minecraft_arguments.clone(),
                libraries: remote.libraries.clone(),
                extra: HashMap::new(),
            },
        };

        Self {
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
            downloads: remote.downloads,
            logging: remote.logging,
            version_type: remote.version_type,
            time: remote.time,
            release_time: remote.release_time,
            minimum_launcher_version: remote.minimum_launcher_version,
            root: Some(true),
            patches: vec![self_patch],
            minecraft_arguments: remote.minecraft_arguments,
            extra: filter_extra(remote.extra),
        }
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
