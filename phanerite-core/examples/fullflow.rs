use phanerite_core::auth::{self, Account};
use phanerite_core::auth::{Authentication, MultiAccount};
use phanerite_core::download::downloader::RawDownloader;
use phanerite_core::download::group::DownloadGroup;
use phanerite_core::download::java::Zulu;
use phanerite_core::download::vanilla::VersionIndex;
use phanerite_core::download::{Downloader, DownloaderExt};
use phanerite_core::error::Error;
use phanerite_core::instance::Instance;
use phanerite_core::mod_loader::neoforge::NeoForge;
use phanerite_core::runtime::java::JavaManager;
use phanerite_core::storage::SharePreference::Hardlink;
use phanerite_core::storage::Storage;
use phanerite_core::storage::multi::{MultiStorageWithPlugin, StorageWithPlugin};

fn main() {
    // （登录信息）
    let _ = dotenvy::dotenv();
    // 日志输出
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    // （用于测试阻塞时间）
    let executor = async_executor::Executor::new();
    let _guard = blocking_monitor(&executor);

    // 异步 Runtime
    if let Err(e) = smol::block_on(executor.run(async {
        // ———————————————————— 全局的初始化阶段 ————————————————————

        // 构造 Downloader

        // 基本下载器，可以全局创建一次（内部有并行限制）
        let raw_downloader = RawDownloader::builder().build().await?;

        // 文件去重，建议保持尽可能长的生命周期
        // 这里只使用内存记录器，仅适合快速验证或演示；实际使用时应通过 with_dedup()
        // 传入可持久化的 `phanerite_core::download::dedup::StorageRegistry`。
        let dedup_downloader = raw_downloader.with_in_memory_dedup();

        // GET 响应缓存，建议保持尽可能长的生命周期
        // 这里只缓存 fetch() 的响应，不负责下载文件的去重；内存缓存仅适合快速验证或演示。
        // 实际使用时可通过 with_cache() 传入自定义的 FetchCache。
        #[cfg(feature = "moka")]
        let cached_downloader = dedup_downloader.with_in_memory_cache();
        #[cfg(not(feature = "moka"))]
        let cached_downloader = dedup_downloader;

        // 创建 Storage
        // 由于登录需要 Storage 提供存放 Authlib-Injector 的位置，将 Storage 移入 MultiStorage 的步骤往后推迟
        // 正常流程建议创建后直接移入 MultiStorage，需要时通过 get 方法取用
        let storage = Storage::new(".minecraft").await?.share_preference(Hardlink);
        // 生成临时文件清理任务
        let (cleaner, shutdown) = storage.run_cleaner();
        smol::spawn(cleaner).detach();

        // 创建登录凭据
        let auth =
            // 此处使用 Yggdrasil 登录
            auth::yggdrasil::Authentication::new_login(&cached_downloader)
                // 注入 Authlib-Injector
                .inject(&storage)
                .await?
                // 自定义 Yggdrasil 地址
                .custom("https://aphanite.enita.cn/api/yggdrasil".parse::<url::Url>()?)
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

        // 构造 MultiStorage
        //
        // 由 MultiStorage 管理 Storage，MultiStorage 可用于全局配置
        // MultiStorageWithPlugin 可以根据需要管理生命周期与 Storage 一致的对象
        let storages = MultiStorageWithPlugin::new();
        // 假设此处希望将清理任务与 Storage 绑定，则插件为 ShutdownGuard
        let storage_with_plugin = StorageWithPlugin {
            storage,
            plugin: shutdown,
        };
        // Storage 移动至 MultiStorage 容器
        let _ = storages
            .insert(
                storage_with_plugin.storage.identifier(),
                storage_with_plugin,
            )
            .await;

        // 构造 MultiAccount
        let accounts = MultiAccount::new();
        let account: Account = auth.into();
        // 将登录凭据移入 MultiAccount
        let _ = accounts.insert(account.identifier(), account).await;

        // Container::insert(k,v) 有可能失败。如果失败，会把传入的 (键, 值) 包装在 Err() 里原样返回。

        // 构造 JavaManager
        let java_manager =
            // 可以使用任何实现 RuntimeScanPath 的类型，例如 Storage 和 MultiStorage
            JavaManager::new(&storages)
                .await
                // 默认启用系统的 Java 运行时检测，此处关闭用于测试
                .disable_system();

        // ———————————————————— 局部的操作阶段 ————————————————————
        {
            // ———————————————————— 获取全局资源 ————————————————————

            // 获取 Storage，取第一个为例
            // Guard 保证 Storage 在当前作用域不会被释放，但是不保证不会被删除
            // 因此任何时候使用 storages.try_get(&id) 都可能为 None
            // 如果希望避免复杂的 id 维护和手动 CAS，请使用 .snapshot()
            let storage_with_plugin = storages.snapshot().into_iter().next().unwrap();
            // StorageWithPlugin 已实现 AsRef<Storage>
            let storage = storage_with_plugin.as_ref();

            // 下载任务组，应该一次性使用
            let downloader = cached_downloader.with_group();
            // （进度监视器）
            let _guard = process_monitor(&downloader);

            // ———————————————————— 业务逻辑 ————————————————————

            // 示例选择的版本名
            const NAME: &str = "1.21.1";

            // （清理测试残留）
            let _ = async_fs::remove_dir_all(storage.versions_dir().join(NAME)).await;

            // 创建实例
            //
            // 获取版本清单
            let version =
                // 下载版本索引
                VersionIndex::sync(&downloader)
                    .await?
                    // 使用迭代器查找需要的版本
                    .iter()
                    .find(|x| x.id == NAME)
                    .expect("Version not found")
                    // // 使用最新的稳定版
                    // .latest_release()?
                    // 下载版本清单
                    .get_manifest(&downloader)
                    .await?;
            // 初始化实例
            let instance = Instance::create(version, Some(NAME), storage, &downloader).await?;

            // 安装 Java
            let java = java_manager
                // 获取符合 major 版本的 JavaRuntime，若不存在则安装，需要传入闭包选择安装位置（需要保证选择的位置在扫描范围内）
                .get_or_install::<Zulu>(instance.java_major(), &downloader, async |x| {
                    // 取第一个为例
                    x.snapshot().into_iter().next().unwrap()
                })
                .await?;
            // 为实例绑定 JavaRuntime，安装模组加载器或启动游戏需要此状态
            let mut instance = instance.bind_java(java.clone()).await?;

            // 安装模组加载器
            instance
                // 安装需要正确的游戏版本 ID，并在闭包内选择需要的加载器版本
                .install_loader::<NeoForge>(NAME, &downloader, async |into_iter| {
                    // （调试输出）
                    // let iter = iter
                    //     .inspect(|x| println!("{}:{} stable:{}", x.name(), x.version(), x.stable()));
                    // 排序和版本选择，元素对版本号实现全序，但是仅人类可读，不一定为正确的版本顺序
                    let latest = into_iter
                        .into_iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .pop_last()
                        .expect("No available loader version");
                    // println!(
                    //     "{}:{} stable:{}",
                    //     latest.name(),
                    //     latest.version(),
                    //     latest.stable()
                    // );
                    Ok(latest)
                })
                .await?;

            // 安装实例
            downloader
                .join(
                    instance
                        // 安装时跳过存在的文件
                        .install_less(std::collections::HashSet::new())
                        .await?,
                )
                .await
                .iter()
                .for_each(|e| tracing::error!("{e}"));

            // 启动游戏
            //
            // 获取登录凭据，取第一个为例
            let auth = accounts.snapshot().into_iter().next().unwrap();
            // 启动前的准备，令牌接近过期时自动续期
            auth.ready(&downloader).await?;
            // 声明游戏完整性（不提供检查），启动游戏需要此状态
            let instance = instance.ensure_ready();
            // 创建启动命令
            let mut cmd = instance.launch(&auth).await?;
            // 启动进程并等待退出
            let exit = cmd.spawn()?.status().await?;

            println!("Game exited: {exit}");
        }

        Ok::<(), Error>(())
    })) {
        tracing::error!("{}", e)
    }
}

// 显示下载速度和进度
/// Shows the download speed and progress
fn process_monitor<D: Downloader, B: std::borrow::Borrow<D> + Send + Sync>(
    group: &DownloadGroup<D, B>,
) -> impl Drop {
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::Relaxed;
    use std::time::Duration;

    struct ExitGuard {
        exit: Arc<AtomicBool>,
    }

    impl Drop for ExitGuard {
        fn drop(&mut self) {
            self.exit.store(true, Relaxed)
        }
    }

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

// 检测阻塞时间
/// Measures the blocking time
fn blocking_monitor(executor: &async_executor::Executor<'static>) -> impl Drop {
    use std::sync::Arc;
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::atomic::{AtomicBool, AtomicU64};
    use std::time::{Duration, Instant};

    pub struct BlockingGuard {
        stopped: Arc<AtomicBool>,
        max_blocking_ns: Arc<AtomicU64>,
    }
    impl Drop for BlockingGuard {
        fn drop(&mut self) {
            self.stopped.store(true, Relaxed);

            let max = Duration::from_nanos(self.max_blocking_ns.load(Relaxed));

            eprintln!("max blocking: {:.3} ms", max.as_secs_f64() * 1000.0,);
        }
    }

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

    BlockingGuard {
        stopped,
        max_blocking_ns,
    }
}
