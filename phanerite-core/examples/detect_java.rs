use std::error::Error;
use std::num::NonZeroU8;
use tracing::Level;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
        phanerite_core::java::system::detect(NonZeroU8::new(4).unwrap())
            .await?
            .iter()
            .for_each(|x| println!("{x:?}"));

        Ok::<(), phanerite_core::error::Error>(())
    })?;
    Ok(())
}
