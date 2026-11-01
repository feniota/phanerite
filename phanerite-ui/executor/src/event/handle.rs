//! Unified submission handle for events and deferred observer changes.

use super::{
    EmitError, Event, ObserveError, ObserverCallback, ObserverId, observer::erase_observer,
    registry::ObserverRegistry,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub(super) type ObserverCommand = Box<dyn FnOnce(&mut ObserverRegistry)>;

/// Cloneable, thread-local handle for event emission and observer registration.
/// Observer changes take effect between event dispatches.
/// Queued callbacks and their asynchronous futures need not be `Send`.
/// Submission success means queued, not yet applied. Commands are FIFO, but
/// event input uses a separate queue; there is no total order across both.
/// Observer commands use an unbounded queue; custom events use a bounded queue.
#[derive(Clone)]
pub struct EventHandle {
    pub(super) commands: async_channel::Sender<ObserverCommand>,
    pub(super) sender: async_channel::Sender<ErasedEvent>,
    pub(super) registry: u64,
    pub(super) next_observer: Arc<AtomicU64>,
}

impl EventHandle {
    /// Reserves an ID immediately and queues registration in FIFO order.
    pub fn observe<E, F, M: 'static>(&self, callback: F) -> Result<ObserverId, ObserveError>
    where
        E: Event,
        F: ObserverCallback<E, M>,
    {
        let sequence = self
            .next_observer
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("observer ID space exhausted");
        let id = ObserverId {
            registry: self.registry,
            sequence,
        };
        let observer = erase_observer(id, callback);
        self.send(Box::new(move |registry| {
            registry.insert_erased(std::any::TypeId::of::<E>(), observer)
        }))?;
        Ok(id)
    }

    /// Queues removal. Removing an already removed observer is a no-op.
    pub fn remove_observer(&self, id: ObserverId) -> Result<(), ObserveError> {
        if id.registry != self.registry {
            return Err(ObserveError::WrongDispatcher);
        }
        self.send(Box::new(move |registry| {
            registry.remove(id);
        }))
    }

    fn send(&self, command: ObserverCommand) -> Result<(), ObserveError> {
        self.commands
            .try_send(command)
            .map_err(|_| ObserveError::DispatcherDropped)
    }
}

use super::payload::ErasedEvent;
use async_channel::TrySendError;
impl EventHandle {
    /// Waits for capacity. Completion means enqueued, not handled.
    ///
    /// Inside this dispatcher's observers use `try_emit`: waiting for capacity
    /// would deadlock if their own dispatcher queue is full.
    pub async fn emit<E: Event>(&self, event: E) -> Result<(), EmitError> {
        self.emit_erased(ErasedEvent::new(event)).await
    }

    pub(crate) async fn emit_erased(&self, event: ErasedEvent) -> Result<(), EmitError> {
        self.sender.send(event).await.map_err(|_| EmitError::Closed)
    }

    /// Enqueues without waiting, including from inside an observer.
    pub fn try_emit<E: Event>(&self, event: E) -> Result<(), EmitError> {
        self.sender
            .try_send(ErasedEvent::new(event))
            .map_err(|error| match error {
                TrySendError::Full(_) => EmitError::Full,
                TrySendError::Closed(_) => EmitError::Closed,
            })
    }

    /// Closes event input for every producer. Queued events drain before pump ends.
    pub fn close(&self) {
        self.sender.close();
    }
}
