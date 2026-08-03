use phanerite_core::auth::yggdrasil::Authentication;
use phanerite_core::download::authlib_injector::AuthlibInjector;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::java::zulu::Zulu;
use phanerite_core::download::mod_loader::LoaderMeta;
use phanerite_core::download::mod_loader::neoforge::NeoForge;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::storage::ShareStrategy::Force;
use phanerite_core::*;
use std::collections::{BTreeSet, HashSet};
use tracing::{Level, error};
use url::Url;

fn main() {
    // let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    if let Err(e) = smol::block_on(async {
        let storage = storage::Storage::new(".minecraft")?.share_strategy(Force);
        let downloader = download::downloader::Downloader::builder(&storage)
            .build()
            .await?;
        let injector = AuthlibInjector::new(&storage);

        let _ = async_fs::remove_dir_all(storage.versions_dir().join("1.21.1-nf")).await;

        let version = VersionIndex::sync(&downloader)
            .await?
            .iter()
            .find(|x| x.id == "1.21.1")
            .expect("Version not found")
            // .latest_release()?
            .install_loader::<NeoForge>(&downloader, async |iter| {
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
        let instance = Instance::create(version, "1.21.1-nf", &storage, &downloader).await?;

        let mut group = DownloadGroup::new();
        group.extend(instance.install_less(HashSet::new(), &storage).await?);
        group.extend(instance.install_java::<Zulu>(&downloader, &storage).await?);
        group.extend(injector.update(&downloader).await?);

        // 显示下载速度和进度
        let monitor = group.processes();
        let total = monitor.total() as f64 / 1024.0 / 1024.0;
        println!("Total size: {:.2} MiB ({} tasks)", total, monitor.len());
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

        // let errs = group
        //     .exec_with_mirror(&downloader, download::mirror::granodiorite::Granodiorite)
        //     .await;
        let errs = group.exec(&downloader).await;
        errs.iter().for_each(|e| error!("{}", e));

        let auth = Authentication::new_login(&downloader)
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

        let arguments = auth.injected_args(&instance, &storage, &injector).await?;
        let java = instance.find_java(&storage).await.remove(0);

        async_process::Command::new(java)
            .args(arguments.iter())
            .spawn()?;

        Ok::<(), Error>(())
    }) {
        error!("{}", e)
    }
}
