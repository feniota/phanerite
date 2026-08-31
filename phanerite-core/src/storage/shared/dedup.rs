use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::storage::shared::registry::HashRegistry;
use crate::utils::{Hash, hash_file};
use bytes::Bytes;
use http::{Request, Response};
use std::borrow::Borrow;
use url::Url;

pub struct DeduplicateDownloader<D, B, R>
where
    D: Downloader,
    B: Borrow<D> + Send + Sync,
    R: HashRegistry,
{
    downloader: B,
    registry: R,

    _marker: std::marker::PhantomData<D>,
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: HashRegistry> DeduplicateDownloader<D, B, R> {
    pub fn new(downloader: B, registry: R) -> Self {
        Self {
            downloader,
            registry,
            _marker: Default::default(),
        }
    }
}

impl<D: Downloader, B: Borrow<D> + Send + Sync, R: HashRegistry> Downloader
    for DeduplicateDownloader<D, B, R>
{
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
        let (hash, dst, share) = match (&task.file_hash, &task.target, &task.share) {
            (hash, crate::download::task::Target::File(dst), Some(share)) if !hash.is_empty() => {
                (hash, dst, share.clone())
            }
            _ => {
                self.downloader.borrow().download(task).await?;
                return Ok(());
            }
        };

        // 命中缓存
        if let Some(src) = self
            .registry
            .get(hash)
            .await
            .map(|t| task.context.storage.share_path(&t))
            && hash_file(&src, hash).await.is_ok()
        {
            task.context.storage.linker()(&src, dst).await?;
            return Ok(());
        }

        // 未命中缓存
        let hash = hash.clone();
        self.downloader.borrow().download(task).await?;
        // 记录地址
        if let Some(blake3) = share.get() {
            let _ = self.registry.insert(*blake3, hash).await;
        };

        Ok(())
    }

    fn concurrency(&self) -> usize {
        self.downloader.borrow().concurrency()
    }
}
