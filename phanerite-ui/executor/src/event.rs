//! Generic mechanisms for type-directed asynchronous event observation.
//!
//! Event definitions, producers, and observer registration belong to callers.
//! The source merges Taao input with explicitly emitted custom events.
//! No concrete UI events or observers are registered by this module.

mod dispatcher;
mod handle;
mod observer;
mod on;
mod payload;
mod registry;
mod source;

pub use dispatcher::EventDispatcher;
pub use handle::{EmitError, EventHandle, ObserveError};
pub use observer::{ObserverCallback, ObserverFuture, ObserverId};
pub use on::On;
pub use payload::Event;
pub use phanerite_event_macros::Event;
pub use source::{EventSource, SourceEvent};

pub(crate) use payload::ErasedEvent;
