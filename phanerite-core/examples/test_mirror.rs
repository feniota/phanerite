use phanerite_core::download::Downloader;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::mirror::Granodiorite;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::instance::Instance;
use phanerite_core::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use tracing::Level;

fn main() -> error::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    smol::block_on(async {
        let storage = storage::Storage::new(".minecraft").await?;
        let downloader = download::downloader::RawDownloader::builder(&storage)
            .build()
            .await?;

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

        let manifest = version.get_manifest(&downloader).await?;

        let mirror = downloader.with_mirror(Granodiorite);
        let mut group = DownloadGroup::new(&mirror);
        let _g = monitor(&group).await;
        group.extend(
            Instance::create(manifest, Some(&version.id), &storage, &downloader)
                .await?
                .install(HashSet::new())
                .await?,
        );

        // 使用 Granodiorite 镜像下载
        let errs = group.exec().await;

        println!("Errors: {}", errs.len());
        for e in &errs {
            eprintln!("  Error: {}", e);
        }

        Ok::<(), error::Error>(())
    })
}

struct ExitGuard {
    exit: Arc<AtomicBool>,
}

impl Drop for ExitGuard {
    fn drop(&mut self) {
        self.exit.store(true, Relaxed)
    }
}

/// 显示下载速度和进度
async fn monitor(group: &DownloadGroup<'_, impl Downloader>) -> ExitGuard {
    let monitor = group.monitor();
    let g = ExitGuard {
        exit: Arc::new(AtomicBool::new(false)),
    };
    let exit = g.exit.clone();
    smol::spawn(async move {
        while !exit.load(Relaxed)  {
            let downloading = monitor.downloading().await;
            let number = monitor.len();
            let speed = monitor
                .speed_by_timer(smol::Timer::after(std::time::Duration::from_secs(1)))
                .await;
            let current = monitor.current().await;
            let total = monitor.total().await;
            let pct = if total > 0 {
                current as f64 / total as f64 * 100.0
            } else {
                0.0
            };
            let finished = monitor.finished().await;
            println!(
                "Progress: {pct:.1}% ({finished}/{number} finished) Downloading: {downloading}  {:.2} MiB/s",
                speed as f64 / 1024.0 / 1024.0,
            );
        }
    })
        .detach();
    g
}
