use crate::download::Downloader;
use crate::download::task::{DownloadProcess, DownloadTask, Target};
use crate::error::{Error, Result};
use crate::storage::temp::TempGuard;
use crate::utils::Hash;
use bytes::Bytes;
use futures::{AsyncReadExt, AsyncWriteExt};
use http::{HeaderMap, Response};
use std::mem::forget;
use std::num::NonZeroU8;
use std::time::Duration;
use tracing::{debug, error};
use trillium_cache::InMemoryStorage;
use trillium_cache::client::Cache;
use trillium_client::Client;
use trillium_client_retry::RetryHandler;
use trillium_compression::client::Compression;
use trillium_logger::ColorMode;
use trillium_logger::client::{ClientLogger, formatters};
use trillium_quinn::ClientQuicConfig;
use trillium_redirect::client::FollowRedirects;
use trillium_rustls::RustlsConfig;
use trillium_smol::ClientConfig;
use url::Url;

const BUFFER_CAPACITY: usize = 2 * 1024 * 1024;

pub struct DownloaderBuilder {
    // 重试次数
    /// Number of retries
    retries: usize,
    // 最大并发数,
    /// Maximum concurrency
    concurrency: usize,
    // UA
    /// User agent
    user_agent: &'static str,
}

impl Default for DownloaderBuilder {
    fn default() -> Self {
        Self {
            retries: 3,
            concurrency: 16,
            // 符合 Modrinth 规范的 User Agent
            user_agent: concat!(
                "feniota/phanerite/",
                env!("CARGO_PKG_VERSION"),
                " (",
                env!("CARGO_PKG_HOMEPAGE"),
                ")"
            ),
        }
    }
}

impl DownloaderBuilder {
    // 设置重试次数
    /// Sets the number of retries
    pub fn retries(mut self, retries: usize) -> Self {
        self.retries = retries;
        self
    }
    // 设置并发度
    /// Sets the concurrency
    pub fn concurrency(mut self, max: NonZeroU8) -> Self {
        self.concurrency = max.get() as usize;
        self
    }
    // 设置 User-Agent
    /// Sets the User-Agent
    pub fn user_agent(mut self, ua: &'static str) -> Self {
        self.user_agent = ua;
        self
    }

    pub async fn build(self) -> Result<RawDownloader> {
        Ok(RawDownloader {
            retries: self.retries,
            concurrency: self.concurrency,
            client: Client::new_with_quic(
                RustlsConfig::<ClientConfig>::default(),
                ClientQuicConfig::with_webpki_roots(),
            )
            .with_handler((
                FollowRedirects::new().with_max_redirects(10),
                Compression::new(),
                Cache::new(InMemoryStorage::default()).shared(),
                RetryHandler::new()
                    .with_exponential_backoff(Duration::from_millis(100))
                    .with_max_attempts(5)
                    .with_max_elapsed(Duration::from_secs(10)),
                ClientLogger::new()
                    .with_formatter((
                        formatters::status,
                        " ",
                        formatters::version,
                        " ",
                        formatters::method,
                        " ",
                        formatters::url,
                        " -> ",
                        formatters::response_time,
                        " ",
                        formatters::body_len_human,
                    ))
                    .with_target(trillium_logger::Target::Logger(
                        tracing_log::log::Level::Info,
                    ))
                    .with_color_mode(ColorMode::Off),
            ))
            .with_default_header("User-Agent", self.user_agent)
            .with_default_header("Accept-Encoding", "zstd, br;q=0.9, gzip;q=0.8"),
        })
    }
}

pub struct RawDownloader {
    retries: usize,
    concurrency: usize,

    // HTTP 客户端
    /// HTTP client
    client: Client,
}

impl Downloader for RawDownloader {
    async fn get(&self, url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        let conn = Self::with_headers(self.client.get(url.as_str()), headers);
        Self::collect_response(conn).await
    }
    async fn get_check(&self, url: Url, hash: Hash) -> Result<Bytes> {
        if hash.is_empty() {
            return Err(Error::other("get_check requires a non-empty hash"));
        }

        for attempt in 0..=self.retries {
            let response = self.get(url.clone(), HeaderMap::new()).await?;
            if !response.status().is_success() {
                return Err(Error::other(format!(
                    "GET {url} returned {}",
                    response.status()
                )));
            }

            let body = response.into_body();
            let mut hasher = hash.hasher();
            hasher.update(&body);
            if hasher.finalize() == hash {
                return Ok(body);
            }

            error!(
                %url,
                attempt = attempt + 1,
                retrying = attempt < self.retries,
                "download hash mismatch"
            );
        }

        Err(Error::other(format!(
            "GET {url} failed hash verification after {} attempts",
            self.retries + 1
        )))
    }
    async fn post(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        let conn = Self::with_headers(self.client.post(url.as_str()).with_body(body), headers);
        Self::collect_response(conn).await
    }
    async fn put(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        let conn = Self::with_headers(self.client.put(url.as_str()).with_body(body), headers);
        Self::collect_response(conn).await
    }
    async fn patch(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        let conn = Self::with_headers(self.client.patch(url.as_str()).with_body(body), headers);
        Self::collect_response(conn).await
    }
    async fn delete(&self, url: Url, headers: HeaderMap, body: Vec<u8>) -> Result<Response<Bytes>> {
        let conn = Self::with_headers(self.client.delete(url.as_str()).with_body(body), headers);
        Self::collect_response(conn).await
    }
    async fn head(&self, url: Url, headers: HeaderMap) -> Result<Response<()>> {
        let conn = Self::with_headers(self.client.build_conn("HEAD", url.as_str()), headers);
        let (parts, _) = Self::collect_response(conn).await?.into_parts();
        Ok(Response::from_parts(parts, ()))
    }
    async fn options(&self, url: Url, headers: HeaderMap) -> Result<Response<Bytes>> {
        let conn = Self::with_headers(self.client.build_conn("OPTIONS", url.as_str()), headers);
        Self::collect_response(conn).await
    }
    async fn download<'cx>(&self, task: DownloadTask<'cx>) -> Result<()> {
        // 用于发送失败信号
        /// Sends the failure signal
        struct FailGuard<'a> {
            process: &'a DownloadProcess,
        }
        impl Drop for FailGuard<'_> {
            fn drop(&mut self) {
                self.process.fail()
            }
        }
        let guard = FailGuard {
            process: &task.process,
        };

        // 准备工作
        debug!(
            "downloading: {}",
            task.process.name().unwrap_or("unknown filename")
        );
        task.process.start();

        // 下载文件
        let mut last_res = Err(Error::other("Unreachable"));
        for _ in 0..=self.retries {
            match self.retry_body(&task).await {
                Ok(v) => {
                    last_res = Ok(v);
                    break;
                }
                Err(Error::Cancelled) => {
                    forget(guard);
                    return Err(Error::Cancelled);
                }
                Err(e) => last_res = Err(e),
            }
        }

        // 保存或解压
        self.post_download(&task, last_res?).await?;

        task.process.finish();
        forget(guard);
        Ok(())
    }

    fn concurrency(&self) -> usize {
        self.concurrency
    }
}

impl RawDownloader {
    async fn collect_response(conn: trillium_client::Conn) -> Result<Response<Bytes>> {
        let mut conn = conn.await?;

        let status = conn.status().unwrap().code();
        let mut builder = Response::builder().status(status);

        for (name, values) in conn.response_headers().iter() {
            for value in values {
                builder = builder.header(name.to_string(), value.as_ref());
            }
        }

        let body = Bytes::from(conn.response_body().read_bytes().await?);

        Ok(builder.body(body).unwrap())
    }
    pub fn builder() -> DownloaderBuilder {
        DownloaderBuilder::default()
    }

    fn with_headers(mut conn: trillium_client::Conn, headers: HeaderMap) -> trillium_client::Conn {
        for (name, value) in headers.iter() {
            conn = conn.with_request_header(name.as_str().to_owned(), value.as_bytes().to_vec());
        }
        conn
    }

    // 重试体
    /// The retried body of the download
    async fn retry_body<'cx>(
        &self,
        task: &DownloadTask<'cx>,
    ) -> Result<(TempGuard<'cx>, Option<blake3::Hash>)> {
        // 创建临时文件
        let cache = task.context.storage.temp_file().await?;

        // 共享储存桶 Hasher
        let mut bucket_hasher = match task.target {
            Target::File(_) => task.share.is_some().then_some(blake3::Hasher::new()),
            // 解压需要单独计算文件 Hash
            Target::Extract(_) => None,
        };

        // 构造和发送请求
        let uri = task.url.as_str();
        let mut res = self.client.get(uri).await?;

        // 补充文件大小（如果不存在）
        if let Ok(len) = res
            .response_headers()
            .get("content-length")
            .and_then(|values| values.as_str())
            .unwrap_or_default()
            .parse()
        {
            task.process.set_total(len);
        }

        // 创建文件
        let mut file = async_fs::File::create(&cache).await?;
        let mut reader = res
            .response_body()
            // Trillium 默认限制响应体为 16 MB
            .with_max_len(u64::MAX);
        let mut hasher = task.file_hash.hasher();
        let mut buf = vec![0u8; BUFFER_CAPACITY];

        // 流式下载
        loop {
            let n = reader.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            if task.process.is_canceled() {
                async_fs::remove_file(&cache).await?;
                return Err(Error::Cancelled);
            }
            file.write_all(&buf[..n]).await?;

            hasher.update(&buf[..n]);
            if let Some(ref mut h) = bucket_hasher {
                h.update(&buf[..n]);
            }
            task.process.step(n as u64);
        }
        file.flush().await?;
        drop(buf);
        drop(reader);
        drop(res);
        drop(file);

        // 校验文件
        if task.file_hash == hasher.finalize() {
            Ok((cache, bucket_hasher.map(|t| blake3::Hasher::finalize(&t))))
        } else {
            Err(Error::other(format!(
                "hash mismatch: {}",
                task.process.name().unwrap_or("Unknow file")
            )))
        }
    }
    // 下载后的保存和解压
    /// Saving and extracting after the download
    async fn post_download<'cx>(
        &self,
        task: &DownloadTask<'cx>,
        (cache, bucket_hash): (TempGuard<'cx>, Option<blake3::Hash>),
    ) -> Result<()> {
        match &task.target {
            // 直接保存
            Target::File(path) => {
                // 确定保存路径
                let save_path = if task.share.is_some() {
                    &task.context.storage.share_path(&bucket_hash.unwrap())
                } else {
                    path
                };

                // 确保父目录存在
                let parent = save_path.parent().expect("invalid save path");
                if !parent.is_dir() {
                    async_fs::create_dir_all(parent).await?;
                }

                // 移动到目标位置
                // 共享文件不覆盖避免破坏硬链接
                if !(task.share.is_some() && save_path.exists()) {
                    async_fs::rename(&cache, &save_path).await?
                }

                // 如果有 bucket，将共享文件链接到 task.target
                if let Some(record) = &task.share {
                    task.context.storage.linker()(save_path, path).await?;

                    // 初始化 Task 的共享位置（用于外部记录）
                    let _ = record.set(bucket_hash.unwrap()).await;
                }
            }
            // 解压缩
            Target::Extract(extract) => {
                task.process.extracting();
                extract
                    .exec(&cache, task.share.is_some(), task.context.storage)
                    .await?
            }
        }
        Ok(())
    }
}
