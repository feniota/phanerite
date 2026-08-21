use phanerite_core::auth::Authentication;
use phanerite_core::auth::microsoft;
use phanerite_core::download;
use phanerite_core::error::Error;
use secrecy::ExposeSecret;
use tracing::{Level, error};

fn main() {
    // （客户端 ID 与刷新令牌）
    let _ = dotenvy::dotenv();
    // 日志输出
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 异步 Runtime
    if let Err(e) = smol::block_on(async {
        // 登录只需要网络访问，不需要缓存与任务组
        let downloader = download::downloader::RawDownloader::builder()
            .build()
            .await?;

        // 创建登录会话
        let login =
            // 此处使用微软账户的设备码登录
            microsoft::Authentication::new_login(&downloader)
            // 可登录的账户类型，默认仅个人账户
            .tenant(microsoft::Tenant::Consumers)
            // Azure 应用注册的客户端 ID，需要允许公共客户端流
            .client_id(
                std::env::var("CLIENT_ID")
                    .expect("Fill in the Azure client ID in the environment variable"),
            );

        // 持久化的刷新令牌可以免交互登录
        let mut auth = match std::env::var("REFRESH_TOKEN") {
            Ok(refresh_token) => login.refresh(refresh_token).await?,
            Err(_) => {
                // 申请设备码
                let mut pending = login.device_code().await?;
                // 提示用户完成授权，`message` 已经包含地址与用户码
                println!("{}", pending.message);
                println!(
                    "({} 后过期，每 {} 秒轮询一次)",
                    pending.expires_at(),
                    pending.interval().as_secs()
                );
                // 按服务端建议的间隔轮询直到授权完成，计时器由调用方提供
                pending
                    .wait(async |d| {
                        smol::Timer::after(d).await;
                    })
                    .await?
            }
        };

        // 启动前的准备，令牌接近过期时自动续期
        auth.ready(&downloader).await?;

        println!("Player: {} ({})", auth.profile.name, auth.profile.id);
        println!("XUID: {}", auth.xuid);
        println!("Expires at: {}", auth.expires_at());
        if let Some(skin) = auth.profile.skin() {
            println!("Skin: {} ({:?})", skin.url, skin.variant);
        }
        if let Some(cape) = auth.profile.cape() {
            println!("Cape: {}", cape.url);
        }

        // 刷新令牌需要持久化，下次登录即可免交互
        // （示例直接输出，实际应该加密保存且不要写入日志）
        if let Some(refresh_token) = auth.refresh_token() {
            println!("REFRESH_TOKEN={}", refresh_token.expose_secret());
        }

        // 之后可以像 fullflow 一样用于启动游戏
        // let mut cmd = instance.launch(&auth).await?;

        Ok::<(), Error>(())
    }) {
        error!("{}", e)
    }
}
