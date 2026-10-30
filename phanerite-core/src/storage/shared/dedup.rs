use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::storage::shared::HashRegistry;
use crate::utils::{Hash, hash_file};
use bytes::Bytes;
use http::{HeaderMap, Response};
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
    async fn get(&self, url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        self.downloader.borrow().get(url, headers).await
    }
    async fn get_check(&self, url: Url, hash: Hash) -> Result<Bytes> {
        self.downloader.borrow().get_check(url, hash).await
    }
    async fn post(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().post(url, headers, body).await
    }
    async fn put(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().put(url, headers, body).await
    }
    async fn patch(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().patch(url, headers, body).await
    }
    async fn delete(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        self.downloader.borrow().delete(url, headers, body).await
    }
    async fn head(&self, url: Url, headers: HeaderMap) -> Result<Response<()>> {
        self.downloader.borrow().head(url, headers).await
    }
    async fn options(&self, url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        self.downloader.borrow().options(url, headers).await
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
