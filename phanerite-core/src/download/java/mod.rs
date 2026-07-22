pub mod zulu;

use crate::download::downloader::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use crate::storage::Storage;

pub trait JavaDownload {
    async fn get_major(
        &self,
        major: u32,
        downloader: &Downloader,
        storage: &Storage,
    ) -> Result<DownloadTask>;
}
