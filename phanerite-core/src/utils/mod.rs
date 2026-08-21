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
