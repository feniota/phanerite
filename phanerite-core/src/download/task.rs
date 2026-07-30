use crate::download::extract::ExtractTask;
use crate::download::task::Target::{Extract, File};
use crate::error::{Error, Result};
use crate::storage::Storage;
use crate::utils::{EmptyHash, Hash, HashValue};
use event_listener::Event;
use futures::StreamExt;
use futures::{AsyncReadExt, Stream};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicU8, AtomicU64};
use std::sync::{Arc, OnceLock};
use url::Url;

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
    share: bool,
    file_name: Option<String>,
    file_size: Option<u64>,
    file_hash: Hash,
}

pub struct DownloadTask {
    pub(crate) url: Url,
    pub(crate) target: Target,
    pub(crate) share: bool,
    pub(crate) file_hash: Hash,

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

    state: AtomicU8,
}

const STATE_PENDING: u8 = 0;
const STATE_STARTED: u8 = 1;
const STATE_EXTRACTING: u8 = 2;
const STATE_FINISHED: u8 = 3;
const STATE_FAILED: u8 = 4;
const STATE_CANCELLED: u8 = 5;

impl DownloadTask {
    pub fn builder() -> DownloadTaskBuilder<Missing, Missing> {
        DownloadTaskBuilder {
            url: Missing,
            target: Missing,
            share: false,
            file_name: None,
            file_size: None,
            file_hash: Hash::Empty(EmptyHash),
        }
    }
}

impl<T> DownloadTaskBuilder<Missing, T> {
    pub fn url(self, url: impl Into<Url>) -> DownloadTaskBuilder<Url, T> {
        DownloadTaskBuilder {
            url: url.into(),
            target: self.target,
            share: self.share,
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
            share: self.share,
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
            share: self.share,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
    pub fn to_path(self, path: PathBuf) -> DownloadTaskBuilder<U, PathBuf> {
        DownloadTaskBuilder {
            url: self.url,
            target: path,
            share: self.share,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
    pub fn extract_to(self, extract_task: ExtractTask) -> DownloadTaskBuilder<U, ExtractTask> {
        DownloadTaskBuilder {
            url: self.url,
            target: extract_task,
            share: self.share,
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
    pub fn share(mut self) -> Self {
        self.share = true;
        self
    }
}

impl<P: Into<Target>> DownloadTaskBuilder<Url, P> {
    pub fn build(self) -> DownloadTask {
        DownloadTask {
            url: self.url,
            target: self.target.into(),
            share: self.share,
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

                    state: AtomicU8::new(STATE_PENDING),
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
        self.inner.state.store(STATE_CANCELLED, Release);
        self.inner.event.notify(usize::MAX);
    }
    /// 下载任务未开始
    pub fn is_pending(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_PENDING
    }
    /// 下载任务开始，以及开始后的状态
    pub fn is_started(&self) -> bool {
        self.inner.state.load(Acquire) >= STATE_STARTED
    }
    /// 下载任务正在解压
    pub fn is_extracting(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_EXTRACTING
    }
    /// 下载任务正常完成
    pub fn is_finished(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_FINISHED
    }
    /// 下载任务失败
    pub fn is_failed(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_FAILED
    }
    /// 下载任务取消
    pub fn is_canceled(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_CANCELLED
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
        self.inner.state.store(STATE_STARTED, Release);
        self.inner.event.notify(usize::MAX);
    }
    pub(super) fn extracting(&self) {
        self.inner.state.store(STATE_EXTRACTING, Release);
        self.inner.event.notify(usize::MAX);
    }
    pub(super) fn finish(&self) {
        self.inner.state.store(STATE_FINISHED, Release);
        self.inner.event.notify(usize::MAX);
    }
    pub(super) fn fail(&self) {
        self.inner.state.store(STATE_FAILED, Release);
        self.inner.event.notify(usize::MAX);
    }
}

/// 检验文件存在，压缩包默认存在
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

/// 检验文件存在，压缩包默认失效
pub fn filter_hash(tasks: impl Stream<Item = DownloadTask>) -> impl Stream<Item = DownloadTask> {
    tasks
        .map(async |x| {
            let invalid = match &x.target {
                File(p) => hash_file(p, &x.file_hash).await.is_err(),
                Extract(_) => true,
            };

            (invalid, x)
        })
        .buffer_unordered(8)
        .filter_map(async |(invalid, x)| invalid.then_some(x))
}

async fn hash_file(path: &Path, hash: &Hash) -> Result<()> {
    let mut file = async_fs::File::open(path).await?;
    let mut buffer = vec![0u8; 128 * 1024];
    let mut hasher = hash.hasher();
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n])
    }
    if hasher.finalize() == *hash {
        Ok(())
    } else {
        Err(Error::other("hash mismatch"))
    }
}
