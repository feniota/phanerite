//! Event constraints and the internal representation of queued payloads.

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

/// An owned event that can be submitted from another task or thread.
///
/// Implement explicitly, or use `#[derive(Event)]`. No reflection or component
/// registration is required. Events do not need to implement `Clone`.
pub trait Event: Any + Send + Sync {}

pub(super) type EventPayload = Arc<dyn Any + Send + Sync>;

/// Creates the runtime type key and payload together before type erasure.
pub(crate) struct ErasedEvent {
    pub(super) type_id: TypeId,
    pub(super) payload: EventPayload,
}

impl ErasedEvent {
    pub(crate) fn downcast_ref<E: Event>(&self) -> Option<&E> {
        self.payload.downcast_ref::<E>()
    }

    pub(crate) fn new<E: Event>(event: E) -> Self {
        Self {
            type_id: TypeId::of::<E>(),
            payload: Arc::new(event),
        }
    }
}
