use phanerite_core::auth::Authentication;
use phanerite_core::instance::Instance;
use phanerite_core::storage::Storage;
use std::error::Error;
use tracing::Level;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
        let storage = Storage::new(".minecraft").await?;
        let instance = Instance::open("1.21.1", &storage).await?;
        let auth = phanerite_core::auth::offline::Authentication::new("Steve");
        let arguments = auth
            .args(&instance)
            .await?
            .set_memory(Some(2048), Some(2048));

        println!("{arguments}");

        Ok::<(), phanerite_core::error::Error>(())
    })?;
    Ok(())
}
