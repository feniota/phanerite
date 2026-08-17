use futures::StreamExt;

// BMCL API
mod bmclapi;
pub use bmclapi::Bmclapi;

// Granodiorite
mod granodiorite;
pub use granodiorite::Granodiorite;

use crate::download::Downloader;
use crate::download::downloader::RawDownloader;
use crate::download::task::DownloadTask;
use crate::error::Error;
use crate::error::Result;
use crate::utils::Hash;
use futures::Stream;
use http::{HeaderMap, StatusCode};
use url::Url;

pub trait Mirror {
    const NAME: &str;
    const ATTRIBUTION: &str;
    const NOTICE: &str;
    fn resolve(&self, url: &mut Url);
    fn resolve_task(&self, task: &mut DownloadTask) {
        self.resolve(&mut task.url)
    }
    fn resolve_all(
        &self,
        tasks: impl Iterator<Item = DownloadTask>,
    ) -> impl Iterator<Item = DownloadTask> {
        tasks.map(|mut x| {
            self.resolve_task(&mut x);
            x
        })
    }
    fn resolve_stream(
        &self,
        tasks: impl Stream<Item = DownloadTask>,
    ) -> impl Stream<Item = DownloadTask> {
        tasks.map(|mut x| {
            self.resolve_task(&mut x);
            x
        })
    }
}

pub struct DownloaderWithMirror<'a, M: Mirror> {
    downloader: &'a RawDownloader<'a>,
    mirror: M,
}

impl<'a> RawDownloader<'a> {
    pub fn with_mirror<M: Mirror>(&'a self, mirror: M) -> DownloaderWithMirror<'a, M> {
        DownloaderWithMirror {
            downloader: self,
            mirror,
        }
    }
}

impl<M: Mirror> Downloader for DownloaderWithMirror<'_, M> {
    async fn fetch(&self, mut url: Url, hash: Option<Hash>) -> Result<Vec<u8>> {
        self.mirror.resolve(&mut url);
        self.downloader.fetch(url, hash).await
    }

    async fn post_json(
        &self,
        mut url: Url,
        body: impl AsRef<str>,
    ) -> Result<(StatusCode, Vec<u8>)> {
        self.mirror.resolve(&mut url);
        self.downloader.post_json(url, body).await
    }

    async fn head(&self, mut url: Url) -> Result<HeaderMap> {
        self.mirror.resolve(&mut url);
        self.downloader.head(url).await
    }

    async fn download(&self, mut task: DownloadTask) -> Result<()> {
        self.mirror.resolve_task(&mut task);
        self.downloader.download(task).await
    }

    async fn download_concurrent(
        &self,
        tasks: impl Stream<Item = DownloadTask>,
    ) -> impl Stream<Item = Error> {
        self.downloader
            .download_concurrent(self.mirror.resolve_stream(tasks))
            .await
    }
}
