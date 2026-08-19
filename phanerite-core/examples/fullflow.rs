use async_executor::Executor;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::java::zulu::Zulu;
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::download::{Downloader, DownloaderExt};
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::runtime::java::JavaManager;
use phanerite_core::storage::SharePreference::Hardlink;
use phanerite_core::storage::multi::MultiStorage;
use phanerite_core::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::{Duration, Instant};
use tracing::{Level, error};
use url::Url;

fn main() {
    // （登录信息）
    let _ = dotenvy::dotenv();
    // 日志输出
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    // （用于测试阻塞时间）
    let executor = Executor::new();
    let _guard = BlockingGuard::new(&executor);

    // 异步 Runtime
    if let Err(e) = smol::block_on(executor.run(async {
        // 构造 Storage
        //
        // 由 MultiStorage 管理 Storage，MultiStorage 可用于全局配置
        let storages = MultiStorage::new();
        // 创建 Storage
        let storage = storage::Storage::new(".minecraft")
            .await?
            .share_preference(Hardlink);
        // 生成临时文件清理任务
        let (cleaner, _shutdown) = storage.run_cleaner();
        smol::spawn(cleaner).detach();
        // Storage 移动至 MultiStorage 容器
        storages.insert(storage).await.unwrap();
        // 通过 ID 在局部获取 &Storage
        let id = storages.iter(|mut iter| iter.next().map(|(id, _)| *id).unwrap());
        let storage = storages.get(&id).unwrap();

        // 构造 Downloader
        //
        // 基本下载器，可以全局创建一次（内部有并发限制）
        let raw_downloader = download::downloader::RawDownloader::builder(&storage)
            .build()
            .await?;
        // 下载缓存，建议保持尽可能长的生命周期
        let cached_downloader = raw_downloader.with_cache_default();
        // 任务组，应该一次性使用
        let downloader = cached_downloader.with_group();
        // （进度监视器）
        let _g = monitor(&downloader).await;

        // 构造 JavaManager
        let java_manager =
            // 可以使用任何实现 RuntimeScanPath 的类型，例如 Storage 和 MultiStorage
            JavaManager::new(&storages)
            .await
            // 默认启用系统的 Java 运行时检测，此处关闭用于测试
            .disable_system();

        // 创建登录凭据
        let auth =
            // 此处使用 Yggdrasil 登录
            auth::yggdrasil::Authentication::new_login(&downloader)
            // 注入 Authlib-Injector
            .inject(&storage)
            .await?
            // 自定义 Yggdrasil 地址
            .custom("https://aphanite.enita.cn/api/yggdrasil".parse::<Url>()?)
            .await?
            // 用户名（一般为邮箱，服务器支持则可以为游戏角色名）
            .username(
                std::env::var("USERNAME")
                    .expect("Fill in the login credentials in the environment variable"),
            )
            // 用户密码
            .password(
                std::env::var("PASSWORD")
                    .expect("Fill in the login credentials in the environment variable"),
            )
            // 发送登录请求
            .login()
            .await?;

        // （清理测试残留）
        let _ = async_fs::remove_dir_all(storage.versions_dir().join("latest")).await;

        // 创建实例
        //
        // 获取版本清单
        let version =
            // 下载版本索引
            VersionIndex::sync(&downloader)
            .await?
            // 使用迭代器查找需要的版本
            // .iter()
            // .find(|x| x.id == "1.21.1")
            // .expect("Version not found")
            // 使用最新的稳定版
            .latest_release()?
            // 下载版本清单
            .get_manifest(&downloader)
            .await?;
        // 创初始化实例
        let instance = Instance::create(version, Some("latest"), &storage, &downloader).await?;

        // 安装 Java
        let java = java_manager
            // 获取符合 major 版本的 JavaRuntime，若不存在则安装，需要传入闭包选择安装位置（需要保证选择的位置在扫描范围内）
            .get_or_install::<Zulu>(instance.java_major(), &downloader, async |x| {
                // 对于 x: Storage，必须为 x.runtime_dir()
                x.get(&id).unwrap().runtime_dir().to_owned()
            })
            .await?;
        // 为实例绑定 JavaRuntime，安装模组加载器或启动游戏需要此状态
        let instance = instance.bind_java(java.clone()).await?;

        // // 安装模组加载器
        // instance
        //      // 安装需要正确的游戏版本 ID，并在闭包内选择需要的加载器版本
        //     .install_loader::<NeoForge>("1.21.1", &downloader, async |iter| {
        //         // （调试输出）
        //         // let iter = iter
        //         //     .inspect(|x| println!("{}:{} stable:{}", x.name(), x.version(), x.stable()));
        //         // 排序和版本选择，元素对版本号实现全序，但是仅人类可读，不一定为正确的版本顺序
        //         let latest = iter
        //             .collect::<BTreeSet<_>>()
        //             .pop_last()
        //             .expect("No available loader version");
        //         // println!(
        //         //     "{}:{} stable:{}",
        //         //     latest.name(),
        //         //     latest.version(),
        //         //     latest.stable()
        //         // );
        //         Ok(latest)
        //     })
        //     .await?;

        // 安装实例
        downloader
            .join(instance.install_less(HashSet::new()).await?)
            .await
            .iter()
            .for_each(|e| error!("{e}"));

        // 启动游戏
        //
        // 声明游戏完整性（不提供检查），启动游戏需要此状态
        let instance = instance.ensure_ready();
        // 创建启动命令
        let mut cmd = instance.launch(&auth).await?;
        // 启动进程并等待退出
        let exit = cmd.spawn()?.status().await?;

        println!("Game exited: {exit}");

        Ok::<(), Error>(())
    })) {
        error!("{}", e)
    }
}

struct ExitGuard {
    exit: Arc<AtomicBool>,
}

impl Drop for ExitGuard {
    fn drop(&mut self) {
        self.exit.store(true, Relaxed)
    }
}

/// 显示下载速度和进度
async fn monitor(group: &DownloadGroup<'_, impl Downloader>) -> ExitGuard {
    let monitor = group.monitor();
    let g = ExitGuard {
        exit: Arc::new(AtomicBool::new(false)),
    };
    let exit = g.exit.clone();
    smol::spawn(async move {
        while !exit.load(Relaxed)  {
            let downloading = monitor.downloading().await;
            let number = monitor.len();
            let speed = monitor
                .speed_by_timer(smol::Timer::after(Duration::from_secs(1)))
                .await;
            let current = monitor.current().await;
            let total = monitor.total().await;
            let pct = if total > 0 {
                current as f64 / total as f64 * 100.0
            } else {
                0.0
            };
            let finished = monitor.finished().await;
            println!(
                "Progress: {pct:.1}% ({finished}/{number} finished) Downloading: {downloading}  {:.2} MiB/s",
                speed as f64 / 1024.0 / 1024.0,
            );
        }
    })
        .detach();
    g
}

/// 检测阻塞时间
pub struct BlockingGuard {
    stopped: Arc<AtomicBool>,
    max_blocking_ns: Arc<AtomicU64>,
}

impl BlockingGuard {
    pub fn new(executor: &Executor<'static>) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let max_blocking_ns = Arc::new(AtomicU64::new(0));

        let stopped2 = stopped.clone();
        let max_blocking_ns2 = max_blocking_ns.clone();

        executor
            .spawn(async move {
                let mut last = Instant::now();

                loop {
                    smol::Timer::after(Duration::from_millis(1)).await;

                    let now = Instant::now();
                    let elapsed = now.duration_since(last);
                    last = now;

                    // Timer 本身也可能因为 worker 被 blocking 而延迟执行
                    let ns = elapsed.as_nanos() as u64;

                    max_blocking_ns2.fetch_max(ns, Relaxed);

                    if stopped2.load(Relaxed) {
                        break;
                    }
                }
            })
            .detach();

        Self {
            stopped,
            max_blocking_ns,
        }
    }
}

impl Drop for BlockingGuard {
    fn drop(&mut self) {
        self.stopped.store(true, Relaxed);

        let max = Duration::from_nanos(self.max_blocking_ns.load(Relaxed));

        eprintln!("max blocking: {:.3} ms", max.as_secs_f64() * 1000.0,);
    }
}
