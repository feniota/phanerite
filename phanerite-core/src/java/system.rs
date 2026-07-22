use crate::error::Result;
use crate::java::{JavaRuntime, JAVA_BIN_NAME};
use futures::StreamExt;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use tracing::debug;

pub async fn detect() -> Result<Vec<JavaRuntime>> {
    debug!("detect java");
    let paths = env::var_os("PATH").unwrap_or_default();
    let java_home = env::var_os("JAVA_HOME")
        .map(PathBuf::from)
        .map(|x| x.join("bin").join(JAVA_BIN_NAME))
        .unwrap_or_default();
    let javas = env::split_paths(&paths)
        .map(|x| x.join(JAVA_BIN_NAME))
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
        .for_each_concurrent(4, async |path| {
            let _ = check_java(path).await;
        })
        .await;

    Ok(result.into_inner())
}
