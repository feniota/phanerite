use crate::error::Result;
use std::path::PathBuf;
use tracing::trace;

pub mod buildin;
pub mod system;

#[cfg(target_os = "windows")]
const BIN_NAME: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
const BIN_NAME: &str = "java";

#[derive(Debug)]
pub struct JavaRuntime {
    name: String,
    major: usize,
    version: String,
    path: PathBuf,
}

impl JavaRuntime {
    async fn from_path(path: PathBuf) -> Result<Self> {
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

        Ok(Self {
            name: name.unwrap_or("Unknown".to_string()),
            major: major.unwrap_or_default(),
            version: version.unwrap_or_default(),
            path,
        })
    }
}
