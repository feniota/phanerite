use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use phanerite_core::*;
use std::num::NonZeroU8;
use std::sync::Arc;
use tracing::Level;
fn main() -> error::Result<()> {
    nyquest_preset::register();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    smol::block_on(async {
        let storage = storage::Storage::new(".minecraft".as_ref()).await?;
        let downloader = download::downloader::Downloader::builder(&storage)
            .concurrent(NonZeroU8::new(16).unwrap())
            .retries(3)
            .build()
            .await?;

        smol::spawn(async move {}).detach();

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

        let mp = Arc::new(MultiProgress::new());

        let tasks = task.collect::<Vec<_>>();

        for task in &tasks {
            let process = task.process.clone();
            let mp = mp.clone();

            smol::spawn(async move {
                // 等待开始
                while !process.is_started() {
                    process.changed().await;
                }

                let pb = mp.add(ProgressBar::new(process.total().unwrap_or_default()));

                pb.set_style(
                    ProgressStyle::with_template(
                        "{msg:20} {bar:40.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec}",
                    )
                    .unwrap(),
                );

                pb.set_message(process.name().unwrap_or("unknown").to_string());

                loop {
                    process.changed().await;

                    if let Some(total) = process.total() {
                        pb.set_length(total);
                    }

                    pb.set_position(process.current());

                    if process.is_finished() {
                        pb.finish_and_clear();
                        break;
                    }
                }
            })
            .detach();
        }

        downloader
            .download_concurrent(download::task::filter_existed(tasks.into_iter()))
            .await
            .iter()
            .for_each(|x| println!("{x:?}"));

        Ok::<(), error::Error>(())
    })
}
