use phanerite_core::auth::yggdrasil::Authentication;
use phanerite_core::download::authlib_injector::AuthlibInjector;
use phanerite_core::download::java::zulu::Zulu;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::vanilla::version_info::VersionManifest;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::storage::ShareStrategy::Force;
use phanerite_core::*;
use std::num::NonZeroU8;
use tracing::log::error;
use tracing::{Level, info};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    if let Err(e) = smol::block_on(async {
        let storage = storage::Storage::new(".minecraft")?.share_strategy(Force);
        let downloader = download::downloader::Downloader::builder(&storage)
            .concurrent(NonZeroU8::new(16).unwrap())
            .retries(3)
            .build()
            .await?;

        let _ = Instance::create(
            VersionManifest::get(
                VersionIndex::sync(&downloader).await?.latest_release()?,
                &downloader,
            )
            .await?,
            "latest",
            &storage,
            &downloader,
        )
        .await;

        let instance_dir = storage.versions_dir().join("latest");
        let instance = Instance::open(&instance_dir).await?;

        let javas = instance.find_java(&storage).await;
        if javas.is_empty() {
            info!("Install java");
            let task = instance.install_java(Zulu, &downloader, &storage).await?;
            downloader
                .download(task.expect("Failed to download java"))
                .await?
        }
        let javas = instance.find_java(&storage).await;
        let Some(java) = javas.first() else {
            return Err(Error::other("Failed to install java"));
        };

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
        if let Some(t) = injector.update(&downloader).await? {
            downloader.download(t).await?
        }

        let arguments = auth.injected_args(&instance, &storage, &injector).await?;

        async_process::Command::new(&java.path)
            .args(arguments.iter())
            .spawn()?;

        Ok::<(), Error>(())
    }) {
        error!("{}", e)
    }
}
