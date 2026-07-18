use crate::storage::Storage;
use crate::utils::{EmptyHash, Hash, HashValue};
use event_listener::Event;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, OnceLock};

pub struct DownloadTaskBuilder {
    url: Option<String>,
    target: Option<PathBuf>,
    bucket: Option<PathBuf>,
    file_name: Option<String>,
    file_size: Option<u64>,
    file_hash: Hash,
}

pub struct DownloadTask {
    pub(super) url: String,
    pub(super) target: PathBuf,
    pub(super) bucket: Option<PathBuf>,
    pub(super) file_hash: Hash,

    pub process: DownloadProcess,
}

pub struct DownloadProcess {
    inner: Arc<DownloadProcessInner>,
}

struct DownloadProcessInner {
    name: Option<String>,
    current: AtomicU64,
    cancelled: AtomicBool,
    event: Event,
    total: OnceLock<u64>,
}

impl DownloadTask {
    pub fn builder() -> DownloadTaskBuilder {
        DownloadTaskBuilder {
            url: None,
            target: None,
            bucket: None,
            file_name: None,
            file_size: None,
            file_hash: Hash::Empty(EmptyHash),
        }
    }
}

impl DownloadTaskBuilder {
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
    pub fn to_asset(mut self, path: &Path, storage: &Storage) -> Self {
        self.target = Some(storage.assets_dir.join(path));
        self
    }
    pub fn to_library(mut self, path: &Path, storage: &Storage) -> Self {
        self.target = Some(storage.libraries_dir.join(path));
        self
    }
    pub fn to_path(mut self, path: PathBuf, storage: &Storage) -> Self {
        self.target = Some(path);
        self.bucket = Some(storage.share_dir.clone());
        self
    }
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
    pub fn build(self) -> DownloadTask {
        if self.url.is_none() || self.target.is_none() {
            unreachable!()
        }
        DownloadTask {
            url: self.url.unwrap(),
            target: self.target.unwrap(),
            bucket: self.bucket,
            file_hash: self.file_hash,
            process: DownloadProcess {
                inner: Arc::new(DownloadProcessInner {
                    name: self.file_name,
                    current: AtomicU64::new(0),
                    cancelled: AtomicBool::new(false),
                    event: Default::default(),
                    total: if let Some(t) = self.file_size {
                        let cell = OnceLock::new();
                        cell.set(t).unwrap();
                        cell
                    } else {
                        OnceLock::new()
                    },
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
    pub fn set_total(&self, total: u64) -> bool {
        if self.inner.total.set(total).is_ok() {
            self.inner.event.notify(usize::MAX);
            true
        } else {
            false
        }
    }
    pub fn current(&self) -> u64 {
        self.inner.current.load(Relaxed)
    }
    pub fn step(&self, size: u64) {
        self.inner.current.fetch_add(size, Relaxed);
        self.inner.event.notify(usize::MAX);
    }
    pub async fn changed(&self) {
        self.inner.event.listen().await;
    }
    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Release)
    }
    pub fn is_canceled(&self) -> bool {
        self.inner.cancelled.load(Acquire)
    }
}

pub fn filter_existed(
    tasks: impl Iterator<Item = DownloadTask>,
) -> impl Iterator<Item = DownloadTask> {
    tasks.filter(|x| !x.target.exists())
}
