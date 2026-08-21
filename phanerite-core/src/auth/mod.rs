//! 登录凭据
//!
//! 凭据会被放进 [`MultiAccount`] 全局共享，因此所有操作都取 `&self`，
//! 会随续期变化的部分由各自内部的一把 `RwLock` 保护。
//!
//! # 并发
//!
//! 续期不是纯函数：Yggdrasil 的 `refresh` 会作废旧的访问令牌，
//! 微软则会轮换刷新令牌。因此「读旧令牌 → 请求 → 写新令牌」
//! 必须是一个临界区，否则并发续期会把已经作废的令牌写回本地。
//!
//! 临界区用 `RwLock::upgradable_read()` 进入：它允许其他读者继续读，
//! 但互斥其他可升级读者，等于免费的单飞门闩；网络交互结束后
//! `upgrade()` 成写锁提交，提交期间不再 `await`。
//!
//! 划线标准是「**会不会改变服务端的令牌状态**」而不是「要不要 `&mut`」，
//! 所以 `invalidate()` / `signout()` 这类不写本地状态的操作同样要进临界区。
//!
//! 由此约定三条纪律，改动这个模块时必须遵守：
//!
//! 1. 锁只在公开方法里获取，私有 helper 一律不碰锁（签名收 `&State`
//!    或裸字段，返回待提交的新状态）。公开方法可以先做一次短读锁的
//!    快路径预检，但进入临界区之后不得再次获取。
//! 2. 持锁期间不调用任何其他 `&self` 方法。`RwLock` 是写者优先的，
//!    递归读锁会被排队中的写者挡死。
//! 3. guard 不逃逸，任何公开访问器都不返回 guard。持久化用
//!    [`Authentication::serialize()`]：它在锁内取一份快照，
//!    序列化本身在锁外进行，不需要阻塞获取读锁。

use crate::download::Downloader;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::arguments::LaunchArguments;
use crate::instance::variables::Variables;
use crate::utils::container::Container;
use crate::utils::state::NotReady;
use serde::{Deserialize, Serialize};

pub mod microsoft;
pub mod offline;
pub mod yggdrasil;

pub type MultiAccount = Container<Account>;

// 临时的 trait，可能需要改进
#[allow(async_fn_in_trait)]
pub trait Authentication: Eq {
    /// 根据登录信息生成变量
    async fn vars(&self) -> Result<Variables<NotReady>>;
    /// 取一份可以持久化的快照
    ///
    /// 内部状态由锁保护，`Serialize` 是同步的，两者无法直接对接，
    /// 因此先在锁内复制出快照，序列化在锁外进行。
    ///
    /// 快照里的凭据是明文，落盘位置与加密方式由调用方决定
    async fn serialize(&self) -> impl Serialize;
    /// 需要对启动参数的额外注入
    fn inject(&self) -> impl AsyncFnOnce(&mut LaunchArguments) {
        async |_| {}
    }
    /// 启动前的准备，凭据失效时自行续期
    ///
    /// 无法续期时返回错误，此时需要用户重新登录
    async fn ready(&self, downloader: &impl Downloader) -> Result<()> {
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
#[derive(Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Account {
    Offline(offline::Authentication),
    Microsoft(microsoft::Authentication),
    Yggdrasil(yggdrasil::Authentication),
}

/// [`Account`] 的快照，与 [`Account`] 的序列化格式一致
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountData<'a> {
    Offline(&'a offline::Authentication),
    Microsoft(microsoft::Data<'a>),
    Yggdrasil(yggdrasil::Data<'a>),
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
    async fn serialize(&self) -> impl Serialize {
        // 各分支的快照类型不同，统一收敛到 `AccountData`
        match self {
            Account::Offline(a) => AccountData::Offline(a),
            Account::Microsoft(a) => AccountData::Microsoft(a.snapshot().await),
            Account::Yggdrasil(a) => AccountData::Yggdrasil(a.snapshot().await),
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
    async fn ready(&self, downloader: &impl Downloader) -> Result<()> {
        match self {
            Account::Offline(a) => a.ready(downloader).await,
            Account::Microsoft(a) => a.ready(downloader).await,
            Account::Yggdrasil(a) => a.ready(downloader).await,
        }
    }
}
