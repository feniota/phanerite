use phanerite_core::auth::Authentication;
use phanerite_core::auth::offline;
use phanerite_core::download::DownloaderExt;
use phanerite_core::download::downloader::RawDownloader;
use phanerite_core::download::java::Zulu;
use phanerite_core::download::vanilla::VersionIndex;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::runtime::java::JavaManager;
use phanerite_core::storage::Storage;

fn main() -> Result<(), Error> {
    pollster::block_on(async {
        // Launcher data directory
        let storage = Storage::new(".minecraft").await?;

        // Create one downloader and reuse it for all downloads
        let raw = RawDownloader::builder().build().await?;
        let downloader = raw.with_group();

        // Download the manifest for the latest release
        let manifest = VersionIndex::sync(&downloader)
            .await?
            .latest_release()?
            .get_manifest(&downloader)
            .await?;

        // Create the game instance from the manifest
        let instance = Instance::create(manifest, Some("latest"), &storage, &downloader).await?;

        // Install Java if a matching runtime is not already available
        let java_manager = JavaManager::new(&storage).await;
        let java = java_manager
            .get_or_install::<Zulu>(instance.java_major(), &downloader, async |storage| storage)
            .await?;
        let instance = instance.bind_java(java).await?;

        // `install` only creates download tasks; the downloader executes them
        downloader
            .join(instance.install(std::collections::HashSet::new()).await?)
            .await
            .iter()
            .for_each(|e| eprintln!("{e}"));

        // Create a local, unauthenticated account and launch the game.
        let auth = offline::Authentication::new("Player");

        // HTTP requests are sent by the Downloader
        auth.ready(&downloader).await?;

        // Launch the game
        let instance = instance.ensure_ready();
        let mut command = instance.launch(&auth).await?;
        let status = command.spawn()?.status().await?;
        println!("Game exited with {status}");

        Ok::<(), Error>(())
    })
}
