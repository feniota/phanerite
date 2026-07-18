//! High-level version manager.
//!
//! [`VersionsManager`] orchestrates the full install flow for a
//! Minecraft version: write the version JSON, download the client
//! JAR, fetch assets, and schedule library downloads.

use crate::download::vanilla::assets::AssetIndexList;
use crate::download::vanilla::version_info::VersionInfo;
use crate::download::{ConcurrentTask, Downloader};
use crate::error::Result;
use crate::io::utils::AsyncFileExt;
use crate::io::{Error, FileSystem, HttpClient};
use crate::storage::Storage;
use std::num::NonZeroU16;
use std::path::PathBuf;
use tracing::{info, instrument};

pub struct VersionsManager<F: FileSystem, H: HttpClient> {
    storage: Storage<F>,
    downloader: Downloader<F, H>,
}

impl<F: FileSystem, H: HttpClient> VersionsManager<F, H> {
    pub fn new(storage: Storage<F>, downloader: Downloader<F, H>) -> Self {
        Self {
            storage,
            downloader,
        }
    }
    #[instrument(skip(self, version), fields(version = name))]
    pub async fn creat_version(&self, name: &str, version: VersionInfo) -> Result<PathBuf> {
        info!("creating version");

        let version_path = self.storage.versions_dir.join(name);
        self.storage.fs.create_dir_all(&version_path).await?;
        let version_file = self
            .storage
            .fs
            .create(&version_path.join(format!("{}.json", name)))
            .await?;
        version_file
            .write_all_at(
                0,
                serde_json::to_vec(&version).map_err(|e| Error::Other(e.to_string()))?,
            )
            .await?;

        let assets_path = self.storage.assets_dir.join("indexes");
        self.storage.fs.create_dir_all(&assets_path).await?;
        let assets_index =
            AssetIndexList::fetch(&version.asset_index, &self.downloader.http_client).await?;
        let assets_index_file = self
            .storage
            .fs
            .create(&assets_path.join(format!("{}.json", version.asset_index.id)))
            .await?;
        assets_index_file
            .write_all_at(
                0,
                serde_json::to_vec(&assets_index).map_err(|e| Error::Other(e.to_string()))?,
            )
            .await?;

        let mut tasks = ConcurrentTask::new(NonZeroU16::new(1).unwrap());

        tasks.push(
            self.downloader.download_to_bucket(
                version.get_client(version_path.join(format!("{}.jar", name)))?,
            ),
        );
        for i in assets_index.iter_downloadable() {
            tasks.push(self.downloader.download_to_path(i))
        }
        for i in version.libraries {
            tasks.push(self.downloader.download_to_path(i))
        }

        tasks.exec().await?;

        Ok(version_path)
    }
}
