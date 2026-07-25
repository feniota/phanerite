use phanerite_core::instance::Instance;
use phanerite_core::instance::arguments::variables::Variables;
use phanerite_core::storage::Storage;
use std::error::Error;
use tracing::Level;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
        let instance_dir = ".minecraft/versions/latest";
        let storage = Storage::new(".minecraft")?;
        let instance = Instance::open(instance_dir).await?;
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
        let arguments = instance.to_arguments(variables);

        for i in arguments.flatten_iter() {
            println!("{}", i)
        }

        Ok::<(), phanerite_core::error::Error>(())
    })?;
    Ok(())
}
