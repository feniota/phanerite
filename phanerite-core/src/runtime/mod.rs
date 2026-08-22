// Java 运行时的定位与安装
//
// # 目录约定
//
// 内建运行时放在 `{root}/runtime/` 下，子目录名必须是
// `{os}-{arch}-{package}-{major}-{vendor}`（见 `RuntimePath`）。名字
// 本身就是索引：扫描时不必逐个执行 `java -version` 就能先按平台筛掉不
// 相干的目录，不符合命名规则的目录会被直接忽略。
//
// # 扫描范围
//
// [`RuntimeScanPath`] 抽象「去哪里找」，[`Storage`] 和
// [`MultiStorage`](crate::storage::multi::MultiStorage) 都实现了它，所以
// 既可以只扫一个游戏目录，也可以一次扫完全部。
//
// [`java::JavaManager`] 在此之上再叠加系统里的 Java（`PATH` 与
// `JAVA_HOME`），可以用 [`java::JavaManager::disable_system`] 关掉。识别
// 版本靠真的把 `java -XshowSettings:properties -version` 跑一遍，因此
// [`java::JavaManager::refresh`] 的开销与候选数量成正比，不适合频繁调用。
//
// 去重按可执行文件的绝对路径进行：同一个 JDK 如果能通过两条不同路径找
// 到，会被当成两个。
//
// # 安装
//
// [`java::JavaManager::get_or_install`] 先找符合大版本的运行时，找不到才
// 下载。装到哪里由调用方给的闭包决定，必须选在扫描范围之内，否则装完也
// 找不回来。具体的下载源实现
// [`JavaDownload`](crate::download::java::JavaDownload)，目前有
// [`Zulu`](crate::download::java::Zulu)。
//! Locating and installing Java runtimes
//!
//! # Directory convention
//!
//! Bundled runtimes live under `{root}/runtime/`, and each subdirectory must
//! be named `{os}-{arch}-{package}-{major}-{vendor}` (see `RuntimePath`).
//! The name is the index: a scan can rule out irrelevant directories by
//! platform without executing `java -version` on each of them, and
//! directories that do not match the naming rule are ignored outright.
//!
//! # Scan scope
//!
//! [`RuntimeScanPath`] abstracts over "where to look". [`Storage`] and
//! [`MultiStorage`](crate::storage::multi::MultiStorage) both implement it,
//! so a scan can cover a single game directory or all of them at once.
//!
//! On top of that, [`java::JavaManager`] also picks up the system's own Java
//! (`PATH` and `JAVA_HOME`), which can be turned off with
//! [`java::JavaManager::disable_system`]. Versions are identified by actually
//! running `java -XshowSettings:properties -version`, so the cost of
//! [`java::JavaManager::refresh`] is proportional to the number of
//! candidates and it is not meant to be called often.
//!
//! Deduplication is by the absolute path of the executable: one JDK
//! reachable through two different paths counts as two.
//!
//! # Installation
//!
//! [`java::JavaManager::get_or_install`] first looks for a runtime matching
//! the major version and only downloads when there is none. Where it is
//! installed is decided by a closure the caller supplies, and that location
//! must lie within the scan scope or the result will not be found again.
//! Concrete sources implement
//! [`JavaDownload`](crate::download::java::JavaDownload); currently that is
//! [`Zulu`](crate::download::java::Zulu).

use crate::error::Error;
use crate::storage::Storage;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

pub mod java;

// `runtime` 目录下的子目录命名规则：
// {os}-{arch}-{package}-{major}-{vendor}
/// Naming rule for the subdirectories under `runtime`:
/// {os}-{arch}-{package}-{major}-{vendor}
pub(crate) struct RuntimePath {
    /// `std::env::consts::OS`
    os: String,
    /// `std::env::consts::ARCH`
    arch: String,
    // runtime 类型，例如 `jre`
    /// Runtime type, for example `jre`
    package: String,
    // runtime 版本
    /// Runtime version
    major: usize,
    // runtime 供应商，例如 `zulu`,`oracle`
    /// Runtime vendor, for example `zulu`, `oracle`
    vendor: String,
}

impl RuntimePath {
    pub fn new(package: impl Into<String>, major: usize, vendor: impl Into<String>) -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            package: package.into(),
            major,
            vendor: vendor.into(),
        }
    }
    pub fn matches_current(&self) -> bool {
        self.os == std::env::consts::OS && self.arch == std::env::consts::ARCH
    }
}

// 提供 `RuntimePath` 根目录的类型，例如 `Storage`
/// A type that provides the root directory of a `RuntimePath`, for example
/// `Storage`
pub trait RuntimeScanPath {
    type Provider<'a>: AsRef<Storage> + 'a
    where
        Self: 'a;
    fn storages(&self) -> impl Iterator<Item = Self::Provider<'_>> + '_;
}
impl AsRef<Storage> for Storage {
    fn as_ref(&self) -> &Storage {
        self
    }
}

impl RuntimeScanPath for Storage {
    type Provider<'a> = &'a Storage;
    fn storages(&self) -> impl Iterator<Item = Self::Provider<'_>> + '_ {
        std::iter::once(self)
    }
}

impl FromStr for RuntimePath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        Ok(Self {
            os: parts
                .next()
                .ok_or(Error::other("OS is missing"))?
                .to_string(),
            arch: parts
                .next()
                .ok_or(Error::other("architecture is missing"))?
                .to_string(),
            package: parts
                .next()
                .ok_or(Error::other("package is missing"))?
                .to_string(),
            major: parts
                .next()
                .ok_or(Error::other("major is missing"))?
                .parse()
                .map_err(|_| Error::other("major is missing"))?,
            vendor: parts
                .next()
                .ok_or(Error::other("vendor is missing"))?
                .to_string(),
        })
    }
}

impl TryFrom<OsString> for RuntimePath {
    type Error = Error;
    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        value.to_string_lossy().parse()
    }
}

impl Display for RuntimePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}-{}-{}",
            self.os, self.arch, self.package, self.major, self.vendor
        )
    }
}

impl From<RuntimePath> for PathBuf {
    fn from(value: RuntimePath) -> Self {
        PathBuf::from(value.to_string())
    }
}
