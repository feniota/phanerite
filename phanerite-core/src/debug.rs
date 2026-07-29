#![cfg(debug_assertions)]

/// 不应该实现 Clone 但需要 clone 的类型
pub trait DebugClone {
    fn debug_clone(&self) -> Self;
}
