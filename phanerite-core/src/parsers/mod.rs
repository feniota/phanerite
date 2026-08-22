// 游戏侧文件与日志的读写
//
// 这个模块只和 Minecraft 自己产生的东西打交道，不依赖
// [`Storage`](crate::storage::Storage) 也不依赖
// [`Downloader`](crate::download::Downloader)，可以脱离启动流程单独使用。
//
// - [`logs`]：有状态的日志解析器。[`logs::LogsParser::update`] 接受任意
//   大小的 chunk，自己处理跨 chunk 的半行，一行可以产出多个事件；
//   [`logs::State`] 保留能从历史日志推出、且对后续解析有意义的上下文
//   （在线玩家、服务器生命周期等）。适合直接接在子进程的 stdout 上。
// - [`options`]：`options.txt` 的读写。[`options::OptionEditor`] 按行改写
//   而不是整体重写，键名和冒号之前的格式原样保留，游戏自己写进去的未知
//   键也不会丢。
// - [`launcher_profiles`]：官方启动器的 `launcher_profiles.json`。未知
//   字段统一收进 `other`，往返读写不会丢东西。
//! Reading and writing the game's own files and logs
//!
//! This module deals only with what Minecraft itself produces. It depends on
//! neither [`Storage`](crate::storage::Storage) nor
//! [`Downloader`](crate::download::Downloader), so it can be used entirely
//! outside the launch flow.
//!
//! - [`logs`]: a stateful log parser. [`logs::LogsParser::update`] accepts
//!   chunks of any size and handles half-lines spanning chunk boundaries
//!   itself, and one line may yield several events; [`logs::State`] retains
//!   the context that can be inferred from the log so far and that matters
//!   for parsing what follows (players online, server lifecycle, and so on).
//!   It is meant to be attached directly to a child process's stdout.
//! - [`options`]: reading and writing `options.txt`. [`options::OptionEditor`]
//!   rewrites line by line instead of regenerating the file, preserving the
//!   key names and whatever formatting precedes the colon, and never losing
//!   unknown keys the game wrote itself.
//! - [`launcher_profiles`]: the official launcher's
//!   `launcher_profiles.json`. Unknown fields are collected into `other`, so
//!   a read/write round trip loses nothing.

pub mod launcher_profiles;
pub mod logs;
pub mod options;
