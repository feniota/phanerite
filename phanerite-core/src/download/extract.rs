use crate::error::Result;
use std::path::{Path, PathBuf};

pub enum ArchiveFormat {
    Zip,
    Tar,
}

pub struct ExtractTask {
    // 解压目标目录
    path: PathBuf,
    // 压缩包格式
    format: ArchiveFormat,

    // 自动拉平压缩包外层目录
    auto_flattens: bool,
}

impl ExtractTask {
    pub(super) async fn exec(
        &self,
        archive_file: impl AsRef<Path>,
        _bucket: Option<PathBuf>,
        buf: &mut [u8],
    ) -> Result<()> {
        todo!()
    }
}
