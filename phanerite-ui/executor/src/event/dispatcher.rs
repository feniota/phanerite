//! Explicit dispatch and asynchronous queue driving.

use futures_lite::Stream;
use std::{pin::Pin, task::Poll};

use super::{
    Event, EventSource, ObserverCallback, ObserverId,
    handle::{EventHandle, ObserverCommand},
    payload::ErasedEvent,
    registry::ObserverRegistry,
};

/// Invokes matching observers sequentially in registration order, across `.await`.
///
/// A new dispatcher is empty. Callers define events, register observers, submit
/// payloads, and decide when to drive it. This type owns no application runtime.
/// Observer captures and futures may be `!Send`; payloads must be Send + Sync.
pub struct EventDispatcher {
    registry: ObserverRegistry,
    changes: Pin<Box<async_channel::Receiver<ObserverCommand>>>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        let registry = ObserverRegistry::new();
        let (_sender, receiver) = async_channel::unbounded();
        Self {
            registry,
            changes: Box::pin(receiver),
        }
    }
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a dispatcher, its input source, and one unified submission handle.
    /// Panics if capacity is zero. The handle does not keep either consumer alive.
    pub fn channel(capacity: usize) -> (Self, EventSource, EventHandle) {
        let mut dispatcher = Self::new();
        let (commands, changes) = async_channel::unbounded();
        let (sender, events) = async_channel::bounded(capacity);
        dispatcher.changes = Box::pin(changes);
        let handle = EventHandle {
            commands,
            sender,
            registry: dispatcher.registry.id,
            next_observer: dispatcher.registry.next_observer.clone(),
        };
        let source = EventSource::from_receiver(events, dispatcher.registry.id);
        (dispatcher, source, handle)
    }

    fn apply_commands(&mut self) {
        // A finite snapshot prevents producers from keeping this call busy forever.
        for _ in 0..self.changes.len() {
            let Ok(command) = self.changes.try_recv() else {
                break;
            };
            command(&mut self.registry);
        }
    }

    /// Idempotent registration; observing an event also registers it automatically.
    pub fn register_event<E: Event>(&mut self) -> &mut Self {
        self.apply_commands();
        self.registry.register_event::<E>();
        self
    }

    /// Infers the event type from the callback's `On<E>` argument.
    pub fn observe<E, F, M: 'static>(&mut self, callback: F) -> &mut Self
    where
        E: Event,
        F: ObserverCallback<E, M>,
    {
        self.add_observer(callback);
        self
    }

    /// Registers a listener and returns a handle for explicit removal.
    pub fn add_observer<E, F, M: 'static>(&mut self, callback: F) -> ObserverId
    where
        E: Event,
        F: ObserverCallback<E, M>,
    {
        self.apply_commands();
        self.registry.insert(callback)
    }

    /// Returns false for removed handles or handles from another dispatcher.
    pub fn remove_observer(&mut self, id: ObserverId) -> bool {
        self.apply_commands();
        self.registry.remove(id)
    }

    /// Invokes all matching observers before returning. Unobserved events are ignored.
    pub async fn trigger<E: Event>(&mut self, event: E) {
        self.apply_commands();
        self.registry.dispatch(ErasedEvent::new(event)).await;
        self.apply_commands();
    }

    /// Waits for input or observer commands until the event source ends.
    /// Each raw event and its typed route share a stable observer configuration.
    /// Applies up to 64 commands before polling the next input for fairness.
    /// No input means Pending; no timer or polling loop is used to wait for work.
    pub async fn pump(mut self, mut source: EventSource) {
        assert_eq!(
            source.registry, self.registry.id,
            "event source belongs to another dispatcher"
        );
        let mut processed = 0;
        loop {
            let input = std::future::poll_fn(|cx| {
                let mut changed = false;
                // Bound command processing so event traffic also makes progress.
                for _ in 0..64 {
                    match self.changes.as_mut().poll_next(cx) {
                        Poll::Ready(Some(command)) => {
                            command(&mut self.registry);
                            changed = true;
                        }
                        _ => break,
                    }
                }
                match Pin::new(&mut source).poll_next(cx) {
                    Poll::Ready(event) => Poll::Ready(Some(event)),
                    Poll::Pending if changed => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            })
            .await;
            let event = match input {
                Some(Some(event)) => event,
                Some(None) => break,
                None => {
                    futures_lite::future::yield_now().await;
                    continue;
                }
            };
            self.registry.dispatch(event.event).await;
            if let Some(routed) = event.routed {
                self.registry.dispatch(routed).await;
            }
            processed += 1;
            // Cooperate under sustained traffic, never self-wake while idle.
            if processed == 64 {
                futures_lite::future::yield_now().await;
                processed = 0;
            }
        }
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        self.changes.close();
        while self.changes.try_recv().is_ok() {}
    }
}
