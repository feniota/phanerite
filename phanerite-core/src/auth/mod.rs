use crate::error::Result;
use crate::instance::arguments::LaunchArguments;
use crate::instance::Instance;
use crate::storage::Storage;

pub mod offline;
pub mod yggdrasil;

#[allow(async_fn_in_trait)]
pub trait Authentication {
    async fn args<R, C>(
        &self,
        instance: &Instance<R, C>,
        storage: &Storage,
    ) -> Result<LaunchArguments>;
}
