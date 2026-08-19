use futures::StreamExt;
use phanerite_core::runtime::java::JavaRuntime;
use tracing::Level;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    smol::block_on(async {
        phanerite_core::runtime::java::detect_system()
            .map(JavaRuntime::from_path)
            .for_each_concurrent(4, async |x| println!("{:?}", x.await))
            .await;
    });
}
