//! Retained UI data structures for PhaUI.
//!
//! Taffy owns node identity, topology and layout data. Components are concrete
//! Rust values associated with their root nodes in a sparse secondary map.
//! This crate does not create OS windows or implement rendering, events,
//! reactive scheduling or component lifecycle callbacks.

mod error;
mod node;
mod tree;

pub use error::{TreeError, TreeResult};
pub use node::{NodeKind, NodeMetadata};
pub use tree::{NoComponents, RemovedSubtree, Window};

pub use taffy;
pub use taffy::NodeId;

/// Marker contract implemented by a compiler-generated closed component set.
///
/// The generated enum remains an ordinary, statically typed Rust value. This
/// trait gives the component store a stable bound to extend when lifecycle
/// dispatch is designed; it does not imply dynamic dispatch or type erasure.
pub trait Component {}

impl Component for NoComponents {}
