use crate::auth::yggdrasil::Authentication;
use crate::download::authlib_injector::AuthlibInjector;
use crate::error::Result;
use crate::instance::Instance;
use crate::instance::arguments::LaunchArguments;
use crate::storage::Storage;

impl Authentication {
    pub async fn injected_args(
        &self,
        instance: &Instance,
        storage: &Storage,
        authlib_injector: &AuthlibInjector<'_>,
    ) -> Result<LaunchArguments> {
        let mut args = self.args(instance, storage)?;
        let agent = format!(
            "-javaagent:{}={}",
            authlib_injector.get().await?.to_string_lossy(),
            self.server,
        );
        let meta = format!(
            "-Dauthlibinjector.yggdrasil.prefetched={}",
            self.meta_base64()?
        );
        args.jvm.push((agent, None));
        args.jvm.push((meta, None));
        Ok(args)
    }
}
