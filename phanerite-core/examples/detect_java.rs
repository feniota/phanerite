use std::error::Error;
use tracing::Level;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
        phanerite_core::runtime::java::detect_system()
            .await?
            .iter()
            .for_each(|x| println!("{x:?}"));

        Ok::<(), phanerite_core::error::Error>(())
    })?;
    Ok(())
}
