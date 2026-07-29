use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::mirror::granodiorite::Granodiorite;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::vanilla::version_info::VersionManifest;
use phanerite_core::instance::Instance;
use phanerite_core::storage::ShareStrategy::Force;
use phanerite_core::*;
use std::time::Instant;
use tracing::Level;

fn main() -> error::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    smol::block_on(async {
        let storage = storage::Storage::new(".minecraft")?.share_strategy(Force);
        let downloader = download::downloader::Downloader::builder(&storage)
            .build()
            .await?;

        let mut group = DownloadGroup::new();

        // 下载最新正式版
        let version_id = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "1.20.1".to_string());

        let index = VersionIndex::sync(&downloader).await?;
        let version = index
            .iter()
            .find(|v| v.id == version_id)
            .expect("Version not found");
        println!("Downloading: {} ({})", version.id, version.version_type);

        let manifest = VersionManifest::get(version, &downloader).await?;
        let name = version.id.clone();

        group.extend(Instance::create(manifest, &name, &storage, &downloader).await?);

        let processes = std::sync::Arc::new(group.processes());
        let total = processes.total() as f64 / 1024.0 / 1024.0;
        println!("Total size: {:.2} MiB ({} tasks)", total, processes.len());

        // 显示下载速度和进度
        let monitor = processes.clone();
        smol::spawn(async move {
            while !monitor.is_finished() {
                let downloading = monitor.downloading();
                let speed = monitor
                    .speed_by_timer(smol::Timer::after(std::time::Duration::from_secs(1)))
                    .await;
                let current = monitor.current();
                let pct = if monitor.total() > 0 {
                    current as f64 / monitor.total() as f64 * 100.0
                } else {
                    0.0
                };
                let finished = monitor.finished();
                println!(
                    "Progress: {pct:.1}% ({finished} finished) Downloading: {downloading}  {:.2} MiB/s",
                    speed as f64 / 1024.0 / 1024.0,
                );
            }
        })
        .detach();

        // 使用 Granodiorite 镜像下载
        let instant = Instant::now();
        let errs = group.exec_with_mirror(&downloader, Granodiorite).await;
        let spend = instant.elapsed().as_secs_f64();

        println!("Errors: {}", errs.len());
        for e in &errs {
            eprintln!("  Error: {}", e);
        }

        println!(
            "Done! Download {:.2} MB in {:.2} s, avg:{:.2} MB/s",
            total,
            spend,
            total / spend
        );
        Ok::<(), error::Error>(())
    })
}
