//! Typed routing of Taao/Tao events, without interpreting UI interactions.
//!
//! Observers register normally using `On<window::MouseInput>`,
//! `On<window::Resized>`, or `On<device::MouseMotion>`.
//! `TaaoEventSource` forwards each backend input through the event handle:
//! its original `taao::event::EventKind` first, followed by its typed event.
//! These are separate queued events; other producers may enqueue between them.
//! Manually emitted events are not routed again.

pub mod device;
pub mod lifecycle;
mod routing;
mod source;
pub mod window;

pub use source::TaaoEventSource;
