use crate::download::downloader::Downloader;
use crate::download::java::JavaDownload;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::instance::instance_info::InstanceManifest;
use crate::instance::Instance;
use crate::java::buildin::BuildInRuntime;
use crate::java::system::detect;
use crate::storage::Storage;
use std::ffi::OsStr;
use std::path::PathBuf;
use tracing::trace;

pub mod buildin;
pub mod system;

#[cfg(target_os = "windows")]
const JAVA_BIN_NAME: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
const JAVA_BIN_NAME: &str = "java";

impl Instance {
    pub async fn install_java(
        &self,
        java: impl JavaDownload,
        downloader: &Downloader,
        storage: &Storage,
    ) -> Result<Option<DownloadTask>> {
        if BuildInRuntime::new(storage)
            .list()
            .await
            .unwrap_or_default()
            .iter()
            .find(|x| x.major == self.manifest.java_version.major_version)
            .is_some()
        {
            return Ok(None);
        }
        let task = java
            .get_major(
                self.manifest.java_version.major_version,
                downloader,
                storage,
            )
            .await?;
        Ok(Some(task))
    }
    pub async fn find_java(&self, storage: &Storage) -> Vec<JavaRuntime> {
        let build_in = BuildInRuntime::new(storage)
            .list()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|x| x.major == self.manifest.java_version.major_version);
        let system = detect()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|x| x.major == self.manifest.java_version.major_version);
        build_in.chain(system).collect()
    }
}

impl InstanceManifest {
    pub fn java_major(&self) -> u32 {
        self.java_version.major_version
    }
}

#[derive(Debug)]
pub struct JavaRuntime {
    pub name: String,
    pub major: u32,
    pub version: String,
    pub path: PathBuf,
}

impl AsRef<OsStr> for JavaRuntime {
    fn as_ref(&self) -> &OsStr {
        self.path.as_ref()
    }
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
