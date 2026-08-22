// README 中的快速开始示例，两处需要保持一致
//
// 存在的意义主要是让 `cargo build --examples` 编译它，API 变动时立刻失败，
// 而不是放任 README 里的代码悄悄失效
//! The quick-start example from the README; the two are kept in sync
//!
//! It exists mainly so that `cargo build --examples` compiles it, so an API
//! change fails the build instead of letting the code in the README rot

use phanerite_core::download::DownloaderExt;
use phanerite_core::download::downloader::RawDownloader;
use phanerite_core::download::vanilla::VersionIndex;
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::storage::Storage;
use std::collections::HashSet;

fn main() -> Result<(), Error> {
    smol::block_on(async {
        // 数据存储位置
        let storage = Storage::new(".minecraft").await?;

        // 基础下载器。通常创建一次后持续复用
        let raw = RawDownloader::builder().build().await?;

        // 任务组只负责跟踪这一批任务的执行状态
        let downloader = raw.with_group();

        // 获取要安装的版本
        let manifest = VersionIndex::sync(&downloader)
            .await?
            .latest_release()?
            .get_manifest(&downloader)
            .await?;

        let instance = Instance::create(manifest, Some("latest"), &storage, &downloader).await?;

        // install 只生成任务，实际执行由下载器负责
        downloader
            .join(instance.install(HashSet::new()).await?)
            .await
            .iter()
            .for_each(|e| eprintln!("{e}"));

        Ok::<(), Error>(())
    })
}
