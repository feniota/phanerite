#![deny(clippy::unused_trait_names)]

// Lets `::phanerite_layout::…` paths emitted by our derive macros resolve inside this crate.
extern crate self as phanerite_layout;

mod aspect;
mod batch;
mod commands;
mod context;
mod node;
mod resources;
mod system;
mod world;

pub use aspect::{AnyStorage, Aspect, Storage, StorageIterator};
pub use batch::{Batch, Bundle};
pub use commands::Commands;
pub use context::{Context, SharedContext};
pub use node::NodeMetadata;
pub use phanerite_ui_macros::Aspect;
pub use system::{AspectSet, System, SystemKey};
pub use world::World;

pub use taffy;

pub use World as LayoutWindow;

#[cfg(test)]
mod tests;
