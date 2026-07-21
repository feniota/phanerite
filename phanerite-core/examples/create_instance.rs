use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::vanilla::version_info::VersionInfo;
use phanerite_core::instance::Instance;
use phanerite_core::*;
use std::num::NonZeroU8;
use tracing::Level;

fn main() -> error::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
        let storage = storage::Storage::new(".minecraft").await?;
        let downloader = download::downloader::Downloader::builder(&storage)
            .concurrent(NonZeroU8::new(16).unwrap())
            .retries(3)
            .build()
            .await?;

        Instance::create(
            VersionInfo::get(
                VersionIndex::sync(&downloader).await?.latest_release()?,
                &downloader,
            )
            .await?,
            "26.2",
            &storage,
            &downloader,
        )
        .await?;

        Ok::<(), error::Error>(())
    })
}
