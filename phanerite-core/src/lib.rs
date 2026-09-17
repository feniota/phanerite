// 硬链接检测引用计数
#![cfg_attr(target_os = "windows", feature(windows_by_handle))]
//! Phanerite Core — A general backend library for Rust Minecraft launchers.
//!
//! # Module layout
//!
//! Based on their respective functionalities, the flow of a Minecraft launch is
//! splitted into several parts, which lay in different modules of this library:
//!
//! - [`storage`]: Where to store Minecraft files
//! - [`download`]: How to retrieve and save those files
//! - [`instance`]: An isolated Minecraft instance
//! - [`auth`]: Minecraft account management
//! - [`runtime`]: Java runtime management
//!
//! And some additional helper modules:
//!
//! - [`mod_loader`]: Mod loader management
//! - [`mod_repo`]: Interact with public mod registries (Modrinth, CurseForge)
//! - [`parsers`]: Utilties to deal with the game's own files
//! - [`utils`]: Miscellaneous utilties
//!
//! # Design
//!
//! Phanerite Core is well-designed, and aims to be best at flexibility
//! and usability, so it has the following features in mind:
//!
//! ## Composable and Extensible by Default
//!
//! Nearly **everything** that needs to be passed around different modules
//! in Phanerite Core is trait. For example, whenever Phanerite
//! Core needs a Downloader, it calls for an *implementor* of the
//! [`Downloader`](download::Downloader) trait, not a specific struct, so
//! you may truly decide exactly what your code will do.
//!
//! Some other types that have to be a `struct` (so that type parameters
//! will not pollute the whole library) are also designed to be composable,
//! and can be modified dynamically, just like how mod loaders are layered
//! upon [`Instance`](instance::Instance)s.
//!
//! ## Async
//!
//! Phanerite Core is designed to be asynchronous and generic over async
//! runtimes, not tied to a specific runtime library. You can choose
//! whatever runtime you want, still having the same performance.
//!
//! Even if you don't need async, you can simply
//! [`block_on`](https://docs.rs/pollster/latest/pollster/fn.block_on.html)
//! the futures.
//!
//! ## No global state
//!
//! Phanerite Core does not require global states. Everything is created,
//! customized (by composing and extending them) and managed by the caller,
//! and finally passed in explicitly. There is no more global states and
//! implicit arguments, everything is under control.
//!
//! ## Performance
//!
//! Phanerite Core has concurrency in mind by utilizing libraries like
//! [`scc`](https://docs.rs/scc/latest/scc). It also tries to perform
//! minimum copies and dispatch type parameters as statically as
//! possible.
//!
//! # Getting started
//!
//! Here's an example for using Phanerite Core to launch Minecraft:
//!
//! ```no_run
#![doc = include_str!("../examples/quickstart.rs")]
//! ```
//! 
//! # Nightly toolchain required
//!
//! Phanerite Core needs a nightly toolchain to compile since it
//! needs a few beta features. You can check out the source code of
//! `lib.rs` to get a list of these features used in this library.

pub mod auth;
pub mod download;
pub mod instance;
pub mod runtime;
pub mod storage;

pub mod mod_loader;
pub mod mod_repo;
pub mod parsers;
pub mod utils;

pub mod error;
