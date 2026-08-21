use crate::error::Result;
use crate::instance::variables::Variables;
use crate::utils::state::NotReady;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 离线登录
#[derive(Serialize, Deserialize)]
pub struct Authentication {
    pub nickname: String,
    pub uuid: Uuid,
}

impl Authentication {
    pub fn new(nickname: impl Into<String>) -> Self {
        let nickname = nickname.into();
        Self {
            uuid: offline_uuid(&nickname),
            nickname,
        }
    }
}

/// 离线登录由 UUID 确定
impl PartialEq for Authentication {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Eq for Authentication {}

impl super::Authentication for Authentication {
    async fn vars(&self) -> Result<Variables<NotReady>> {
        Ok(Variables::new()
            .required(&self.nickname, self.uuid.to_string(), "0")
            .legacy("0", "legacy"))
    }
    /// 没有会变化的状态，直接借用自身
    async fn serialize(&self) -> impl Serialize {
        self
    }
}

/// 离线模式的 UUID 算法
pub fn offline_uuid(username: &str) -> Uuid {
    let input = format!("OfflinePlayer:{username}");

    let mut bytes = md5::compute(input.as_bytes()).0;

    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Uuid::from_bytes(bytes)
}
