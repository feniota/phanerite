use crate::download::Downloader;
use crate::download::task::{DownloadProcess, DownloadTask};
use crate::error::{Error, Result};
use crate::utils::Hash;
use futures::{Stream, StreamExt};
use http::{HeaderMap, StatusCode};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use url::Url;

pub struct DownloadGroup<'a, D: Downloader> {
    downloader: &'a D,
    stage: Vec<DownloadTask>,
    monitor: Monitor,
}

#[derive(Clone, Default)]
pub struct Monitor {
    inner: Arc<MonitorInner>,
}

#[derive(Default)]
struct MonitorInner {
    max_id: AtomicUsize,
    processes: scc::HashMap<usize, DownloadProcess>,
}

/// 批量添加任务到暂存区
impl<D: Downloader> Extend<DownloadTask> for DownloadGroup<'_, D> {
    fn extend<T: IntoIterator<Item = DownloadTask>>(&mut self, iter: T) {
        self.stage.extend(iter)
    }
}

impl<'a, D: Downloader> DownloadGroup<'a, D> {
    pub(crate) fn new(downloader: &'a D) -> Self {
        Self {
            downloader,
            stage: vec![],
            monitor: Default::default(),
        }
    }
    /// 获取监视器
    pub fn monitor(&self) -> Monitor {
        self.monitor.clone()
    }
    /// 立即执行任务
    pub async fn join(&self, tasks: impl IntoIterator<Item = DownloadTask>) -> Vec<Error> {
        self.download_concurrent(futures::stream::iter(tasks))
            .filter_map(async |x| x.err())
            .collect()
            .await
    }
    /// 添加任务到暂存区
    pub fn push(&mut self, task: DownloadTask) {
        self.stage.push(task)
    }
    /// 执行暂存区的任务
    pub async fn exec(&mut self) -> Vec<Error> {
        let tasks = std::mem::take(&mut self.stage);
        self.join(tasks).await
    }
}

impl Monitor {
    async fn push_async(&self, process: DownloadProcess) {
        let id = self.inner.max_id.fetch_add(1, Relaxed);
        let _ = self.inner.processes.insert_async(id, process).await;
    }
    pub fn len(&self) -> usize {
        self.inner.processes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.processes.is_empty()
    }
    pub async fn total(&self) -> u64 {
        let mut total = 0;
        self.inner
            .processes
            .iter_async(|_, x| {
                total += x.total().unwrap_or_default();
                true
            })
            .await;
        total
    }
    pub async fn current(&self) -> u64 {
        let mut current = 0;
        self.inner
            .processes
            .iter_async(|_, x| {
                current += x.current();
                true
            })
            .await;
        current
    }
    pub async fn downloading(&self) -> usize {
        let mut count = 0;
        self.inner
            .processes
            .iter_async(|_, x| {
                if x.is_started() && !x.is_finished() && !x.is_canceled() && !x.is_failed() {
                    count += 1
                }
                true
            })
            .await;
        count
    }
    pub async fn finished(&self) -> usize {
        let mut count = 0;
        self.inner
            .processes
            .iter_async(|_, x| {
                if x.is_finished() {
                    count += 1;
                }
                true
            })
            .await;
        count
    }
    pub async fn is_finished(&self) -> bool {
        self.inner
            .processes
            .iter_async(|_, x| x.is_finished() || x.is_failed() || x.is_canceled())
            .await
    }
    pub async fn speed_by_timer(&self, timer: impl Future) -> u64 {
        let start = self.current().await;
        timer.await;
        self.current().await - start
    }
}

impl<D: Downloader> Downloader for DownloadGroup<'_, D> {
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Vec<u8>> {
        self.downloader.fetch(url, hash).await
    }
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<(StatusCode, Vec<u8>)> {
        self.downloader.post_json(url, body).await
    }
    async fn head(&self, url: Url) -> Result<HeaderMap> {
        self.downloader.head(url).await
    }
    async fn download(&self, task: DownloadTask) -> Result<()> {
        self.monitor.push_async(task.process.clone()).await;
        self.downloader.download(task).await
    }
    fn concurrency(&self) -> usize {
        self.downloader.concurrency()
    }
    fn download_concurrent(
        &self,
        tasks: impl Stream<Item = DownloadTask>,
    ) -> impl Stream<Item = Result<()>> {
        enum CollectState<S, I>
        where
            S: Stream<Item = DownloadTask>,
            I: Iterator<Item = DownloadTask>,
        {
            Collect(S),
            Ready(I),
        }
        futures::stream::unfold(
            CollectState::<_, std::vec::IntoIter<DownloadTask>>::Collect(tasks),
            async |state| match state {
                CollectState::Collect(s) => {
                    let mut iter = s
                        .then(async |x| {
                            self.monitor.push_async(x.process.clone()).await;
                            x
                        })
                        .collect::<Vec<_>>()
                        .await
                        .into_iter();
                    iter.next().map(|t| (t, CollectState::Ready(iter)))
                }
                CollectState::Ready(mut i) => i.next().map(|t| (t, CollectState::Ready(i))),
            },
        )
        .map(async |x| self.downloader.download(x).await)
        .buffer_unordered(self.concurrency())
    }
}
