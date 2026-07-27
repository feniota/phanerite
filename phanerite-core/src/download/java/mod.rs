pub mod zulu;

use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::storage::Storage;

// 具体实现应该是 ZeroType，不需要担心 Send 问题
#[allow(async_fn_in_trait)]
pub trait JavaDownload {
    async fn get_major(
        &self,
        major: u32,
        downloader: &Downloader,
        storage: &Storage,
    ) -> Result<DownloadTask>;
}
