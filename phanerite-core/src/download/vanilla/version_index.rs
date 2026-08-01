use crate::download::downloader::Downloader;
use crate::error::{Error, Result};
pub(crate) use crate::instance::manifest::VersionType;
use crate::utils::Sha1Hash;
use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use std::slice::Iter;
use std::sync::LazyLock;
use url::Url;

static VERSION_INDEX_URL: LazyLock<Url> = LazyLock::new(|| {
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json"
        .parse()
        .unwrap()
});

impl VersionIndex {
    pub async fn sync(downloader: &Downloader) -> Result<Self> {
        let body = downloader.fetch(&VERSION_INDEX_URL, None).await?;
        let json = serde_json::from_slice(&body)?;
        Ok(json)
    }
    pub fn iter(&self) -> Iter<'_, Version> {
        self.versions.iter()
    }
    pub fn latest_release(&self) -> Result<&Version> {
        match self.iter().find(|&x| x.id == self.latest.release) {
            None => Err(Error::other("Error instance index format")),
            Some(v) => Ok(v),
        }
    }
    pub fn latest_snapshot(&self) -> Result<&Version> {
        match self.iter().find(|&x| x.id == self.latest.snapshot) {
            None => Err(Error::other("Error instance index format")),
            Some(v) => Ok(v),
        }
    }
}

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
    pub url: Url,
    pub time: DateTime<FixedOffset>,
    pub release_time: DateTime<FixedOffset>,
    pub sha1: Sha1Hash,
    pub compliance_level: usize,
}
