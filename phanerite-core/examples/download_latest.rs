use phanerite_core::*;
use std::num::NonZeroU8;
use tracing::Level;
fn main() -> error::Result<()> {
    nyquest_preset::register();
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    smol::block_on(async {
        let storage = storage::Storage::new(".minecraft".as_ref()).await?;
        let downloader = download::downloader::Downloader::builder(&storage)
            .concurrent(NonZeroU8::new(16).unwrap())
            .retries(3)
            .build()
            .await?;

        let index = download::vanilla::version_index::VersionIndex::sync(&downloader).await?;

        let latest = index.latest_release()?;

        let version =
            download::vanilla::version_info::VersionInfo::get(latest, &downloader).await?;

        let _ = async_fs::create_dir_all(storage.versions_dir.join(&latest.id)).await;
        let (task, _) = version
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
            .await
            .iter()
            .for_each(|x| println!("{x:?}"));

        Ok::<(), error::Error>(())
    })
}
