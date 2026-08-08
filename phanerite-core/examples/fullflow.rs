use phanerite_core::auth::yggdrasil::Authentication;
use phanerite_core::download::Downloader;
use phanerite_core::download::authlib_injector::AuthlibInjector;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::java::zulu::Zulu;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::instance::manifest::InstanceManifest;
use phanerite_core::mod_loader::LoaderMeta;
use phanerite_core::mod_loader::neoforge::NeoForge;
use phanerite_core::runtime::java::install_java;
use phanerite_core::storage::SharePreference::Hardlink;
use phanerite_core::*;
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use tracing::{Level, error};
use url::Url;

fn main() {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    if let Err(e) = smol::block_on(async {
        let storage = storage::Storage::new(".minecraft")
            .await?
            .share_preference(Hardlink);
        let (cleaner, _shutdown) = storage.run_cleaner();
        smol::spawn(cleaner).detach();
        let downloader = download::downloader::RawDownloader::builder(&storage)
            .build()
            .await?;
        let injector = AuthlibInjector::new(&storage);
        let mut group = DownloadGroup::new(&downloader);
        let _g = monitor(&group).await;

        let _ = async_fs::remove_dir_all(storage.versions_dir().join("1.21.1-nf")).await;

        let version: InstanceManifest = VersionIndex::sync(&downloader)
            .await?
            .iter()
            .find(|x| x.id == "1.21.1")
            .expect("Version not found")
            // .latest_release()?
            .get_manifest(&downloader)
            .await?
            .into();

        group.extend(install_java::<Zulu>(version.java_major(), &storage, &downloader).await?);
        group.extend(injector.update(&downloader).await?);
        group.exec().await.iter().for_each(|e| error!("{e}"));

        let instance = Instance::create(version, Some("1.21.1-nf"), &storage, &downloader).await?;

        let java = instance
            .find_java(&storage)
            .await
            .into_iter()
            .next()
            .ok_or(Error::other("No available java"))?;
        let mut instance = instance.bind_java(java.clone()).await?;

        instance
            .install_loader::<NeoForge>("1.21.1", &downloader, async |iter| {
                // let iter = iter
                //     .inspect(|x| println!("{}:{} stable:{}", x.name(), x.version(), x.stable()));
                let latest = iter
                    .collect::<BTreeSet<_>>()
                    .pop_last()
                    .expect("No available loader version");
                println!(
                    "{}:{} stable:{}",
                    latest.name(),
                    latest.version(),
                    latest.stable()
                );
                Ok(latest)
            })
            .await?;

        group
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

        let mut cmd = instance
            .ensure_ready()
            .bind_java(java)
            .await?
            .launch(&auth)
            .await?;
        let exit = cmd.spawn()?.status().await?;
        println!("Game exited: {exit}");

        Ok::<(), Error>(())
    }) {
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
