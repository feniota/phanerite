use crate::error::{Error, Result};
use crate::io::utils::AsyncFileExt;
use crate::io::{HttpClient, HttpRequest, Method};
use serde::Deserialize;
use std::slice::Iter;
use std::vec::IntoIter;
use time::OffsetDateTime;

const VERSION_INDEX_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Deserialize)]
pub struct VersionIndex {
    latest: Latest,
    versions: Vec<Version>,
}

#[derive(Deserialize)]
struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: OffsetDateTime,
    pub release_time: OffsetDateTime,
    pub sha1: String,
    pub compliance_level: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

impl VersionIndex {
    pub async fn fetch(http_client: &impl HttpClient) -> Result<Self> {
        let request = HttpRequest {
            method: Method::Get,
            url: VERSION_INDEX_URL,
            headers: Default::default(),
            body: None,
        };

        let response = http_client.execute(request).await?;

        if response.status < 200 || response.status >= 300 {
            return Err(Error::Http(response.status));
        }

        let body = response.body.read_all().await?;
        Ok(serde_json::from_slice(&body).map_err(|e| Error::Other(e.to_string()))?)
    }
    pub fn iter(&self) -> Iter<'_, Version> {
        self.versions.iter()
    }
    pub fn into_iter(self) -> IntoIter<Version> {
        self.versions.into_iter()
    }
    pub fn latest_release(&self) -> &Version {
        self.iter()
            .find(|&x| x.id == self.latest.release)
            .expect("Format error: There is no latest version listed in the version index.")
    }
    pub fn latest_snapshot(&self) -> &Version {
        self.iter()
            .find(|&x| x.id == self.latest.snapshot)
            .expect("Format error: There is no latest version listed in the version index.")
    }
}
