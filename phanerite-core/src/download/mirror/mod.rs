use futures::StreamExt;

// BMCL API
mod bmclapi;
pub use bmclapi::Bmclapi;

// Granodiorite
mod granodiorite;
pub use granodiorite::Granodiorite;

use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::{Error, Result};
use crate::utils::Hash;
use futures::Stream;
use http::{Request, Response};
use url::Url;

pub trait Mirror {
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

pub struct DownloaderWithMirror<'a, D: Downloader, M: Mirror> {
    downloader: &'a D,
    mirror: M,
}

impl<'a, D: Downloader, M: Mirror> DownloaderWithMirror<'a, D, M> {
    pub(crate) fn new(downloader: &'a D, mirror: M) -> Self {
        Self { downloader, mirror }
    }
}

impl<D: Downloader, M: Mirror> Downloader for DownloaderWithMirror<'_, D, M> {
    async fn fetch(&self, mut url: Url, hash: Option<Hash>) -> Result<Vec<u8>> {
        self.mirror.resolve(&mut url);
        self.downloader.fetch(url, hash).await
    }
    async fn post_json(&self, mut url: Url, body: impl AsRef<str>) -> Result<Response<Vec<u8>>> {
        self.mirror.resolve(&mut url);
        self.downloader.post_json(url, body).await
    }
    async fn head(&self, mut url: Url) -> Result<Response<()>> {
        self.mirror.resolve(&mut url);
        self.downloader.head(url).await
    }
    async fn send(&self, mut req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>> {
        let mut url: Url = req.uri().to_string().parse()?;
        self.mirror.resolve(&mut url);
        *req.uri_mut() = url
            .as_str()
            .parse()
            .map_err(|e| Error::other(format!("mirror resolved to an invalid URI: {e}")))?;
        self.downloader.send(req).await
    }
    async fn download<'cx>(&self, mut task: DownloadTask<'cx>) -> Result<()> {
        self.mirror.resolve_task(&mut task);
        self.downloader.download(task).await
    }
    fn concurrency(&self) -> usize {
        self.downloader.concurrency()
    }
}
