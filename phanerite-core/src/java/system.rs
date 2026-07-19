use crate::error::Result;
use crate::java::{JavaRuntime, BIN_NAME};
use futures::StreamExt;
use std::collections::HashSet;
use std::env;
use std::num::NonZeroU8;
use std::path::PathBuf;
use tracing::{debug, trace};

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
        .map(|x| x.absolute().unwrap_or(x));

    let result = async_lock::Mutex::new(Vec::new());
    let set = async_lock::Mutex::new(HashSet::new());

    let check_java = async |path: PathBuf| -> Result<()> {
        if !set.lock().await.insert(path.clone()) {
            return Ok(());
        }
        debug!("check java: {}", path.to_string_lossy());
        let mut name = None;
        let mut major = None;
        let mut version = None;
        let out = async_process::Command::new(&path)
            .arg("-XshowSettings:properties")
            .arg("-version")
            .output()
            .await?
            .stderr;

        for (k, v) in String::from_utf8_lossy(&out)
            .lines()
            .filter_map(|x| x.trim().split_once('='))
            .map(|(k, v)| (k.trim(), v.trim()))
        {
            trace!("properties: {}={}", k, v);
            if k == "java.runtime.name" {
                name = Some(v.to_string());
            }
            if k == "java.specification.version" {
                major = Some(v.parse().unwrap_or_default())
            }
            if k == "java.runtime.version" {
                version = Some(v.to_string());
            }
        }

        result.lock().await.push(JavaRuntime {
            name: name.unwrap_or("Unknown".to_string()),
            major: major.unwrap_or_default(),
            version: version.unwrap_or_default(),
            path,
        });
        Ok(())
    };

    futures::stream::iter(javas)
        .for_each_concurrent(concurrent.get() as usize, async |path| {
            let _ = check_java(path).await;
        })
        .await;

    Ok(result.into_inner())
}
