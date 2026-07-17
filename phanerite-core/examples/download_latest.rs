//! Full download pipeline: compio filesystem + ureq HTTP client.
//!
//! ```sh
//! cargo run --example download_latest --features compio,ureq
//! ```

use phanerite_core::download::Downloader;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::vanilla::version_info::VersionInfo;
use phanerite_core::io::adapters::compio::CompioFs;
use phanerite_core::io::adapters::ureq::UreqClient;
use phanerite_core::storage::Storage;
use phanerite_core::version::VersionsManager;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    compio::runtime::Runtime::new()?.block_on(async {
        let http_client = UreqClient::new();
        let root = Path::new("./mc");

        let storage = Storage::new(root, CompioFs).await?;
        let downloader = Downloader::new(storage, http_client);

        let index = VersionIndex::fetch(&downloader.http_client).await?;
        let version = index.latest_release();
        let version_info = VersionInfo::fetch(version, &downloader.http_client).await?;

        let mgr_storage = Storage::new(root, CompioFs).await?;
        let name = version_info.id.clone();
        let manager = VersionsManager::new(mgr_storage, downloader);
        manager.creat_version(&name, version_info).await?;

        Ok::<_, phanerite_core::error::Error>(())
    })?;
    Ok(())
}
