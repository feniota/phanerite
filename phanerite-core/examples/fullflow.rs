use async_executor::Executor;
use phanerite_core::auth::yggdrasil::Authentication;
use phanerite_core::download::authlib_injector::AuthlibInjector;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::java::zulu::Zulu;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::{Downloader, DownloaderExt};
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::instance::manifest::InstanceManifest;
use phanerite_core::runtime::java::install_java;
use phanerite_core::storage::SharePreference::Hardlink;
use phanerite_core::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::{Duration, Instant};
use tracing::{Level, error};
use url::Url;

fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    let executor = Executor::new();
    let _guard = BlockingGuard::new(&executor);
    if let Err(e) = smol::block_on(executor.run(async {
        let storage = storage::Storage::new(".minecraft")
            .await?
            .share_preference(Hardlink);
        let (cleaner, _shutdown) = storage.run_cleaner();
        smol::spawn(cleaner).detach();
        let raw_downloader = download::downloader::RawDownloader::builder(&storage)
            .build()
            .await?;
        let cached_downloader = raw_downloader.with_cache_default();
        let mut downloader = cached_downloader.with_group();
        let _g = monitor(&downloader).await;

        let injector = AuthlibInjector::new(&storage);

        let _ = async_fs::remove_dir_all(storage.versions_dir().join("latest")).await;

        let version: InstanceManifest = VersionIndex::sync(&downloader)
            .await?
            // .iter()
            // .find(|x| x.id == "1.21.1")
            // .expect("Version not found")
            .latest_release()?
            .get_manifest(&downloader)
            .await?
            .into();

        downloader.extend(install_java::<Zulu>(version.java_major(), &storage, &downloader).await?);
        downloader.extend(injector.update(&downloader).await?);
        downloader.exec().await.iter().for_each(|e| error!("{e}"));

        let instance = Instance::create(version, Some("latest"), &storage, &downloader).await?;

        let java = instance
            .find_java(&storage)
            .await
            .into_iter()
            .next()
            .ok_or(Error::other("No available java"))?;
        let instance = instance.bind_java(java.clone()).await?;

        // instance
        //     .install_loader::<NeoForge>("1.21.1", &downloader, async |iter| {
        //         // let iter = iter
        //         //     .inspect(|x| println!("{}:{} stable:{}", x.name(), x.version(), x.stable()));
        //         let latest = iter
        //             .collect::<BTreeSet<_>>()
        //             .pop_last()
        //             .expect("No available loader version");
        //         println!(
        //             "{}:{} stable:{}",
        //             latest.name(),
        //             latest.version(),
        //             latest.stable()
        //         );
        //         Ok(latest)
        //     })
        //     .await?;

        downloader
            .join(instance.install_less(HashSet::new()).await?)
            .await
            .iter()
            .for_each(|e| error!("{e}"));

        let auth = Authentication::new_login(&downloader)
            .inject(&injector)
            .custom("https://aphanite.enita.cn/api/yggdrasil".parse::<Url>()?)
            .await?
            .username(
                std::env::var("USERNAME")
                    .expect("Fill in the login credentials in the environment variable"),
            )
            .password(
                std::env::var("PASSWORD")
                    .expect("Fill in the login credentials in the environment variable"),
            )
            .login()
            .await?;

        let instance = instance.bind_java(java).await?.ensure_ready();
        let mut cmd = instance.launch(&auth).await?;
        let exit = cmd.spawn()?.status().await?;
        println!("Game exited: {exit}");

        Ok::<(), Error>(())
    })) {
        error!("{}", e)
    }
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
                .speed_by_timer(smol::Timer::after(Duration::from_secs(1)))
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

/// 检测阻塞时间
pub struct BlockingGuard {
    stopped: Arc<AtomicBool>,
    max_blocking_ns: Arc<AtomicU64>,
}

impl BlockingGuard {
    pub fn new(executor: &Executor<'static>) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let max_blocking_ns = Arc::new(AtomicU64::new(0));

        let stopped2 = stopped.clone();
        let max_blocking_ns2 = max_blocking_ns.clone();

        executor
            .spawn(async move {
                let mut last = Instant::now();

                loop {
                    smol::Timer::after(Duration::from_millis(1)).await;

                    let now = Instant::now();
                    let elapsed = now.duration_since(last);
                    last = now;

                    // Timer 本身也可能因为 worker 被 blocking 而延迟执行
                    let ns = elapsed.as_nanos() as u64;

                    max_blocking_ns2.fetch_max(ns, Relaxed);

                    if stopped2.load(Relaxed) {
                        break;
                    }
                }
            })
            .detach();

        Self {
            stopped,
            max_blocking_ns,
        }
    }
}

impl Drop for BlockingGuard {
    fn drop(&mut self) {
        self.stopped.store(true, Relaxed);

        let max = Duration::from_nanos(self.max_blocking_ns.load(Relaxed));

        eprintln!("max blocking: {:.3} ms", max.as_secs_f64() * 1000.0,);
    }
}
