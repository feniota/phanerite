use crate::auth::microsoft::MicrosoftError;
use crate::auth::yggdrasil::YggdrasilError;
use std::sync::Arc;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] isahc::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Serde Xml error: {0}")]
    SerdeXml(#[from] serde_xml_rs::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Url parse error: {0}")]
    UrlParseErr(#[from] url::ParseError),

    /// 用于 Moka 的缓存错误
    #[error(transparent)]
    CacheErrors(#[from] Arc<Self>),
    /// Yggdrasil 错误，迁移自 Aphanite
    #[error(transparent)]
    Yggdrasil(#[from] YggdrasilError),
    /// 微软登录链路中的错误
    #[error(transparent)]
    Microsoft(#[from] MicrosoftError),

    /// 用户取消操作
    #[error("Operation cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}
