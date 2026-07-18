use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

const VERSION_INDEX_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Deserialize, Debug)]
pub struct VersionIndex {
    latest: Latest,
    versions: Vec<Version>,
}

#[derive(Deserialize, Debug)]
struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,
    pub sha1: Sha1Hash,
    pub compliance_level: usize,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}
