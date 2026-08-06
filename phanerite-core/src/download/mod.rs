use crate::download::task::DownloadTask;
use crate::error::{Error, Result};
use crate::utils::Hash;
use futures::Stream;
use http::{HeaderMap, StatusCode};
use url::Url;

pub mod authlib_injector;
pub mod downloader;
pub mod extract;
pub mod group;
pub mod java;
pub mod mirror;
pub mod task;
pub mod vanilla;

#[allow(async_fn_in_trait)]
pub trait Downloader {
    /// 下载到内存（GET）
    async fn fetch(&self, url: Url, hash: Option<Hash>) -> Result<Vec<u8>>;
    /// 封装 POST
    async fn post_json(&self, url: Url, body: impl AsRef<str>) -> Result<(StatusCode, Vec<u8>)>;
    /// 封装 HEAD
    async fn head(&self, url: Url) -> Result<HeaderMap>;
    /// 下载文件到储存
    async fn download(&self, task: DownloadTask) -> Result<()>;
    /// 并发下载文件到储存
    async fn download_concurrent(
        &self,
        tasks: impl Stream<Item = DownloadTask>,
    ) -> impl Stream<Item = Error>;
}
