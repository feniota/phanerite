// 硬链接检测引用计数
#![cfg_attr(target_os = "windows", feature(windows_by_handle))]
// Phanerite 的核心库：一个没有界面的 Minecraft 启动器后端
//
// # 模块划分
//
// 按职责而不是按流程切分，模块之间只通过 trait 和值耦合，装配由调用方
// 完成：
//
// - [`storage`]：数据放在哪里
// - [`download`]：数据怎么拿到本地
// - [`instance`]：一个可启动的游戏版本是什么
// - [`auth`]：用谁的身份启动
// - [`runtime`]：用哪个 Java 启动
// - [`mod_loader`]：怎么把模组加载器叠加到实例上
// - [`mod_project`]：从哪里找模组
// - [`parsers`]：怎么读写游戏侧的文件和日志
//
// 一条从登录到启动的完整链路见 `examples/fullflow.rs`。
//
// # 没有全局状态
//
// 这里不存在任何单例。[`storage::Storage`]、[`download::Downloader`]、
// [`runtime::java::JavaManager`] 都由调用方创建并显式传入。代价是装配
// 阶段略显啰嗦，好处是同一个进程里可以并存多套互不干扰的配置，测试也
// 不需要替换全局变量。
//
// 确实需要全局共享的对象（登录凭据、`Storage`）放进
// [`utils::container::Container`]，由它提供并发安全的存取和快照。
//
// # 异步
//
// 只依赖 `futures` 与 `async-*` 系列，不绑定具体的异步运行时，示例用的
// 是 `smol`。真正会长时间占住 CPU 的解压被放到独立线程上做（见
// [`download::extract`]），不会卡住执行器。
//
// # nightly
//
// 需要 nightly 工具链，用到的不稳定特性见本文件顶部的 `#![feature(..)]`。
//! The core library of Phanerite: a headless Minecraft launcher backend
//!
//! # Module layout
//!
//! Modules are split by responsibility rather than by the order of the
//! launch flow. They are coupled only through traits and values; wiring
//! them together is the caller's job:
//!
//! - [`storage`]: where the data lives
//! - [`download`]: how the data gets onto the disk
//! - [`instance`]: what a launchable game version is
//! - [`auth`]: whose identity the game is launched with
//! - [`runtime`]: which Java launches it
//! - [`mod_loader`]: how a mod loader is layered onto an instance
//! - [`mod_project`]: where to find mods
//! - [`parsers`]: how to read and write the game's own files and logs
//!
//! See `examples/fullflow.rs` for a complete chain from login to launch.
//!
//! # No global state
//!
//! There are no singletons here. [`storage::Storage`],
//! [`download::Downloader`] and [`runtime::java::JavaManager`] are all
//! created by the caller and passed in explicitly. The price is a somewhat
//! verbose wiring stage; the payoff is that several independent
//! configurations can coexist in one process, and tests never have to
//! swap out a global.
//!
//! The objects that genuinely do need to be shared globally (credentials,
//! `Storage`) go into [`utils::container::Container`], which provides
//! concurrent access and snapshots.
//!
//! # Async
//!
//! Only `futures` and the `async-*` family are used; no particular async
//! runtime is baked in, and the examples happen to use `smol`. Extraction,
//! the one step that really does hold the CPU for a long time, is moved
//! onto a dedicated thread (see [`download::extract`]) so that it cannot
//! stall the executor.
//!
//! # Nightly
//!
//! A nightly toolchain is required; the unstable features in use are
//! listed in the `#![feature(..)]` attributes at the top of this file.

pub mod auth;
pub mod download;
pub mod error;
pub mod instance;
pub mod mod_loader;
pub mod mod_project;
pub mod parsers;
pub mod runtime;
pub mod storage;
pub mod utils;
