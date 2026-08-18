use crate::mod_project::ModProject;
use chrono::{DateTime, FixedOffset};
use std::fmt::Display;
use url::Url;

/// 颜色
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[allow(async_fn_in_trait)]
pub trait ModDisplay: ModProject {
    /// 展示名称
    async fn title(&self) -> impl Display + '_;
    /// 描述
    async fn description(&self) -> impl Display + '_;
    /// 具体描述
    async fn body(&self) -> impl Display + '_;
    /// 作者
    async fn author(&self) -> impl Display + '_;
    /// 创建时间
    async fn created_time(&self) -> &DateTime<FixedOffset>;
    /// 修改时间
    async fn updated_time(&self) -> &DateTime<FixedOffset>;
    /// 许可证
    async fn license(&self) -> impl Iterator<Item = impl Display + '_> {
        std::iter::empty::<&str>()
    }
    /// 展示分类
    async fn categories(&self) -> impl Iterator<Item = impl Display + '_> {
        std::iter::empty::<&str>()
    }
    /// 图标文件
    async fn icon(&self) -> Option<Url> {
        None
    }
    /// 颜色
    async fn color(&self) -> Option<Rgb> {
        None
    }
    /// 下载量
    async fn downloads(&self) -> Option<usize> {
        None
    }
    /// 关注量
    async fn follows(&self) -> Option<usize> {
        None
    }
    /// 展示相册
    async fn gallery(&self) -> Vec<Url> {
        vec![]
    }
}
