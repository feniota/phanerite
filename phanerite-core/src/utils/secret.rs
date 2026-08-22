use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serializer};

// `SecretString` 的序列化，`secrecy` 只提供了反序列化
//
// `SecretBox<str>` 无法实现 `Serialize`，只能显式声明
// `#[serde(with = "crate::utils::secret")]`，
// 以此保证凭据不会被无意地写出
/// Serialization for `SecretString`; `secrecy` only provides deserialization
///
/// `SecretBox<str>` cannot implement `Serialize`, so
/// `#[serde(with = "crate::utils::secret")]` has to be declared explicitly.
/// This makes sure credentials are never written out by accident
pub fn serialize<S: Serializer>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(secret.expose_secret())
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<SecretString, D::Error> {
    Ok(SecretString::from(String::deserialize(deserializer)?))
}

// 复制一份凭据
//
// `SecretString` 没有 `Clone`，复制明文必须显式写出来，
// 以此保证凭据不会被无意地散播
/// Makes a copy of a credential
///
/// `SecretString` has no `Clone`, so copying the plaintext has to be spelled
/// out explicitly. This makes sure credentials are never spread by accident
pub fn copy_secret(secret: &SecretString) -> SecretString {
    SecretString::from(secret.expose_secret().to_owned())
}

// 用于 `#[serde(with = "crate::utils::secret::option")]`
/// For use with `#[serde(with = "crate::utils::secret::option")]`
pub mod option {
    use super::*;

    pub fn serialize<S: Serializer>(
        secret: &Option<SecretString>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match secret {
            Some(v) => serializer.serialize_some(v.expose_secret()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<SecretString>, D::Error> {
        Ok(Option::<String>::deserialize(deserializer)?.map(SecretString::from))
    }
}
