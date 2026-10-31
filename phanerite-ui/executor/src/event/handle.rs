//! Unified submission handle for events and deferred observer changes.

use super::{
    Event, ObserverCallback, ObserverId, observer::erase_send_observer, registry::ObserverRegistry,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub(super) type ObserverCommand = Box<dyn FnOnce(&mut ObserverRegistry) + Send>;

/// Cloneable, thread-safe handle for event emission and observer registration.
/// Observer changes take effect between event dispatches.
/// Queued callbacks must be Send; their asynchronous futures need not be Send.
/// Submission success means queued, not yet applied. Commands are FIFO, but
/// event input uses a separate queue; there is no total order across both.
/// Observer commands use an unbounded queue; custom events use a bounded queue.
#[derive(Clone)]
pub struct EventHandle {
    pub(super) commands: async_channel::Sender<ObserverCommand>,
    pub(super) sender: async_channel::Sender<super::payload::ErasedEvent>,
    pub(super) registry: u64,
    pub(super) next_observer: Arc<AtomicU64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserveError {
    DispatcherDropped,
    WrongDispatcher,
}

impl std::fmt::Display for ObserveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DispatcherDropped => "event dispatcher has been dropped",
            Self::WrongDispatcher => "observer belongs to another dispatcher",
        })
    }
}

impl std::error::Error for ObserveError {}

impl EventHandle {
    /// Reserves an ID immediately and queues registration in FIFO order.
    pub fn observe<E, F, M: 'static>(&self, callback: F) -> Result<ObserverId, ObserveError>
    where
        E: Event,
        F: ObserverCallback<E, M> + Send,
    {
        let sequence = self
            .next_observer
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("observer ID space exhausted");
        let id = ObserverId {
            registry: self.registry,
            sequence,
        };
        let observer = erase_send_observer(id, callback);
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

/// Failure to enqueue an event. The rejected event is dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitError {
    Full,
    Closed,
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Full => "event queue is full",
            Self::Closed => "event queue is closed",
        })
    }
}

impl std::error::Error for EmitError {}

use super::payload::ErasedEvent;
use async_channel::TrySendError;
impl EventHandle {
    /// Waits for capacity. Completion means enqueued, not handled.
    ///
    /// Inside this dispatcher's observers use `try_emit`: waiting for capacity
    /// would deadlock if their own dispatcher queue is full.
    pub async fn emit<E: Event>(&self, event: E) -> Result<(), EmitError> {
        self.sender
            .send(ErasedEvent::new(event))
            .await
            .map_err(|_| EmitError::Closed)
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

    /// Closes custom input. Queued events drain; Taao input remains active.
    pub fn close(&self) {
        self.sender.close();
    }
}
