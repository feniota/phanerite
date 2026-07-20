use crate::error::Result;
use crate::java::{BIN_NAME, JavaRuntime};
use futures::StreamExt;
use std::collections::HashSet;
use std::env;
use std::num::NonZeroU8;
use std::path::PathBuf;
use tracing::debug;

pub async fn detect(concurrent: NonZeroU8) -> Result<Vec<JavaRuntime>> {
    debug!("detect java");
    let paths = env::var_os("PATH").unwrap_or_default();
    let java_home = env::var_os("JAVA_HOME")
        .map(PathBuf::from)
        .map(|x| x.join("bin").join(BIN_NAME))
        .unwrap_or_default();
    let javas = env::split_paths(&paths)
        .map(|x| x.join(BIN_NAME))
        .chain(std::iter::once(java_home))
        .filter(|x| x.is_file())
        .map(|x| std::path::absolute(&x).unwrap_or(x));

    let result = async_lock::Mutex::new(Vec::new());
    let set = async_lock::Mutex::new(HashSet::new());

    let check_java = async |path: PathBuf| -> Result<()> {
        if !set.lock().await.insert(path.clone()) {
            return Ok(());
        }
        debug!("found java: {}", path.to_string_lossy());
        result
            .lock()
            .await
            .push(JavaRuntime::from_path(path).await?);
        Ok(())
    };

    futures::stream::iter(javas)
        .for_each_concurrent(concurrent.get() as usize, async |path| {
            let _ = check_java(path).await;
        })
        .await;

    Ok(result.into_inner())
}
