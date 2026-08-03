use crate::instance::manifest::{
    Arguments, AssetIndex, Downloads, InstanceManifest, JavaVersion, Library, Logging, VersionType,
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Deserialize a `DateTime<FixedOffset>` that may lack a timezone suffix
/// (some mod-loaders emit bare timestamps). Missing offset → UTC.
pub fn deser_datetime_opt<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<DateTime<FixedOffset>>, D::Error> {
    let s: Option<String> = Option::deserialize(d)?;
    match s {
        None => Ok(None),
        Some(s) => {
            // Try as-is first (chrono's default parser).
            if let Ok(dt) = s.parse::<DateTime<FixedOffset>>() {
                return Ok(Some(dt));
            }
            // Missing offset — treat as UTC.
            if let Ok(naive) = s.parse::<chrono::NaiveDateTime>() {
                return Ok(Some(naive.and_utc().fixed_offset()));
            }
            Err(serde::de::Error::custom(format!("invalid datetime: {s}")))
        }
    }
}

/// All version-manifest fields in their optional form.
/// Used as the `#[serde(flatten)]` target for `OverlayManifest` and `Patch`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_class: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub jar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_index: Option<AssetIndex>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub compliance_level: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub version_type: Option<VersionType>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deser_datetime_opt",
        default
    )]
    pub time: Option<DateTime<FixedOffset>>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deser_datetime_opt",
        default
    )]
    pub release_time: Option<DateTime<FixedOffset>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_launcher_version: Option<u32>,

    /// Old-version argument string (pre-1.13).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,

    /// Libraries — merged across overlays via `extend`, not replaced.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<Library>,

    /// Everything not covered above.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl OptionalManifest {
    /// Merge `other` on top of `self`.
    /// - `Option` fields: `other` wins when `Some`.
    /// - `Vec` fields: `extend` from `other`.
    pub fn merge(&mut self, other: &Self) {
        if other.arguments.is_some() {
            self.arguments.clone_from(&other.arguments);
        }
        if other.main_class.is_some() {
            self.main_class.clone_from(&other.main_class);
        }
        if other.jar.is_some() {
            self.jar.clone_from(&other.jar);
        }
        if other.asset_index.is_some() {
            self.asset_index.clone_from(&other.asset_index);
        }
        if other.assets.is_some() {
            self.assets.clone_from(&other.assets);
        }
        if other.compliance_level.is_some() {
            self.compliance_level = other.compliance_level;
        }
        if other.java_version.is_some() {
            self.java_version.clone_from(&other.java_version);
        }
        if other.downloads.is_some() {
            self.downloads.clone_from(&other.downloads);
        }
        if other.logging.is_some() {
            self.logging.clone_from(&other.logging);
        }
        if other.version_type.is_some() {
            self.version_type = other.version_type;
        }
        if other.time.is_some() {
            self.time = other.time;
        }
        if other.release_time.is_some() {
            self.release_time = other.release_time;
        }
        if other.minimum_launcher_version.is_some() {
            self.minimum_launcher_version = other.minimum_launcher_version;
        }
        if other.minecraft_arguments.is_some() {
            self.minecraft_arguments
                .clone_from(&other.minecraft_arguments);
        }
        self.libraries.extend(other.libraries.iter().cloned());
        for (k, v) in &other.extra {
            self.extra.insert(k.clone(), v.clone());
        }
    }
}

/// A version file that inherits from a base version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayManifest {
    pub inherits_from: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patches: Vec<Patch>,
    #[serde(flatten)]
    pub manifest: OptionalManifest,
}

impl OverlayManifest {
    /// Turn this overlay into a `Patch` with the given priority.
    pub fn into_patch(self, priority: i32) -> Patch {
        Patch {
            priority,
            manifest: self.manifest,
        }
    }
}

impl InstanceManifest {
    /// Merge an overlay as a patch with the given priority, then re-resolve.
    pub fn merge_overlay(&mut self, overlay: OverlayManifest, priority: i32) {
        self.patches.push(overlay.into_patch(priority));
        self.resolve();
    }

    /// Re-resolve all patches in priority order on top of the current fields.
    pub fn resolve(&mut self) {
        // Accumulative fields — clear contents before re-applying.
        if let Some(ref mut a) = self.arguments {
            a.game.clear();
            a.jvm.clear();
        }
        self.libraries.clear();

        let manifests: Vec<_> = self.patches.iter().map(|p| p.manifest.clone()).collect();
        for m in &manifests {
            self.apply_optional(m);
        }
    }

    fn apply_optional(&mut self, opt: &OptionalManifest) {
        if let Some(ref v) = opt.arguments {
            match self.arguments {
                Some(ref mut current) => {
                    current.game.extend(v.game.iter().cloned());
                    current.jvm.extend(v.jvm.iter().cloned());
                }
                None => self.arguments = Some(v.clone()),
            }
        }
        if let Some(ref v) = opt.main_class {
            self.main_class.clone_from(v);
        }
        if let Some(ref v) = opt.jar {
            self.jar.clone_from(v);
        }
        if let Some(ref v) = opt.asset_index {
            self.asset_index.clone_from(v);
        }
        if let Some(ref v) = opt.assets {
            self.assets.clone_from(v);
        }
        if let Some(ref v) = opt.java_version {
            self.java_version.clone_from(v);
        }
        if let Some(ref v) = opt.downloads {
            self.downloads.clone_from(v);
        }
        if let Some(ref v) = opt.logging {
            self.logging = Some(v.clone());
        }
        if let Some(v) = opt.version_type {
            self.version_type = v;
        }
        if let Some(v) = opt.time {
            self.time = v;
        }
        if let Some(v) = opt.release_time {
            self.release_time = v;
        }
        if let Some(v) = opt.minimum_launcher_version {
            self.minimum_launcher_version = Some(v);
        }
        if let Some(ref v) = opt.minecraft_arguments {
            self.minecraft_arguments = Some(v.clone());
        }
        self.libraries.extend(opt.libraries.iter().cloned());
        for (k, v) in &opt.extra {
            self.extra.insert(k.clone(), v.clone());
        }
    }
}

/// A patch entry within a version file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub priority: i32,
    #[serde(flatten)]
    pub manifest: OptionalManifest,
}
