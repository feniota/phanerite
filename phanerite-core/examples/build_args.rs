use phanerite_core::instance::arguments::variables::Variables;
use phanerite_core::instance::arguments::LaunchArguments;
use phanerite_core::instance::Instance;
use phanerite_core::storage::Storage;
use std::error::Error;
use tracing::Level;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
        let instance_dir = ".minecraft/versions/26.2";
        let storage = Storage::new(".minecraft")?;
        let manifest = Instance::open(instance_dir).await?;
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
            .build(&manifest.manifest, instance_dir, &storage)?;
        let arguments = LaunchArguments::from_vars(&manifest.manifest, variables);

        for i in arguments.args {
            println!("{} {}", i.0, i.1.unwrap_or_default())
        }

        Ok::<(), phanerite_core::error::Error>(())
    })?;
    Ok(())
}
