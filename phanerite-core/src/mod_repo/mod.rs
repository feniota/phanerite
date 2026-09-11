use crate::error::Result;
use crate::instance::Instance;
use chrono::{DateTime, FixedOffset};
use futures::Stream;
use url::Url;

#[derive(Clone, Copy)]
pub enum ModType {
    Mod,
    ModsPack,
    Shader,
    ResourcePack,
}

// 用于显示的模组元信息
#[derive(Clone)]
pub struct ModItem {
    // 仓库内部标识
    pub(super) _id: String,

    pub name: String,
    pub mod_type: ModType,
    pub categories: Vec<String>,
    pub description: String,
    pub icon: Option<Url>,
    pub gallery: Vec<Url>,
    pub created_date: DateTime<FixedOffset>,
    pub updated_date: DateTime<FixedOffset>,
}

// 用于显示的模组版本信息
#[derive(Clone)]
pub struct ModVersion {
    // 仓库内部标识
    pub(super) _id: String,

    pub version: String,
    pub change_log: Option<String>,
    pub updated_date: DateTime<FixedOffset>,
}

// 模组仓库，不保证 Send + Sync，使用时构造而不是全局共享
// 内部维护一份清单，ModItem 和 ModVersion 仅用于显示
pub trait ModsRepository: Default + Sized {
    const NAME: &str;
    const ATTRIBUTION: &str = "";
    const NOTICE: &str = "";
    type Mounted<'instance>: InstanceMods<Self> + 'instance;

    // 根据关键词搜索模组，用于不挂载实例时的操作
    fn search(&mut self, keyword: &str) -> impl Stream<Item = Result<ModItem>>;
    // 挂载到示例，提供对实例的模组操作
    fn mount<'instance, R, C>(
        self,
        instance: &'instance Instance<R, C>,
    ) -> Self::Mounted<'instance>;
}

// 模组的启用状态
pub type Enabled = bool;

// 挂载到实例的模组仓库
#[allow(async_fn_in_trait)]
pub trait InstanceMods<R: ModsRepository> {
    // 根据关键词搜索模组
    fn search(&mut self, keyword: &str) -> impl Stream<Item = Result<ModItem>>;
    // 列出当前实例的模组
    fn list(&mut self) -> impl Stream<Item = Result<(Enabled, ModVersion)>>;
    // 安装模组
    async fn install(&mut self, version: &ModVersion) -> Result<()>;
    // 移除模组
    async fn remove(&mut self, version: &ModVersion) -> Result<()>;
    // 启用模组
    async fn enable(&mut self, version: &ModVersion) -> Result<()>;
    // 禁用模组
    async fn disable(&mut self, version: &ModVersion) -> Result<()>;
}
