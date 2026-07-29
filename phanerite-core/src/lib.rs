// 硬链接检测引用计数
#![cfg_attr(target_os = "windows", feature(windows_by_handle))]

pub mod auth;
pub mod debug;
pub mod download;
pub mod error;
pub mod instance;
pub mod java;
pub mod storage;
pub mod utils;
