use async_lock::RwLock;
use serde::{Deserialize, Deserializer};

// `async_lock::RwLock` 的反序列化，`serde` 只提供了 `std` 锁的实现
//
// 显式声明 `#[serde(with = "crate::utils::lock")]` 即可。
//
// 这里只有反序列化：序列化是同步的，取读锁只能阻塞线程，
// 而写者优先的锁一旦有写者在排队就会把阻塞变成死锁。
// 需要序列化时改为在锁内取快照，参见
// [`Authentication::serialize()`](crate::auth::Authentication::serialize)
/// Deserialization for `async_lock::RwLock`; `serde` only implements this for
/// the `std` locks
///
/// Just declare `#[serde(with = "crate::utils::lock")]` explicitly.
///
/// Only deserialization is provided: serialization is synchronous, so taking
/// the read lock could only block the thread, and with a writer-preferring
/// lock that blocking turns into a deadlock as soon as a writer is queued.
/// When serialization is needed, take a snapshot inside the lock instead; see
/// [`Authentication::serialize()`](crate::auth::Authentication::serialize)
pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<RwLock<T>, D::Error> {
    Ok(RwLock::new(T::deserialize(deserializer)?))
}
