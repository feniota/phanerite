use crate::download::downloader::Downloader;
use crate::download::java::JavaDownload;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::instance_info::InstanceManifest;
use crate::runtime::RuntimePath;
use crate::storage::Storage;
use futures::StreamExt;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tracing::{debug, trace};

#[cfg(target_os = "windows")]
const JAVA_BIN_NAME: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
const JAVA_BIN_NAME: &str = "java";

impl Instance {
    pub async fn install_java<J: JavaDownload>(
        &self,
        downloader: &Downloader,
        storage: &Storage,
    ) -> Result<Option<DownloadTask>> {
        if list_build_in(storage.runtime_dir())
            .await
            .unwrap_or_default()
            .iter()
            .find(|x| x.major == self.manifest.java_version.major_version)
            .is_some()
        {
            return Ok(None);
        }
        let task = J::get_major(
            self.manifest.java_version.major_version,
            downloader,
            storage,
        )
        .await?;
        Ok(Some(task))
    }
    pub async fn find_java(&self, storage: &Storage) -> Vec<JavaRuntime> {
        let build_in = list_build_in(storage.runtime_dir())
            .await
            .unwrap_or_default();
        let system = detect_system().await.unwrap_or_default();
        build_in
            .into_iter()
            .chain(system)
            .filter(|x| x.major == self.manifest.java_version.major_version)
            .collect()
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
    async fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
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

/// 列出内建 Java
pub async fn list_build_in(runtime_dir: &Path) -> Result<Vec<JavaRuntime>> {
    let result = async_fs::read_dir(runtime_dir)
        .await?
        .filter_map(async |x| x.ok())
        .filter_map(async |x| {
            RuntimePath::try_from(x.file_name()).ok()?;
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

/// 探测系统的 Java
pub async fn detect_system() -> Result<Vec<JavaRuntime>> {
    debug!("detect runtime");
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
        debug!("found runtime: {}", path.to_string_lossy());
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
