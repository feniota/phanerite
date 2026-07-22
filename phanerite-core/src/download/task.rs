use crate::download::extract::ExtractTask;
use crate::download::task::Target::{Extract, File};
use crate::storage::Storage;
use crate::utils::{EmptyHash, Hash, HashValue};
use event_listener::Event;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, OnceLock};

pub struct Missing;

pub enum Target {
    File(PathBuf),
    Extract(ExtractTask),
}

impl From<PathBuf> for Target {
    fn from(value: PathBuf) -> Self {
        File(value)
    }
}
impl From<ExtractTask> for Target {
    fn from(value: ExtractTask) -> Self {
        Extract(value)
    }
}

pub struct DownloadTaskBuilder<U, T> {
    url: U,
    target: T,
    bucket: Option<PathBuf>,
    file_name: Option<String>,
    file_size: Option<u64>,
    file_hash: Hash,
}

pub struct DownloadTask {
    pub(super) url: String,
    pub(super) target: Target,
    pub(super) bucket: Option<PathBuf>,
    pub(super) file_hash: Hash,

    pub process: DownloadProcess,
}

#[derive(Clone)]
pub struct DownloadProcess {
    inner: Arc<DownloadProcessInner>,
}

struct DownloadProcessInner {
    name: Option<String>,
    event: Event,

    current: AtomicU64,
    total: OnceLock<u64>,

    started: AtomicBool,
    extracting: AtomicBool,
    finished: AtomicBool,
    cancelled: AtomicBool,
}

impl DownloadTask {
    pub fn builder() -> DownloadTaskBuilder<Missing, Missing> {
        DownloadTaskBuilder {
            url: Missing,
            target: Missing,
            bucket: None,
            file_name: None,
            file_size: None,
            file_hash: Hash::Empty(EmptyHash),
        }
    }
}

impl<T> DownloadTaskBuilder<Missing, T> {
    pub fn url(self, url: impl Into<String>) -> DownloadTaskBuilder<String, T> {
        DownloadTaskBuilder {
            url: url.into(),
            target: self.target,
            bucket: self.bucket,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
}

impl<U> DownloadTaskBuilder<U, Missing> {
    pub fn to_asset(
        self,
        path: impl AsRef<Path>,
        storage: &Storage,
    ) -> DownloadTaskBuilder<U, PathBuf> {
        DownloadTaskBuilder {
            url: self.url,
            target: storage.assets_objects().join(path),
            bucket: self.bucket,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
    pub fn to_library(
        self,
        path: impl AsRef<Path>,
        storage: &Storage,
    ) -> DownloadTaskBuilder<U, PathBuf> {
        DownloadTaskBuilder {
            url: self.url,
            target: storage.libraries_dir().join(path),
            bucket: self.bucket,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
    pub fn to_path(self, path: PathBuf) -> DownloadTaskBuilder<U, PathBuf> {
        DownloadTaskBuilder {
            url: self.url,
            target: path,
            bucket: self.bucket,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
    pub fn extract_to(self, extract_task: ExtractTask) -> DownloadTaskBuilder<U, ExtractTask> {
        DownloadTaskBuilder {
            url: self.url,
            target: extract_task,
            bucket: self.bucket,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
}

impl<U, P> DownloadTaskBuilder<U, P> {
    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = Some(name.into());
        self
    }
    pub fn file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }
    pub fn hash<H: HashValue>(mut self, hash: H) -> Self {
        self.file_hash = hash.into();
        self
    }
    pub fn share(mut self, storage: &Storage) -> Self {
        self.bucket = Some(storage.share_dir().to_path_buf());
        self
    }
}

impl<P: Into<Target>> DownloadTaskBuilder<String, P> {
    pub fn build(self) -> DownloadTask {
        DownloadTask {
            url: self.url,
            target: self.target.into(),
            bucket: self.bucket,
            file_hash: self.file_hash,
            process: DownloadProcess {
                inner: Arc::new(DownloadProcessInner {
                    name: self.file_name,
                    event: Default::default(),

                    current: AtomicU64::new(0),
                    total: if let Some(t) = self.file_size {
                        let cell = OnceLock::new();
                        cell.set(t).unwrap();
                        cell
                    } else {
                        OnceLock::new()
                    },

                    started: AtomicBool::new(false),
                    extracting: AtomicBool::new(false),
                    finished: AtomicBool::new(false),
                    cancelled: AtomicBool::new(false),
                }),
            },
        }
    }
}

impl DownloadProcess {
    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }
    pub fn total(&self) -> Option<u64> {
        self.inner.total.get().copied()
    }
    pub fn current(&self) -> u64 {
        self.inner.current.load(Relaxed)
    }
    pub async fn changed(&self) {
        self.inner.event.listen().await;
    }
    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Release);
        self.inner.event.notify(usize::MAX);
    }
    pub fn is_started(&self) -> bool {
        self.inner.started.load(Acquire)
    }
    pub fn is_extracting(&self) -> bool {
        self.inner.extracting.load(Acquire)
    }
    pub fn is_finished(&self) -> bool {
        self.inner.finished.load(Acquire)
    }
    pub fn is_canceled(&self) -> bool {
        self.inner.cancelled.load(Acquire)
    }

    pub(super) fn set_total(&self, total: u64) -> bool {
        if self.inner.total.set(total).is_ok() {
            self.inner.event.notify(usize::MAX);
            true
        } else {
            false
        }
    }
    pub(super) fn step(&self, size: u64) {
        self.inner.current.fetch_add(size, Relaxed);
        self.inner.event.notify(usize::MAX);
    }
    pub(super) fn start(&self) {
        self.inner.started.store(true, Release);
        self.inner.event.notify(usize::MAX);
    }
    pub(super) fn extracting(&self) {
        self.inner.extracting.store(true, Release);
        self.inner.event.notify(usize::MAX);
    }
    pub(super) fn finish(&self) {
        self.inner.finished.store(true, Release);
        self.inner.event.notify(usize::MAX);
    }
}

pub fn filter_existed(
    tasks: impl Iterator<Item = DownloadTask>,
) -> impl Iterator<Item = DownloadTask> {
    tasks.filter(|x| {
        if let File(p) = &x.target {
            !p.exists()
        } else {
            true
        }
    })
}
