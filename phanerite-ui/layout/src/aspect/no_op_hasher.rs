// Adapted from `bevy_platform` crate.
//
// Original source:
// https://github.com/bevyengine/bevy/blob/7ea4827e16d5d50d48e22d9a249c554ad073f38a/crates/bevy_platform/src/hash.rs#L163-L197
//
// Copyright (c) 2026 Bevy Engine developers

use std::hash::{BuildHasher, Hasher};

/// [`BuildHasher`] for types that already contain a high-quality hash.
///
/// This is copied from [`bevy_platform::hash::NoOpHash`](https://docs.rs/bevy_platform/latest/bevy_platform/hash/struct.NoOpHash.html).
#[derive(Clone, Default)]
pub struct NoOpHash;

impl BuildHasher for NoOpHash {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher(0)
    }
}

#[doc(hidden)]
pub struct NoOpHasher(u64);

// This is for types that already contain a high-quality hash and want to skip
// re-hashing that hash.
impl Hasher for NoOpHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        // This should never be called by consumers. Prefer to call `write_u64` instead.
        // Don't break applications (slower fallback, just check in test):
        self.0 = bytes.iter().fold(self.0, |hash, b| {
            hash.rotate_left(8).wrapping_add(*b as u64)
        });
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}
