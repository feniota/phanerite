use crate::error::Result;
use crate::instance::Instance;
use crate::instance::arguments::LaunchArguments;
use crate::instance::variables::Variables;
use crate::utils::state::NotReady;

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
