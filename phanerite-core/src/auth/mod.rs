use crate::download::Downloader;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::arguments::LaunchArguments;
use crate::instance::variables::Variables;
use crate::utils::state::NotReady;
use serde::{Deserialize, Serialize};

pub mod microsoft;
pub mod offline;
pub mod yggdrasil;

// 临时的 trait，可能需要改进
#[allow(async_fn_in_trait)]
pub trait Authentication {
    /// 根据登录信息生成变量
    async fn vars(&self) -> Result<Variables<NotReady>>;
    /// 需要对启动参数的额外注入
    fn inject(&self) -> impl AsyncFnOnce(&mut LaunchArguments) {
        async |_| {}
    }
    /// 启动前的准备，凭据失效时自行续期
    ///
    /// 无法续期时返回错误，此时需要用户重新登录
    async fn ready(&mut self, downloader: &impl Downloader) -> Result<()> {
        let _ = downloader;
        Ok(())
    }

    /// 根据登录信息和 `Instance` 生成启动参数
    async fn args<R: Clone, C: Clone>(
        &self,
        instance: &Instance<'_, R, C>,
    ) -> Result<LaunchArguments> {
        let vars = self.vars().await?;
        let mut args = vars.to_arguments(instance)?;
        self.inject()(&mut args).await;
        Ok(args)
    }
}

/// 统一的账户，用于持久化
///
/// 落盘位置与加密方式由调用方决定，
/// 其中的凭据都是明文，不应该直接暴露给用户
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Account {
    Offline(offline::Authentication),
    Microsoft(microsoft::Authentication),
    Yggdrasil(yggdrasil::Authentication),
}

impl From<offline::Authentication> for Account {
    fn from(value: offline::Authentication) -> Self {
        Account::Offline(value)
    }
}

impl From<microsoft::Authentication> for Account {
    fn from(value: microsoft::Authentication) -> Self {
        Account::Microsoft(value)
    }
}

impl From<yggdrasil::Authentication> for Account {
    fn from(value: yggdrasil::Authentication) -> Self {
        Account::Yggdrasil(value)
    }
}

impl Authentication for Account {
    async fn vars(&self) -> Result<Variables<NotReady>> {
        match self {
            Account::Offline(a) => a.vars().await,
            Account::Microsoft(a) => a.vars().await,
            Account::Yggdrasil(a) => a.vars().await,
        }
    }
    fn inject(&self) -> impl AsyncFnOnce(&mut LaunchArguments) {
        // 各分支的返回类型不同，只能在闭包内部分发
        async |args| match self {
            Account::Offline(a) => a.inject()(args).await,
            Account::Microsoft(a) => a.inject()(args).await,
            Account::Yggdrasil(a) => a.inject()(args).await,
        }
    }
    async fn ready(&mut self, downloader: &impl Downloader) -> Result<()> {
        match self {
            Account::Offline(a) => a.ready(downloader).await,
            Account::Microsoft(a) => a.ready(downloader).await,
            Account::Yggdrasil(a) => a.ready(downloader).await,
        }
    }
}
