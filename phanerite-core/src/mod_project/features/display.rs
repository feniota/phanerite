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

pub trait ModProjectDisplayExt: ModProject {
    /// 展示名称
    fn title(&self) -> impl Display + '_;
    /// 描述
    fn description(&self) -> impl Display + '_;
    /// 具体描述
    fn body(&self) -> impl Display + '_;
    /// 作者
    fn author(&self) -> impl Display + '_;
    /// 创建时间
    fn created_time(&self) -> &DateTime<FixedOffset>;
    /// 修改时间
    fn updated_time(&self) -> &DateTime<FixedOffset>;
    /// 许可证
    fn license(&self) -> impl Iterator<Item = impl Display + '_> {
        std::iter::empty::<&str>()
    }
    /// 展示分类
    fn categories(&self) -> impl Iterator<Item = impl Display + '_> {
        std::iter::empty::<&str>()
    }
    /// 图标文件
    fn icon(&self) -> Option<Url> {
        None
    }
    /// 颜色
    fn color(&self) -> Option<Rgb> {
        None
    }
    /// 下载量
    fn downloads(&self) -> Option<usize> {
        None
    }
    /// 关注量
    fn follows(&self) -> Option<usize> {
        None
    }
    /// 展示相册
    fn gallery(&self) -> Vec<Url> {
        vec![]
    }
}
