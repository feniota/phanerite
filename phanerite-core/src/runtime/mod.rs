use crate::error::Error;
use crate::storage::Storage;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

pub mod java;

/// `runtime` 目录下的子目录命名规则：
/// {os}-{arch}-{package}-{major}-{vendor}
pub(crate) struct RuntimePath {
    /// `std::env::consts::OS`
    os: String,
    /// `std::env::consts::ARCH`
    arch: String,
    /// runtime 类型，例如 `jre`
    package: String,
    /// runtime 版本
    major: usize,
    /// runtime 供应商，例如 `zulu`,`oracle`
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

/// 提供 `RuntimePath` 根目录的类型，例如 `Storage`
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
