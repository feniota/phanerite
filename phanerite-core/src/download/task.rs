use crate::download::extract::ExtractTask;
use crate::download::task::Target::{Extract, File};
use crate::storage::Storage;
use crate::utils::state::NotReady;
use crate::utils::{EmptyHash, Hash, HashValue, hash_file};
use async_lock::OnceCell;
use event_listener::Event;
use futures::Stream;
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{AtomicU8, AtomicU64};
use std::sync::{Arc, OnceLock};
use url::Url;

#[derive(Debug)]
pub enum Target {
    File(PathBuf),
    Extract(ExtractTask),
}

#[derive(Debug)]
pub struct Context<'cx> {
    pub storage: &'cx Storage,
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

pub struct DownloadTaskBuilder<U, T, C> {
    context: C,
    url: U,
    target: T,
    file_name: Option<String>,
    file_size: Option<u64>,
    file_hash: Hash,
    share: Option<Arc<OnceCell<PathBuf>>>,
}

#[derive(Debug)]
pub struct DownloadTask<'cx> {
    pub(crate) context: Context<'cx>,
    pub(crate) url: Url,
    pub(crate) target: Target,
    pub(crate) file_hash: Hash,
    pub(crate) share: Option<Arc<OnceCell<PathBuf>>>,

    pub process: DownloadProcess,
}

#[derive(Clone, Debug)]
pub struct DownloadProcess {
    inner: Arc<DownloadProcessInner>,
}

#[derive(Debug)]
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

impl DownloadTask<'_> {
    pub fn builder() -> DownloadTaskBuilder<NotReady, NotReady, NotReady> {
        DownloadTaskBuilder {
            context: NotReady,
            url: NotReady,
            target: NotReady,
            share: None,
            file_name: None,
            file_size: None,
            file_hash: Hash::Empty(EmptyHash),
        }
    }
}

impl<T, C> DownloadTaskBuilder<NotReady, T, C> {
    pub fn url(self, url: impl Into<Url>) -> DownloadTaskBuilder<Url, T, C> {
        DownloadTaskBuilder {
            context: self.context,
            url: url.into(),
            target: self.target,
            share: self.share,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
}

impl<U> DownloadTaskBuilder<U, NotReady, NotReady> {
    pub fn to_asset(
        self,
        path: impl AsRef<Path>,
        storage: &Storage,
    ) -> DownloadTaskBuilder<U, PathBuf, Context<'_>> {
        DownloadTaskBuilder {
            context: Context { storage },
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
    ) -> DownloadTaskBuilder<U, PathBuf, Context<'_>> {
        DownloadTaskBuilder {
            context: Context { storage },
            url: self.url,
            target: storage.libraries_dir().join(path),
            share: self.share,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
    pub fn to_path(
        self,
        path: PathBuf,
        storage: &Storage,
    ) -> DownloadTaskBuilder<U, PathBuf, Context<'_>> {
        DownloadTaskBuilder {
            context: Context { storage },
            url: self.url,
            target: path,
            share: self.share,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
    pub fn extract_to(
        self,
        extract_task: ExtractTask,
        storage: &Storage,
    ) -> DownloadTaskBuilder<U, ExtractTask, Context<'_>> {
        DownloadTaskBuilder {
            context: Context { storage },
            url: self.url,
            target: extract_task,
            share: self.share,
            file_name: self.file_name,
            file_size: self.file_size,
            file_hash: self.file_hash,
        }
    }
}

impl<U, P, C> DownloadTaskBuilder<U, P, C> {
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
        self.share = Some(Default::default());
        self
    }
}

impl<'cx, P: Into<Target>> DownloadTaskBuilder<Url, P, Context<'cx>> {
    pub fn build(self) -> DownloadTask<'cx> {
        DownloadTask {
            context: self.context,
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
    // 下载任务未开始
    /// The download task has not started
    pub fn is_pending(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_PENDING
    }
    // 下载任务开始，以及开始后的状态
    /// The download task has started, plus every state after that
    pub fn is_started(&self) -> bool {
        self.inner.state.load(Acquire) >= STATE_STARTED
    }
    // 下载任务正在解压
    /// The download task is extracting
    pub fn is_extracting(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_EXTRACTING
    }
    // 下载任务正常完成
    /// The download task finished normally
    pub fn is_finished(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_FINISHED
    }
    // 下载任务失败
    /// The download task failed
    pub fn is_failed(&self) -> bool {
        self.inner.state.load(Acquire) == STATE_FAILED
    }
    // 下载任务取消
    /// The download task was cancelled
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

// 检验文件存在
/// Checks that the files exist
pub fn filter_existed<'cx>(
    tasks: impl Iterator<Item = DownloadTask<'cx>>,
    default: bool,
) -> impl Iterator<Item = DownloadTask<'cx>> {
    tasks.filter(move |x| {
        if let File(p) = &x.target {
            !p.exists()
        } else {
            default
        }
    })
}

// 检验文件 Hash
/// Checks the files' hashes
pub fn filter_hash<'cx>(
    tasks: impl Stream<Item = DownloadTask<'cx>>,
    default: bool,
) -> impl Stream<Item = DownloadTask<'cx>> {
    tasks
        .map(move |x| async move {
            let invalid = match &x.target {
                File(p) => hash_file(p, &x.file_hash).await.is_err(),
                Extract(_) => default,
            };

            (invalid, x)
        })
        .buffer_unordered(8)
        .filter_map(async |(invalid, x)| invalid.then_some(x))
}
