use phanerite_core::download::java::zulu::Zulu;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::vanilla::version_info::VersionManifest;
use phanerite_core::error::Error;
use phanerite_core::instance::variables::Variables;
use phanerite_core::instance::Instance;
use phanerite_core::storage::ShareStrategy::Force;
use phanerite_core::*;
use std::num::NonZeroU8;
use tracing::{info, Level};

fn main() -> error::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
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
            instance.install_java(Zulu, &downloader, &storage).await?;
        }
        let javas = instance.find_java(&storage).await;
        let Some(java) = javas.first() else {
            return Err(Error::other("Failed to install java"));
        };

        let variables = Variables::builder()
            .required(
                "Steve",
                "10000000-0000-0000-0000-000000000000",
                "20000000-0000-0000-0000-000000000000",
            )
            .modern(
                "30000000-0000-0000-0000-000000000000",
                "40000000-0000-0000-0000-000000000000",
            )
            .feature("is_demo_user")
            .build(&instance, &storage)?;
        let arguments = variables.to_arguments(&instance);

        async_process::Command::new(&java.path)
            .args(arguments.iter())
            .spawn()?;

        Ok::<(), Error>(())
    })
}
