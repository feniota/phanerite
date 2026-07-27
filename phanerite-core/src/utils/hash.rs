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
    + fmt::Display
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
    Empty(EmptyHash),
    Blake3(Blake3Hash),
    Sha1(Sha1Hash),
    Sha256(Sha256Hash),
}

impl Hash {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Empty(v) => v.as_ref(),
            Self::Blake3(v) => v.as_ref(),
            Self::Sha1(v) => v.as_ref(),
            Self::Sha256(v) => v.as_ref(),
        }
    }

    pub fn algorithm(&self) -> &'static str {
        match self {
            Self::Empty(_) => EmptyHash::NAME,
            Self::Blake3(_) => Blake3Hash::NAME,
            Self::Sha1(_) => Sha1Hash::NAME,
            Self::Sha256(_) => Sha256Hash::NAME,
        }
    }

    pub fn hasher(&self) -> HashHasher {
        match self {
            Self::Empty(_) => HashHasher::Empty(EmptyAlgorithm::create()),
            Self::Blake3(_) => HashHasher::Blake3(Blake3Algorithm::create()),
            Self::Sha1(_) => HashHasher::Sha1(Sha1Algorithm::create()),
            Self::Sha256(_) => HashHasher::Sha256(Sha256Algorithm::create()),
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
// Runtime Hasher
// =======================

// 傻逼 Clippy，32B 会炸栈再来找我
#[allow(clippy::large_enum_variant)]
pub enum HashHasher {
    Empty(EmptyHasher),
    Blake3(blake3::Hasher),
    Sha1(Sha1),
    Sha256(Sha256),
}

impl HashHasher {
    pub fn update(&mut self, data: &[u8]) {
        match self {
            Self::Empty(v) => v.update(data),
            Self::Blake3(v) => Hasher::update(v, data),
            Self::Sha1(v) => Hasher::update(v, data),
            Self::Sha256(v) => Hasher::update(v, data),
        }
    }

    pub fn finalize(self) -> Hash {
        match self {
            Self::Empty(v) => v.finalize().into(),
            Self::Blake3(v) => Hasher::finalize(v).into(),
            Self::Sha1(v) => Hasher::finalize(v).into(),
            Self::Sha256(v) => Hasher::finalize(v).into(),
        }
    }
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

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&hex::encode(self.0))
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
// Empty Hash
// =======================

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EmptyHash;

impl AsRef<[u8]> for EmptyHash {
    fn as_ref(&self) -> &[u8] {
        &[]
    }
}

impl fmt::Debug for EmptyHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "empty")
    }
}

impl fmt::Display for EmptyHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("empty")
    }
}

impl Serialize for EmptyHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for EmptyHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <()>::deserialize(deserializer)?;
        Ok(Self)
    }
}

impl PartialEq<str> for EmptyHash {
    fn eq(&self, _: &str) -> bool {
        true
    }
}

impl PartialEq<&str> for EmptyHash {
    fn eq(&self, _: &&str) -> bool {
        true
    }
}

impl HashValue for EmptyHash {
    const NAME: &'static str = "empty";

    type Algorithm = EmptyAlgorithm;

    fn from_bytes(_: &[u8]) -> Option<Self> {
        Some(Self)
    }
}

impl From<EmptyHash> for Hash {
    fn from(_: EmptyHash) -> Self {
        Hash::Empty(EmptyHash)
    }
}

// =======================
// Empty Algorithm
// =======================

pub struct EmptyAlgorithm;

pub struct EmptyHasher;

impl HashAlgorithm for EmptyAlgorithm {
    const NAME: &'static str = "empty";

    type Value = EmptyHash;

    type Hasher = EmptyHasher;

    fn create() -> Self::Hasher {
        EmptyHasher
    }
}

impl Hasher for EmptyHasher {
    type Value = EmptyHash;

    fn update(&mut self, _: &[u8]) {}

    fn finalize(self) -> Self::Value {
        EmptyHash
    }
}

// =======================
// Algorithms
// =======================

pub struct Blake3Algorithm;

pub struct Sha1Algorithm;

pub struct Sha256Algorithm;

// =======================
// Hash Types
// =======================

impl_hash_value!(Blake3Hash, 32, "blake3", Blake3Algorithm, Blake3);

impl_hash_value!(Sha1Hash, 20, "sha1", Sha1Algorithm, Sha1);

impl_hash_value!(Sha256Hash, 32, "sha256", Sha256Algorithm, Sha256);

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
        Blake3Hash(*blake3::Hasher::finalize(&self).as_bytes())
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
        <Sha1 as Digest>::update(self, data);
    }

    fn finalize(self) -> Self::Value {
        Sha1Hash(<Sha1 as Digest>::finalize(self).into())
    }
}

// =======================
// Sha256
// =======================

use sha2::{Digest as Sha2Digest, Sha256};

impl HashAlgorithm for Sha256Algorithm {
    const NAME: &'static str = "sha256";

    type Value = Sha256Hash;

    type Hasher = Sha256;

    fn create() -> Self::Hasher {
        Sha256::new()
    }
}

impl Hasher for Sha256 {
    type Value = Sha256Hash;

    fn update(&mut self, data: &[u8]) {
        <Sha256 as Sha2Digest>::update(self, data);
    }

    fn finalize(self) -> Self::Value {
        Sha256Hash(<Sha256 as Sha2Digest>::finalize(self).into())
    }
}
