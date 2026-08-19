pub mod zulu;

use crate::download::Downloader;
use crate::download::task::DownloadTask;
use crate::error::Result;
use std::path::Path;

#[allow(async_fn_in_trait)]
pub trait JavaDownload {
    /// 根据 Major 版本下载 Java
    async fn get_major(
        major: u32,
        downloader: &impl Downloader,
        runtime_dir: &Path,
    ) -> Result<DownloadTask>;
}
