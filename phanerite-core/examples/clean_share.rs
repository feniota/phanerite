use std::error::Error;
use tracing::Level;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    smol::block_on(async {
        let storage = phanerite_core::storage::Storage::new(".minecraft")?;
        phanerite_core::storage::bucket::clean_hardlink(&storage).await?;

        Ok::<(), phanerite_core::error::Error>(())
    })?;
    Ok(())
}
