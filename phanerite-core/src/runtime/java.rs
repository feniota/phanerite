use crate::download::Downloader;
use crate::download::java::JavaDownload;
use crate::error::{Error, Result};
use crate::runtime::{RuntimePath, RuntimeScanPath};
use crate::storage::Storage;
use futures::{Stream, StreamExt};
use std::env;
use std::ffi::OsStr;
use std::hash::Hasher;
use std::path::PathBuf;
use tracing::trace;

#[cfg(target_os = "windows")]
const JAVA_BIN_NAME: &str = "java.exe";
#[cfg(not(target_os = "windows"))]
const JAVA_BIN_NAME: &str = "java";

#[derive(Debug, Clone, Eq)]
pub struct JavaRuntime {
    pub name: String,
    pub major: u32,
    pub version: String,
    pub path: PathBuf,
}

impl PartialEq for JavaRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl std::hash::Hash for JavaRuntime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

impl AsRef<OsStr> for JavaRuntime {
    fn as_ref(&self) -> &OsStr {
        self.path.as_ref()
    }
}

impl JavaRuntime {
    pub async fn from_path(path: impl Into<PathBuf>) -> Result<Self> {
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

// 列出内建 Java
/// Lists the bundled Java installations
pub fn list_build_in(storage: impl AsRef<Storage>) -> impl Stream<Item = PathBuf> {
    #[allow(clippy::large_enum_variant)]
    enum State<F>
    where
        F: Future,
    {
        Init(F),
        Read(async_fs::ReadDir),
    }
    futures::stream::unfold(
        State::Init(async move {
            let runtime_dir = storage.as_ref().runtime_dir();
            async_fs::read_dir(runtime_dir).await
        }),
        async |state| match state {
            State::Init(f) => {
                let mut dir = f.await.unwrap();
                dir.next().await.map(|t| (t, State::Read(dir)))
            }
            State::Read(mut r) => r.next().await.map(|t| (t, State::Read(r))),
        },
    )
    .filter_map(async |x| x.ok())
    .filter_map(async |x| {
        RuntimePath::try_from(x.file_name())
            .ok()?
            .matches_current()
            .then_some(x.path())
    })
    .map(|path| path.join("bin").join(JAVA_BIN_NAME))
    .map(|x| std::path::absolute(&x).unwrap_or(x))
}

// 探测系统的 Java
/// Detects the system's Java installations
pub fn detect_system() -> impl Stream<Item = PathBuf> {
    let java_home = env::var_os("JAVA_HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("bin");
    let iter = env::var_os("PATH")
        .into_iter()
        .flat_map(|x| env::split_paths(&x).collect::<Vec<_>>())
        .chain(std::iter::once(java_home))
        .map(|x| x.join(JAVA_BIN_NAME))
        .filter(|x| x.is_file())
        .map(|x| std::path::absolute(&x).unwrap_or(x));
    futures::stream::iter(iter)
}

pub struct JavaManager<'scan, S: RuntimeScanPath> {
    javas: scc::HashSet<JavaRuntime>,

    // 是否扫描系统 Java
    /// Whether to scan the system's Java installations
    system: bool,
    // 内建 Runtime 扫描路径
    /// Scan paths for the bundled runtimes
    scan_paths: &'scan S,
}

impl<'scan, S: RuntimeScanPath> JavaManager<'scan, S> {
    pub async fn new(scan: &'scan S) -> Self {
        let new = Self {
            javas: Default::default(),
            system: true,
            scan_paths: scan,
        };
        new.refresh().await;
        new
    }
    pub fn enable_system(mut self) -> Self {
        self.system = true;
        self
    }
    pub fn disable_system(mut self) -> Self {
        self.system = false;
        self
    }
    pub async fn refresh(&self) {
        let insert = async |java| {
            if let Ok(v) = java.await {
                let _ = self.javas.insert_async(v).await;
            }
        };

        let build_in = self.scan_paths.storages().map(list_build_in);

        self.javas.clear_async().await;
        if self.system {
            let system = detect_system();
            futures::stream::iter(build_in)
                .flatten()
                .chain(system)
                .map(JavaRuntime::from_path)
                .for_each_concurrent(4, insert)
                .await;
        } else {
            futures::stream::iter(build_in)
                .flatten()
                .map(JavaRuntime::from_path)
                .for_each_concurrent(4, insert)
                .await;
        }
    }
    pub async fn get_or_install<J: JavaDownload>(
        &self,
        major: u32,
        downloader: &impl Downloader,
        install_to: impl AsyncFnOnce(&S) -> S::Provider<'_>,
    ) -> Result<JavaRuntime> {
        if let Some(v) = self.find(major).await {
            return Ok(v);
        }
        let storage = install_to(self.scan_paths).await;
        let task = J::get_major(major, downloader, storage.as_ref()).await?;
        downloader.download(task).await?;
        self.refresh().await;
        self.find(major)
            .await
            .ok_or(Error::other("Failed to install java"))
    }
    pub async fn all(&self) -> Vec<JavaRuntime> {
        let mut v = vec![];
        self.for_each(|x| {
            v.push(x.clone());
            true
        })
        .await;
        v
    }
    pub async fn for_each<F>(&self, f: F) -> bool
    where
        F: FnMut(&JavaRuntime) -> bool,
    {
        self.javas.iter_async(f).await
    }
    pub async fn filter(&self, major: u32) -> Vec<JavaRuntime> {
        let mut v = vec![];
        self.javas
            .iter_async(|x| {
                if x.major == major {
                    v.push(x.clone())
                }
                true
            })
            .await;
        v
    }
    pub async fn find(&self, major: u32) -> Option<JavaRuntime> {
        let mut r = None;
        self.javas
            .iter_async(|x| {
                if x.major == major {
                    r = Some(x.clone());
                    false
                } else {
                    true
                }
            })
            .await;
        r
    }
}
