use crate::io::utils::Hasher;

/// Content-addressable hash value.
///
/// Concrete implementations: [`Sha1`], [`Blake3`].
pub trait HashValue: std::fmt::Display + std::fmt::Debug + Clone + PartialEq + Eq {
    fn as_bytes(&self) -> &[u8];
    fn from_hex(hex: impl AsRef<str>) -> Self;
    fn hasher() -> impl Hasher;
}

/// SHA-1 hash (20 bytes).
#[derive(Clone, PartialEq, Eq)]
pub struct Sha1(pub(crate) [u8; 20]);

impl HashValue for Sha1 {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    fn from_hex(hex: impl AsRef<str>) -> Self {
        let bytes = hex::decode(hex.as_ref()).expect("invalid hex");
        assert_eq!(bytes.len(), 20);
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Sha1(arr)
    }
    fn hasher() -> impl Hasher {
        sha1::Sha1::default()
    }
}

impl Sha1 {
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        Sha1(bytes)
    }
}

/// BLAKE3 hash (32 bytes).
#[derive(Clone, PartialEq, Eq)]
pub struct Blake3(pub(crate) [u8; 32]);

impl HashValue for Blake3 {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    fn from_hex(hex: impl AsRef<str>) -> Self {
        let bytes = hex::decode(hex.as_ref()).expect("invalid hex");
        assert_eq!(bytes.len(), 32);
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Blake3(arr)
    }
    fn hasher() -> impl Hasher {
        blake3::Hasher::new()
    }
}

impl Blake3 {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Blake3(bytes)
    }
}

macro_rules! impl_hash_serde {
    ($ty:ty) => {
        impl serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }
        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let hex: String = String::deserialize(deserializer)?;
                Ok(HashValue::from_hex(hex))
            }
        }
    };
}
impl_hash_serde!(Sha1);
impl_hash_serde!(Blake3);

macro_rules! impl_hash_fmt {
    ($ty:ty) => {
        impl std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($ty))
                    .field(&hex::encode(self.as_bytes()))
                    .finish()
            }
        }
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&hex::encode(self.as_bytes()))
            }
        }
    };
}
impl_hash_fmt!(Sha1);
impl_hash_fmt!(Blake3);
