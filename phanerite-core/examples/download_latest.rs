//! Full download pipeline.
//!
//! ```sh
//! # tokio + reqwest
//! cargo run --example download_latest --features tokio,reqwest
//!
//! # compio + ureq
//! cargo run --example download_latest --features compio,ureq
//! ```

use phanerite_core::download::Downloader;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::vanilla::version_info::VersionInfo;
use phanerite_core::storage::Storage;
use phanerite_core::version::VersionsManager;
use std::path::Path;
use tracing::Level;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    // ── filesystem ──────────────────────────────────────────────

    #[cfg(feature = "compio")]
    let fs = {
        use phanerite_core::io::adapters::compio::CompioFs;
        CompioFs
    };
    #[cfg(feature = "tokio")]
    let fs = {
        use phanerite_core::io::adapters::tokio::TokioFs;
        TokioFs
    };

    // ── HTTP client ─────────────────────────────────────────────

    #[cfg(feature = "nyquest")]
    nyquest_preset::register();

    #[cfg(feature = "ureq")]
    let http_client = {
        use phanerite_core::io::adapters::ureq::UreqClient;
        UreqClient::new()
    };
    #[cfg(feature = "reqwest")]
    let http_client = {
        use phanerite_core::io::adapters::reqwest::ReqwestClient;
        ReqwestClient::new()
    };
    #[cfg(feature = "nyquest")]
    let http_client = {
        use phanerite_core::io::adapters::nyquest::NyquestClient;
        let inner = nyquest::ClientBuilder::default()
            .max_response_buffer_size(200 * 1024 * 1024) // 200 MB
            .build_async()
            .await?;
        NyquestClient::new(inner)
    };

    // ── pipeline ────────────────────────────────────────────────

    let root = Path::new("./mc");
    let storage = Storage::new(root, fs).await?;
    let downloader = Downloader::new(3, storage, http_client);

    let index = VersionIndex::fetch(&downloader.http_client).await?;
    let version = index.latest_release();
    let version_info = VersionInfo::fetch(version, &downloader.http_client).await?;

    let mgr_storage = {
        #[cfg(feature = "compio")]
        let fs = {
            use phanerite_core::io::adapters::compio::CompioFs;
            CompioFs
        };
        #[cfg(feature = "tokio")]
        let fs = {
            use phanerite_core::io::adapters::tokio::TokioFs;
            TokioFs
        };
        Storage::new(root, fs).await?
    };
    let name = version_info.id.clone();
    let manager = VersionsManager::new(mgr_storage, downloader);
    manager.creat_version(&name, version_info).await?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════

#[cfg(feature = "compio")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    compio::runtime::Runtime::new()?.block_on(run())
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}
