use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UnhyphenatedUuid(Uuid);

impl From<Uuid> for UnhyphenatedUuid {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<UnhyphenatedUuid> for Uuid {
    fn from(value: UnhyphenatedUuid) -> Self {
        value.0
    }
}

impl Serialize for UnhyphenatedUuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.0.simple().to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for UnhyphenatedUuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let uuid = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
        Ok(UnhyphenatedUuid(uuid))
    }
}
