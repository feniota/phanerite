use crate::error::{Error, Result};
use crate::io::utils::AsyncFileExt;
use crate::io::{HttpClient, HttpRequest, Method};
use crate::utils::Sha1;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::slice::Iter;
use std::vec::IntoIter;
use tracing::{debug, instrument};

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
    pub sha1: Sha1,
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

impl VersionIndex {
    #[instrument(skip(http_client))]
    pub async fn fetch(http_client: &impl HttpClient) -> Result<Self> {
        debug!("fetching version index");
        let request = HttpRequest {
            method: Method::Get,
            url: VERSION_INDEX_URL,
            headers: Default::default(),
            body: None,
        };

        let response = http_client.execute(request).await?;

        response.ok()?;

        let body = response.body.read_all().await?;
        let json: Self = serde_json::from_slice(&body).map_err(|e| Error::Other(e.to_string()))?;
        debug!("fetched {} versions", json.versions.len());
        Ok(json)
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

#[cfg(test)]
mod test {
    #[cfg(feature = "reqwest")]
    #[test]
    fn test_fetch() {
        use super::*;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let fs = crate::io::adapters::tokio::TokioFs;
                let http_client = crate::io::adapters::reqwest::ReqwestClient::new();
                let res = VersionIndex::fetch(&http_client).await;
                assert!(res.is_ok());
            });
    }
}
