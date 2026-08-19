use crate::download::Downloader;
use crate::download::java::JavaDownload;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::manifest::InstanceManifest;
use crate::runtime::RuntimePath;
use crate::storage::Storage;
use async_lock::Mutex;
use futures::StreamExt;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tracing::trace;

#[cfg(target_os = "windows")]
const JAVA_BIN_NAME: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
const JAVA_BIN_NAME: &str = "java";

pub async fn install_java<J: JavaDownload>(
    major: u32,
    storage: &Storage,
    downloader: &impl Downloader,
) -> Result<Option<DownloadTask>> {
    if find_java(major, storage).await.into_iter().next().is_some() {
        return Ok(None);
    }
    let task = J::get_major(major, downloader, storage).await?;
    Ok(Some(task))
}

pub async fn find_java(major: u32, storage: &Storage) -> Vec<JavaRuntime> {
    let build_in = list_build_in(storage.runtime_dir()).await;
    let system = detect_system().await;
    build_in
        .into_iter()
        .chain(system)
        .filter(|x| x.major == major)
        .collect()
}

impl<R: Clone, C: Clone> Instance<'_, R, C> {
    pub async fn install_java<J: JavaDownload>(
        &self,
        storage: &Storage,
        downloader: &impl Downloader,
    ) -> Result<Option<DownloadTask>> {
        install_java::<J>(self.manifest.java_major(), storage, downloader).await
    }
    pub async fn find_java(&self, storage: &Storage) -> Vec<JavaRuntime> {
        find_java(self.manifest.java_major(), storage).await
    }
}

impl InstanceManifest {
    pub fn java_major(&self) -> u32 {
        self.java_version.major_version
    }
}

#[derive(Debug, Clone)]
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
pub async fn list_build_in(runtime_dir: &Path) -> Vec<JavaRuntime> {
    futures::stream::iter(async_fs::read_dir(runtime_dir).await)
        .flatten()
        .filter_map(async |x| x.ok())
        .filter_map(async |x| {
            RuntimePath::try_from(x.file_name())
                .ok()?
                .matches_current()
                .then_some(x.path())
        })
        .map(|path| path.join("bin").join(JAVA_BIN_NAME))
        .map(|x| std::path::absolute(&x).unwrap_or(x))
        .map(JavaRuntime::from_path)
        .buffer_unordered(4)
        .filter_map(async |x| x.ok())
        .collect()
        .await
}

/// 探测系统的 Java
pub async fn detect_system() -> Vec<JavaRuntime> {
    let java_home = env::var_os("JAVA_HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("bin");
    let javas = env::split_paths(&env::var_os("PATH").unwrap_or_default())
        .chain(std::iter::once(java_home))
        .map(|x| x.join(JAVA_BIN_NAME))
        .filter(|x| x.is_file())
        .map(|x| std::path::absolute(&x).unwrap_or(x))
        .collect::<HashSet<_>>();
    futures::stream::iter(javas)
        .map(JavaRuntime::from_path)
        .buffer_unordered(4)
        .filter_map(async |x| x.ok())
        .collect()
        .await
}

pub struct GlobalManager<'storage> {
    storage: &'storage Storage,
    build_in: Mutex<Vec<JavaRuntime>>,
    system: Mutex<Vec<JavaRuntime>>,
}

impl<'storage> GlobalManager<'storage> {
    pub async fn new(storage: &'storage Storage) -> Self {
        let new = Self {
            storage,
            build_in: Default::default(),
            system: Default::default(),
        };
        new.refresh().await;
        new
    }
    pub async fn refresh(&self) {
        *self.build_in.lock().await = list_build_in(self.storage.runtime_dir()).await;
        *self.system.lock().await = detect_system().await;
    }
}
