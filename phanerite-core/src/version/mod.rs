use crate::download::Downloader;
use crate::download::vanilla::assets::AssetIndexList;
use crate::download::vanilla::version_info::VersionInfo;
use crate::error::Result;
use crate::io::utils::AsyncFileExt;
use crate::io::{Error, FileSystem, HttpClient};
use crate::storage::Storage;
use std::path::PathBuf;

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
    pub async fn creat_version(&self, name: &str, version: VersionInfo) -> Result<PathBuf> {
        let version_path = self.storage.versions_dir.join(name);
        self.storage.fs.create_dir_all(&version_path).await?;
        let cfg = self
            .storage
            .fs
            .create(&version_path.join(format!("{}.json", name)))
            .await?;
        cfg.write_all_at(
            0,
            serde_json::to_vec(&version).map_err(|e| Error::Other(e.to_string()))?,
        )
        .await?;

        // 以下代码需要并发优化
        self.downloader
            .download_to_bucket(version.get_client(version_path.join(format!("{}.jar", name)))?)
            .await?;
        for i in AssetIndexList::fetch(&version.asset_index, &self.downloader.http_client)
            .await?
            .iter_downloadable()
        {
            self.downloader.download_to_path(i).await?;
        }
        for i in version.libraries {
            self.downloader.download_to_path(i).await?;
        }
        Ok(version_path)
    }
}
