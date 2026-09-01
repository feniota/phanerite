use phanerite_core::error::{Error, Result};
use phanerite_core::storage::Storage;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    smol::block_on(async {
        let storage = Storage::new(".minecraft").await?;

        storage.clean_hardlink().await?;
        storage.clean_symlink().await?;

        Ok::<(), Error>(())
    })?;
    Ok(())
}
