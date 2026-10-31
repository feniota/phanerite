//! The typed event argument passed to a registered observer.

use std::{ops::Deref, sync::Arc};

use super::{Event, ObserverId};

/// Shared, immutable event data owned by an observer invocation.
///
/// This value may be retained across `.await`. Cloning it shares the payload
/// without requiring `E: Clone`.
pub struct On<E: Event> {
    pub(super) event: Arc<E>,
    pub(super) observer: ObserverId,
}

impl<E: Event> On<E> {
    pub fn event(&self) -> &E {
        &self.event
    }
    pub fn observer(&self) -> ObserverId {
        self.observer
    }
    pub fn into_inner(self) -> Arc<E> {
        self.event
    }
}

impl<E: Event> Clone for On<E> {
    fn clone(&self) -> Self {
        Self {
            event: self.event.clone(),
            observer: self.observer,
        }
    }
}

impl<E: Event> Deref for On<E> {
    type Target = E;
    fn deref(&self) -> &E {
        self.event()
    }
}
