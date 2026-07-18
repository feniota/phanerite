use serde::{Deserialize, Serialize};
use std::fmt;
// =======================
// Hash Value
// =======================

pub trait HashValue:
    AsRef<[u8]>
    + Clone
    + Eq
    + Send
    + Sync
    + fmt::Debug
    + Serialize
    + for<'de> Deserialize<'de>
    + Into<Hash>
{
    const NAME: &'static str;

    type Algorithm: HashAlgorithm<Value = Self>;

    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized;

    fn hasher() -> <Self::Algorithm as HashAlgorithm>::Hasher {
        Self::Algorithm::create()
    }
}

// =======================
// Hash Enum
// =======================

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "algorithm", content = "value")]
pub enum Hash {
    Blake3(Blake3Hash),
    Sha1(Sha1Hash),
}

impl Hash {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Blake3(v) => v.as_ref(),
            Self::Sha1(v) => v.as_ref(),
        }
    }

    pub fn algorithm(&self) -> &'static str {
        match self {
            Self::Blake3(_) => Blake3Hash::NAME,
            Self::Sha1(_) => Sha1Hash::NAME,
        }
    }
}

// =======================
// Hasher
// =======================

pub trait Hasher {
    type Value: HashValue;

    fn update(&mut self, data: &[u8]);

    fn finalize(self) -> Self::Value;
}

// =======================
// Algorithm
// =======================

pub trait HashAlgorithm {
    const NAME: &'static str;

    type Value: HashValue;

    type Hasher: Hasher<Value = Self::Value>;

    fn create() -> Self::Hasher;
}

// =======================
// HashValue Macro
// =======================

macro_rules! impl_hash_value {
    (
        $name:ident,
        $size:expr,
        $algo:literal,
        $algorithm:ty,
        $variant:ident
    ) => {
        #[derive(Clone, Eq, PartialEq)]
        pub struct $name(pub(crate) [u8; $size]);

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}:{}", Self::NAME, hex::encode(self.0))
            }
        }

        impl HashValue for $name {
            const NAME: &'static str = $algo;

            type Algorithm = $algorithm;

            fn from_bytes(bytes: &[u8]) -> Option<Self> {
                if bytes.len() != $size {
                    return None;
                }

                let mut value = [0u8; $size];

                value.copy_from_slice(bytes);

                Some(Self(value))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&hex::encode(self.0))
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;

                let bytes = hex::decode(value).map_err(serde::de::Error::custom)?;

                Self::from_bytes(&bytes)
                    .ok_or_else(|| serde::de::Error::custom("invalid hash length"))
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                hex::encode(self.0).eq_ignore_ascii_case(other)
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.eq(*other)
            }
        }

        impl From<$name> for Hash {
            fn from(value: $name) -> Self {
                Hash::$variant(value)
            }
        }
    };
}

// =======================
// Algorithms
// =======================

pub struct Blake3Algorithm;

pub struct Sha1Algorithm;

// =======================
// Hash Types
// =======================

impl_hash_value!(Blake3Hash, 32, "blake3", Blake3Algorithm, Blake3);

impl_hash_value!(Sha1Hash, 20, "sha1", Sha1Algorithm, Sha1);

// =======================
// Blake3
// =======================

impl HashAlgorithm for Blake3Algorithm {
    const NAME: &'static str = "blake3";

    type Value = Blake3Hash;

    type Hasher = blake3::Hasher;

    fn create() -> Self::Hasher {
        blake3::Hasher::new()
    }
}

impl Hasher for blake3::Hasher {
    type Value = Blake3Hash;

    fn update(&mut self, data: &[u8]) {
        blake3::Hasher::update(self, data);
    }

    fn finalize(self) -> Self::Value {
        Blake3Hash(self.finalize().0)
    }
}

// =======================
// Sha1
// =======================

use sha1::{Digest, Sha1};

impl HashAlgorithm for Sha1Algorithm {
    const NAME: &'static str = "sha1";

    type Value = Sha1Hash;

    type Hasher = Sha1;

    fn create() -> Self::Hasher {
        Sha1::new()
    }
}

impl Hasher for Sha1 {
    type Value = Sha1Hash;

    fn update(&mut self, data: &[u8]) {
        Digest::update(self, data);
    }

    fn finalize(self) -> Self::Value {
        Sha1Hash(Digest::finalize(self).into())
    }
}
