use crate::error::Result;
use crate::instance::arguments::LaunchArguments;
use crate::instance::Instance;

pub mod offline;
pub mod yggdrasil;

// 临时的 trait，可能需要改进
#[allow(async_fn_in_trait)]
pub trait Authentication {
    /// 根据登录信息和 `Instance` 生成启动参数
    async fn args<R: Clone, C: Clone>(&self, instance: &Instance<R, C>) -> Result<LaunchArguments>;
}
