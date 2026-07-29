use futures::StreamExt;
use phanerite_core::debug::DebugClone;
use phanerite_core::download::downloader::Downloader;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::task::DownloadTask;
use phanerite_core::storage::ShareStrategy::Force;
use phanerite_core::utils::{HashValue, Sha1Hash};
use phanerite_core::*;
use sha1::Digest;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::Level;
use url::Url;

async fn hash_file(path: &Path) -> Option<Sha1Hash> {
    let mut hasher = sha1::Sha1::new();
    let mut file = async_fs::File::open(path).await.ok()?;
    let mut buf = vec![0u8; 1024];
    loop {
        let n = file.read(&mut buf).await.ok()?;
        if n == 0 {
            break;
        }
        Digest::update(&mut hasher, &buf[..n])
    }
    Sha1Hash::from_bytes(&Digest::finalize(hasher).0)
}

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

fn main() -> error::Result<()> {
    let _profiler = dhat::Profiler::new_heap();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    smol::block_on(async {
        let storage = storage::Storage::new(".minecraft")?.share_strategy(Force);

        let test_output = PathBuf::new().join("test_output");
        let group = async_fs::read_dir("test_data")
            .await?
            .filter_map(async |x| x.ok())
            .filter_map(async |x| {
                let file_name = x.file_name().to_string_lossy().to_string();
                let meta_data = x.metadata().await.ok()?;
                let task = DownloadTask::builder()
                    .url(Url::parse(&format!("http://127.0.0.1:8080/{}", file_name)).ok()?)
                    .to_path(test_output.join(&file_name))
                    .file_name(file_name)
                    .file_size(meta_data.len())
                    .hash(hash_file(&x.path()).await?)
                    .build();
                Some(task)
            })
            .collect::<DownloadGroup>()
            .await;

        let processes = group.processes();
        let total = processes.total() as f64 / 1024.0 / 1024.0;
        println!("Total size: {:.2} MiB ({} tasks)", total, processes.len());
        let mut result_file = async_fs::File::create("result.txt").await?;

        for i in (32..=64).step_by(4) {
            let _ = async_fs::remove_dir_all("test_output").await;
            async_fs::create_dir_all("test_output").await?;
            let downloader = Downloader::builder(&storage)
                .concurrency(NonZeroU8::new(i).unwrap())
                .build()
                .await?;

            let instant = Instant::now();
            let errs = group.debug_clone().exec(&downloader).await;
            let spend = instant.elapsed().as_secs_f64();

            println!("Errors: {}", errs.len());
            for e in &errs {
                eprintln!("  Error: {}", e);
            }

            let result = format!(
                "Concurrency:{} Download {:.2} MB in {:.2} s, avg:{:.2} MB/s",
                i,
                total,
                spend,
                total / spend
            );
            println!("{result}");
            result_file.write_all(result.as_bytes()).await?;
        }

        Ok::<(), error::Error>(())
    })
}
