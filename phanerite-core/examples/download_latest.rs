use phanerite_core::*;
use tracing::Level;
fn main() -> error::Result<()> {
    nyquest_preset::register();
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    smol::block_on(async {
        let storage = storage::Storage::new(".minecraft".as_ref()).await?;
        let downloader = download::downloader::Downloader::new(&storage)
            .await?
            .retries(10);

        let index = download::vanilla::version_index::VersionIndex::sync(&downloader).await?;

        let latest = index.latest_release()?;

        let version =
            download::vanilla::version_info::VersionInfo::get(latest, &downloader).await?;

        let _ = async_fs::create_dir_all(storage.versions_dir.join(&latest.id)).await;
        let task = version
            .build_all_task(
                storage
                    .versions_dir
                    .join(&latest.id)
                    .join(format!("{}.jar", latest.id)),
                &storage,
                &downloader,
            )
            .await?;

        downloader
            .download_concurrent(download::task::filter_existed(task))
            .await?;

        Ok::<(), error::Error>(())
    })
}
