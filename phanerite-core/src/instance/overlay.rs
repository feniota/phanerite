use crate::error::Result;
use crate::instance::Instance;
use crate::instance::instance_info::{
    Arguments, AssetIndex, Downloads, InstanceManifest, JavaVersion, Library, Logging, Patch,
    VersionType,
};
use crate::storage::Storage;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 此处顺序不可改变
/// 若 `inherits_from` 存在，则允许其他字段 `Option`
/// 否则 `InstanceManifest` 的字段必须存在
#[derive(Deserialize)]
#[serde(untagged)]
pub enum RawManifest {
    Overlay(InheritsManifest),
    Base(InstanceManifest),
}

impl RawManifest {
    #[async_recursion::async_recursion] // 需要递归
    pub async fn merge(
        self,
        storage: &Storage,
        visiting: HashSet<&str>,
    ) -> Result<InstanceManifest> {
        let overlay = match self {
            RawManifest::Overlay(v) => v,
            RawManifest::Base(v) => return Ok(v),
        };
        let base = Instance::open_inner(&overlay.inherits_from, storage, visiting).await?;
        Ok(overlay.merge_from(base.manifest))
    }
}

impl InheritsManifest {
    /// 更改继承对象
    pub fn inherits(mut self, id: impl Into<String>) -> Self {
        self.inherits_from = id.into();
        self
    }
    /// 合并到完整清单
    fn merge_from(mut self, base: InstanceManifest) -> InstanceManifest {
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
pub struct InheritsManifest {
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
