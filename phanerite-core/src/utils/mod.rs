// 跨模块共用的基础设施
//
// 这里不是杂物间，放的是被多个模块共同依赖、又不属于其中任何一个的东西。
//
// - [`container`]：全局资源容器。基于 `scc::TreeIndex`，键是 UUID v7，
//   取出来的 [`container::Guard`] 由 EBR 保护，guard 活着期间对应 entry
//   的内存不会被回收。代价是「拿到 guard」和「这个 UUID 现在还指向它」
//   是两回事，需要按当前状态并发修改时得自己做 CAS。元素不多时直接用
//   [`container::Container::snapshot`] 更省事。
// - [`hash`]：把多种摘要算法收进一个 [`hash::Hash`] 枚举，下载校验、共享
//   桶命名、清单解析共用一套类型。`Hash::Empty` 表示不要求校验。
// - [`state`]：[`Ready`](state::Ready) / [`NotReady`](state::NotReady) 两个
//   空类型，[`Instance`](crate::instance::Instance) 和各种 builder 的
//   typestate 都用它们标记。
// - [`lock`] 与 [`secret`]：serde 的 `with` 适配器。前者只提供反序列化
//   （序列化是同步的，对写者优先的锁取读锁会死锁）；后者把
//   `SecretString` 的序列化显式写出来，让「凭据被写成明文」这件事必须在
//   字段上声明才会发生。
// - [`maven`]、[`uuid`]、[`version`]：Maven 坐标、无连字符
//   UUID、版本号比较。其中 [`version::compare_versions`] 和
//   [`version::is_stable`] 都是启发式的，只供展示与排序，不要拿来做兼容性
//   判断。
//! Infrastructure shared across modules
//!
//! This is not a junk drawer: it holds the things several modules depend on
//! that do not belong to any one of them.
//!
//! - [`container`]: the container for globally shared resources. Built on
//!   `scc::TreeIndex` with UUID v7 keys; the [`container::Guard`] it hands
//!   out is protected by EBR, so the memory of the corresponding entry is not
//!   reclaimed while the guard is alive. The price is that "holding a guard"
//!   and "this UUID still points at it" are two different things, so a
//!   concurrent modification that depends on the current state has to do its
//!   own CAS. When there are few elements,
//!   [`container::Container::snapshot`] is simpler.
//! - [`hash`]: gathers several digest algorithms into one [`hash::Hash`]
//!   enum, so download verification, share-bucket naming and manifest parsing
//!   all share a single type. `Hash::Empty` means no verification is
//!   required.
//! - [`state`]: the two empty types [`Ready`](state::Ready) and
//!   [`NotReady`](state::NotReady), used as typestate markers by
//!   [`Instance`](crate::instance::Instance) and by the various builders.
//! - [`lock`] and [`secret`]: serde `with` adapters. The former only provides
//!   deserialization, because serialization is synchronous and taking a read
//!   lock on a writer-preferring lock would deadlock; the latter spells out
//!   the serialization of `SecretString` explicitly, so that "the credential
//!   gets written out in plain text" can only happen when a field declares
//!   it.
//! - [`maven`], [`uuid`] and [`version`]: Maven coordinates,
//!   hyphenless UUIDs, and version comparison. Both
//!   [`version::compare_versions`] and [`version::is_stable`] are heuristics
//!   meant for display and ordering only; do not use them to decide
//!   compatibility.

// ouroboros 展开后存在多余生命周期
// https://github.com/someguynamedjosh/ouroboros/issues/140
#[allow(clippy::extra_unused_lifetimes)]
pub mod container;

pub mod hash;
pub mod lock;
pub mod maven;
pub mod secret;
pub mod state;
pub mod uuid;
pub mod version;

pub use hash::*;
