// 舒适处理 Iterator 错误
#![feature(iterator_try_collect)]
// 测试 impl_restriction 并反馈到 Rust 社区
#![feature(impl_restriction)]
// 硬链接检测引用计数
#![cfg_attr(target_os = "windows", feature(windows_by_handle))]

pub mod auth;
pub mod download;
pub mod error;
pub mod instance;
pub mod mod_loader;
pub mod mod_project;
pub mod runtime;
pub mod storage;
pub mod utils;
