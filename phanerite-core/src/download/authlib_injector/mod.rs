use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::error::{Error, Result};
use crate::storage::Storage;
use crate::utils::Sha256Hash;
use futures::StreamExt;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::LazyLock;
use url::Url;

static AUTHLIB_INJECTOR_LATEST_META: LazyLock<Url> = LazyLock::new(|| {
    "https://authlib-injector.yushi.moe/artifact/latest.json"
        .parse()
        .unwrap()
});

// #[derive(Deserialize)]
// struct Index {
//     latest_build_number: usize,
//     artifacts: Vec<Artifact>,
// }

#[derive(Deserialize)]
struct Artifact {
    build_number: usize,
    // version: String,
    download_url: Option<Url>,
    checksums: Option<Checksums>,
}

#[derive(Deserialize)]
struct Checksums {
    sha256: Sha256Hash,
}

pub struct AuthlibInjector<'a> {
    storage: &'a Storage,
}

impl<'a> AuthlibInjector<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }
    pub async fn update(&self, downloader: &Downloader) -> Result<Option<DownloadTask>> {
        match self.find_latest().await {
            Err(_) => Ok(Some(self.install_latest(downloader).await?)),
            Ok(v) => {
                let res = downloader
                    .fetch(&AUTHLIB_INJECTOR_LATEST_META, None)
                    .await?;
                let res = serde_json::from_slice::<Artifact>(&res)?;
                if res.build_number != v {
                    Ok(Some(self.install_latest(downloader).await?))
                } else {
                    Ok(None)
                }
            }
        }
    }
    pub async fn get(&self) -> Result<PathBuf> {
        Ok(self.storage.authlib_injector().join(format!(
            "authlib-injector-{}.jar",
            self.find_latest().await?
        )))
    }
    async fn find_latest(&self) -> Result<usize> {
        match async_fs::read_dir(self.storage.authlib_injector())
            .await?
            .filter_map(async |x| x.map(|t| t.file_name().to_string_lossy().into_owned()).ok())
            .filter_map(async |x| {
                x.strip_prefix("authlib-injector-")
                    .and_then(|t| t.strip_suffix(".jar"))
                    .and_then(|t| t.parse::<usize>().ok())
            })
            .fold(None, async |max, item| {
                Some(match max {
                    Some(m) if m > item => m,
                    _ => item,
                })
            })
            .await
        {
            None => Err(Error::other("No available authlib-injector")),
            Some(v) => Ok(v),
        }
    }
    async fn install_latest(&self, downloader: &Downloader) -> Result<DownloadTask> {
        let res = downloader
            .fetch(&AUTHLIB_INJECTOR_LATEST_META, None)
            .await?;
        let res = serde_json::from_slice::<Artifact>(&res)?;

        let Some(download) = res.download_url else {
            return Err(Error::other("No download url"));
        };
        let Some(hash) = res.checksums.map(|t| t.sha256) else {
            return Err(Error::other("No download hash"));
        };
        let file_name = format!("authlib-injector-{}.jar", res.build_number);

        let task = DownloadTask::builder()
            .url(download)
            .to_path(self.storage.authlib_injector().join(&file_name))
            .file_name(&file_name)
            .hash(hash)
            .build();

        Ok(task)
    }
}
