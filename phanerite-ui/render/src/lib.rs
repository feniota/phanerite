//! GPU drawing of layout snapshots, independent of native windows and executors.
//!
//! The host supplies a device, queue, target and coherent physical/logical
//! viewport. This crate encodes drawing; acquiring a target, submitting commands
//! and presenting frames remain the host's responsibility.

mod content;
#[cfg(feature = "layout-debug")]
mod layout;
mod viewport;

pub use content::RenderContent;
#[cfg(feature = "layout-debug")]
pub use layout::{LayoutDebugRenderer, LayoutRenderError};
pub use viewport::RenderViewport;
