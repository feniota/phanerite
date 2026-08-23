use crate::download::Downloader;
use crate::download::task::{DownloadProcess, DownloadTask};
use crate::error::{Error, Result};
use crate::utils::Hash;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use http::{Request, Response};
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use url::Url;

pub struct DownloadGroup<D: Downloader, B: Borrow<D> + Send + Sync> {
    monitor: Monitor,

    downloader: B,
    _marker: PhantomData<fn() -> D>,
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

impl<D: Downloader, B: Borrow<D> + Send + Sync> Downloader for DownloadGroup<D, B> {
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Bytes> {
        self.downloader.borrow().fetch(url, hash).await
    }
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<Response<Bytes>> {
        self.downloader.borrow().post_json(url, body).await
    }
    async fn head(&self, url: Url) -> Result<Response<()>> {
        self.downloader.borrow().head(url).await
    }
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Bytes>> {
        self.downloader.borrow().send(req).await
    }
    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()> {
        self.monitor.push_async(task.process.clone()).await;
        self.downloader.borrow().download(task).await
    }
    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
    fn download_concurrent<'cx>(
        &self,
        tasks: impl IntoIterator<Item = DownloadTask<'cx>>,
    ) -> impl Stream<Item = Result<()>> {
        enum CollectState<'a, C, R>
        where
            C: IntoIterator<Item = DownloadTask<'a>>,
            R: Iterator<Item = DownloadTask<'a>>,
        {
            Collect(C),
            Ready(R),
        }
        futures::stream::unfold(
            CollectState::<_, std::vec::IntoIter<DownloadTask>>::Collect(tasks),
            async |state| match state {
                CollectState::Collect(s) => {
                    let mut iter = futures::stream::iter(s)
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
        .map(async |x| self.downloader.borrow().download(x).await)
        .buffer_unordered(self.concurrency())
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync> DownloadGroup<D, B> {
    pub fn new(downloader: B) -> Self {
        Self {
            monitor: Default::default(),

            downloader,
            _marker: Default::default(),
        }
    }
    // 获取监视器
    /// Gets the monitor
    pub fn monitor(&self) -> Monitor {
        self.monitor.clone()
    }
    // 立即执行任务
    /// Runs the tasks immediately
    pub async fn join<'cx>(
        &self,
        tasks: impl IntoIterator<Item = DownloadTask<'cx>>,
    ) -> Vec<Error> {
        self.download_concurrent(tasks)
            .filter_map(async |x| x.err())
            .collect()
            .await
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
