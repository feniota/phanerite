//! Generic mechanisms for type-directed asynchronous event observation.
//!
//! Event definitions, producers, and observer registration belong to callers.
//! The source consumes explicitly emitted events without interpreting producers.
//! No concrete UI events or observers are registered by this module.

mod dispatcher;
mod error;
mod handle;
mod observer;
mod on;
mod payload;
mod receiver;
mod registry;

pub use dispatcher::EventDispatcher;
pub use error::{EmitError, ObserveError};
pub use handle::EventHandle;
pub use observer::{ObserverCallback, ObserverFuture, ObserverId};
pub use on::On;
pub use payload::Event;
pub use phanerite_ui_macro::Event;
pub use receiver::{EventReceiver, QueuedEvent};

pub(crate) use payload::ErasedEvent;
