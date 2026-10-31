use bytes::Bytes;
use futures::StreamExt;
use std::borrow::Borrow;
use std::marker::PhantomData;

// BMCL API
mod bmclapi;
pub use bmclapi::Bmclapi;

// Granodiorite
mod granodiorite;
pub use granodiorite::Granodiorite;

use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::utils::Hash;
use futures::Stream;
use http::{HeaderMap, Response};
use url::Url;

pub trait Mirror: Send + Sync {
    const NAME: &str;
    const ATTRIBUTION: &str;
    const NOTICE: &str;
    fn resolve(&self, url: &mut Url);
    fn resolve_task(&self, task: &mut DownloadTask) {
        self.resolve(&mut task.url)
    }
    fn resolve_all<'cx>(
        &self,
        tasks: impl Iterator<Item = DownloadTask<'cx>>,
    ) -> impl Iterator<Item = DownloadTask<'cx>> {
        tasks.map(|mut x| {
            self.resolve_task(&mut x);
            x
        })
    }
    fn resolve_stream<'cx>(
        &self,
        tasks: impl Stream<Item = DownloadTask<'cx>>,
    ) -> impl Stream<Item = DownloadTask<'cx>> {
        tasks.map(|mut x| {
            self.resolve_task(&mut x);
            x
        })
    }
}

pub struct DownloaderWithMirror<D: Downloader, B: Borrow<D> + Send + Sync, M: Mirror> {
    mirror: M,

    downloader: B,
    _marker: PhantomData<D>,
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, M: Mirror> DownloaderWithMirror<D, B, M> {
    pub fn new(downloader: B, mirror: M) -> Self {
        Self {
            downloader,
            mirror,
            _marker: Default::default(),
        }
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, M: Mirror> Downloader
    for DownloaderWithMirror<D, B, M>
{
    async fn get(&self, mut url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().get(url, headers).await
    }
    async fn get_check(&self, mut url: Url, hash: Hash) -> Result<Bytes> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().get_check(url, hash).await
    }
    async fn post(
        &self,
        mut url: Url,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().post(url, headers, body).await
    }
    async fn put(
        &self,
        mut url: Url,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().put(url, headers, body).await
    }
    async fn patch(
        &self,
        mut url: Url,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().patch(url, headers, body).await
    }
    async fn delete(
        &self,
        mut url: Url,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<Response<Bytes>> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().delete(url, headers, body).await
    }
    async fn head(&self, mut url: Url, headers: HeaderMap) -> Result<Response<()>> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().head(url, headers).await
    }
    async fn options(&self, mut url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        self.mirror.resolve(&mut url);
        self.downloader.borrow().options(url, headers).await
    }
    async fn download<'cx>(&self, mut task: DownloadTask<'cx>) -> Result<()> {
        self.mirror.resolve_task(&mut task);
        self.downloader.borrow().download(task).await
    }
    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
}
