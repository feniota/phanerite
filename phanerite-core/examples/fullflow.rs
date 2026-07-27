use phanerite_core::auth::yggdrasil::Authentication;
use phanerite_core::download::authlib_injector::AuthlibInjector;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::java::zulu::Zulu;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::vanilla::version_info::VersionManifest;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::storage::ShareStrategy::Force;
use phanerite_core::*;
use std::num::NonZeroU8;
use tracing::log::error;
use tracing::{info, Level};

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    if let Err(e) = smol::block_on(async {
        let storage = storage::Storage::new(".minecraft")?.share_strategy(Force);
        let downloader = download::downloader::Downloader::builder(&storage)
            .concurrent(NonZeroU8::new(16).unwrap())
            .retries(3)
            .build()
            .await?;
        let mut group = DownloadGroup::new();

        group.extend(
            Instance::create(
                VersionManifest::get(
                    VersionIndex::sync(&downloader).await?.latest_release()?,
                    &downloader,
                )
                .await?,
                "latest",
                &storage,
                &downloader,
            )
            .await?,
        );

        let instance_dir = storage.versions_dir().join("latest");
        let instance = Instance::open(&instance_dir).await?;

        if instance.find_java(&storage).await.is_empty() {
            info!("Install java");
            group.extend(
                instance
                    .install_java(Zulu, &downloader, &storage)
                    .await?
                    .into_iter(),
            );
        }

        let auth = Authentication::new_login(&downloader)
            .custom("https://littleskin.cn/api/yggdrasil")
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

        let injector = AuthlibInjector::new(&storage);
        group.extend(injector.update(&downloader).await?.into_iter());

        let processes = group.processes();

        smol::spawn(async move {
            loop {
                println!("Downloading: {}", processes.downloading());
                println!(
                    "Speed: {:.2} MiB/s",
                    processes
                        .speed_by_timer(smol::Timer::after(std::time::Duration::from_secs(1)))
                        .await as f64
                        / 1024.0
                        / 1024.0
                )
            }
        })
        .detach();

        let errs = group.exec(&downloader).await;
        if !errs.is_empty() {
            errs.iter().for_each(|e| error!("{}", e))
        }

        let arguments = auth.injected_args(&instance, &storage, &injector).await?;

        async_process::Command::new(
            instance
                .find_java(&storage)
                .await
                .first()
                .expect("No available java"),
        )
        .args(arguments.iter())
        .spawn()?;

        Ok::<(), Error>(())
    }) {
        error!("{}", e)
    }
}
