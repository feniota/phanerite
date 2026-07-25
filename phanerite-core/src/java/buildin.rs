use crate::error::Result;
use crate::java::{JAVA_BIN_NAME, JavaRuntime};
use crate::storage::Storage;
use futures::stream::StreamExt;
use std::path::PathBuf;

pub struct BuildInRuntime {
    runtime_dir: PathBuf,
}

impl BuildInRuntime {
    pub fn new(storage: &Storage) -> Self {
        Self {
            runtime_dir: storage.runtime_dir().to_path_buf(),
        }
    }
    pub async fn list(&self) -> Result<Vec<JavaRuntime>> {
        let result = async_fs::read_dir(&self.runtime_dir)
            .await?
            .filter_map(async |x| x.ok())
            .filter_map(async |x| {
                let filename = x.file_name();
                let mut parts = filename.to_str()?.split('-');

                let _major = parts.next()?.parse::<u32>().ok()?;

                if parts.next()? != std::env::consts::OS {
                    return None;
                }
                if parts.next()? != std::env::consts::ARCH {
                    return None;
                }

                let _vendor = parts.next()?;
                let _package = parts.next()?;

                Some(x.path())
            })
            .map(|path| path.join("bin").join(JAVA_BIN_NAME))
            .map(JavaRuntime::from_path)
            .buffer_unordered(4)
            .filter_map(async |x| x.ok())
            .collect()
            .await;
        Ok(result)
    }
}
