use crate::instance::instance_info::{
    Arguments, AssetIndex, Downloads, InstanceManifest, JavaVersion, Library, Logging, Patch,
    VersionType,
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl OverlayManifest {
    /// 合并到完整清单
    pub fn merge(mut self, base: InstanceManifest) -> InstanceManifest {
        InstanceManifest {
            id: self.id,
            arguments: match base.arguments {
                None => self.arguments,
                Some(mut a) => match self.arguments {
                    None => Some(a),
                    Some(i) => {
                        a.game.extend(i.game);
                        a.jvm.extend(i.jvm);
                        Some(a)
                    }
                },
            },
            main_class: self.main_class.unwrap_or(base.main_class),
            jar: self.jar.unwrap_or(base.jar),
            asset_index: self.asset_index.unwrap_or(base.asset_index),
            assets: self.assets.unwrap_or(base.assets),
            compliance_level: self.compliance_level.unwrap_or(base.compliance_level),
            java_version: self.java_version.unwrap_or(base.java_version),
            libraries: {
                self.libraries.extend(base.libraries);
                self.libraries
            },
            downloads: self.downloads.unwrap_or(base.downloads),
            logging: self.logging.or(base.logging),
            version_type: self.version_type.unwrap_or(base.version_type),
            time: self.time.unwrap_or(base.time),
            release_time: self.release_time.unwrap_or(base.release_time),
            minimum_launcher_version: self
                .minimum_launcher_version
                .or(base.minimum_launcher_version),
            root: base.root,
            patches: {
                self.patches.extend(base.patches);
                self.patches
            },
            minecraft_arguments: self.minecraft_arguments.or(base.minecraft_arguments),
            extra: {
                self.extra.extend(base.extra);
                self.extra
            },
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayManifest {
    pub id: String,

    pub inherits_from: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,

    pub main_class: Option<String>,

    pub jar: Option<String>,

    pub asset_index: Option<AssetIndex>,

    pub assets: Option<String>,

    pub compliance_level: Option<u32>,

    pub java_version: Option<JavaVersion>,

    #[serde(default)]
    pub libraries: Vec<Library>,

    pub downloads: Option<Downloads>,

    pub logging: Option<Logging>,

    #[serde(rename = "type")]
    pub version_type: Option<VersionType>,

    pub time: Option<DateTime<FixedOffset>>,

    pub release_time: Option<DateTime<FixedOffset>>,

    pub minimum_launcher_version: Option<u32>,

    #[serde(default)]
    pub patches: Vec<Patch>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
