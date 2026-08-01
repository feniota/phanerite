use crate::download::vanilla::maven::MavenArtifact;
use crate::error::Result;
use crate::instance::overlay::Patch;
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use futures::AsyncReadExt;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use strum::{Display, EnumString};
use url::Url;

/// Top-level Minecraft version manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceManifest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
    pub main_class: String,
    pub jar: String,
    pub asset_index: AssetIndex,
    pub assets: String,
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub downloads: Downloads,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_launcher_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<bool>,
    #[serde(default)]
    pub patches: Vec<Patch>,
    /// 旧版本 minecraftArguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// JVM / game launch arguments split into two arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

/// A single argument entry: either a plain string or a conditional block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Conditional(ConditionalArgument),
}

/// A group of arguments guarded by OS / feature rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalArgument {
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(deserialize_with = "deser_value_string_or_vec")]
    pub value: Vec<String>,
}

/// A single rule: OS-based or feature-based allow/deny.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<OsInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
}

impl Rule {
    /// Evaluate this rule against the current OS and a set of enabled features.
    ///
    /// Returns `Some(action)` if the rule's OS **and** feature conditions
    /// are all met, or `None` if this rule does not apply.
    pub fn evaluate(&self, features: &HashSet<&'static str>) -> Option<Action> {
        if let Some(ref os) = self.os
            && !os.matches_current()
        {
            return None;
        }
        if let Some(ref feats) = self.features {
            for (k, v) in feats {
                if features.contains(k.as_str()) != *v {
                    return None;
                }
            }
        }
        Some(self.action)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Allow,
    Disallow,
}

impl Action {
    pub fn allow(&self) -> bool {
        match self {
            Action::Allow => true,
            Action::Disallow => false,
        }
    }
}

/// OS constraint (name + optional architecture).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OsInfo {
    #[serde(default)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

impl OsInfo {
    /// Check whether the current system is included by this OS constraint.
    ///
    /// - `name` empty → matches any OS.
    /// - `name` "osx" matches macOS (`std::env::consts::OS == "macos"`).
    /// - `arch` `None` → matches any architecture.
    pub fn matches_current(&self) -> bool {
        if !self.name.is_empty() {
            let current_os = std::env::consts::OS;
            let mapped = match current_os {
                "macos" => "osx",
                other => other,
            };
            if self.name != mapped {
                return false;
            }
        }
        if let Some(ref arch) = self.arch
            && arch != std::env::consts::ARCH
        {
            return false;
        }
        true
    }
}

/// Asset index metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub total_size: u64,
    pub id: String,
    pub url: Url,
    pub sha1: Sha1Hash,
    pub size: u64,
}

/// Java version requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

/// A single library dependency.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Library {
    pub name: MavenArtifact,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibraryDownloads>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract: Option<Extract>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, Artifact>>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LibraryDownloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Artifact>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,

    pub sha1: Sha1Hash,

    pub size: u64,

    pub url: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Extract {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

/// A download without a path field (used for client / server jars).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadInfo {
    pub url: Url,
    pub sha1: Sha1Hash,
    pub size: u64,
}

/// Client / server download URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Downloads {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<DownloadInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<DownloadInfo>,
}

/// Client-side logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    pub client: LoggingClient,
}

fn default_logging_type() -> String {
    "log4j2-xml".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingClient {
    pub file: LoggingFileInfo,
    pub argument: String,
    #[serde(rename = "type", default = "default_logging_type")]
    pub type_: String,
}

/// Logging configuration file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingFileInfo {
    pub id: String,
    pub url: Url,
    pub sha1: Sha1Hash,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, Display, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    #[strum(to_string = "release")]
    Release,
    #[strum(to_string = "snapshot")]
    Snapshot,
    #[strum(to_string = "old_beta")]
    OldBeta,
    #[strum(to_string = "old_alpha")]
    OldAlpha,
}

impl InstanceManifest {
    pub async fn form_local(path: impl AsRef<Path>) -> Result<Self> {
        let mut buf = vec![];
        async_fs::File::open(path)
            .await?
            .read_to_end(&mut buf)
            .await?;
        Ok(serde_json::from_slice(&buf)?)
    }

    /// Override `id` and `jar` with the given instance name.
    pub fn rename(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.id = name.clone();
        self.jar = name;
        self
    }
}

// ── Custom deserializer: accept both `"str"` and `["a","b"]` ──────

fn deser_value_string_or_vec<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    struct StringOrVec;
    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or a sequence of strings")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<Self::Value, E> {
            Ok(vec![v.to_owned()])
        }

        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                v.push(s);
            }
            Ok(v)
        }
    }
    d.deserialize_any(StringOrVec)
}
