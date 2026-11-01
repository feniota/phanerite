//! Waker-driven input for explicitly emitted events.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_channel::Receiver;
use futures_lite::Stream;

use super::{Event, payload::ErasedEvent};

/// An owned payload from the event queue.
pub struct QueuedEvent {
    pub(super) event: ErasedEvent,
}

impl QueuedEvent {
    /// Inspects the payload using its original event type.
    pub fn event<E: Event>(&self) -> Option<&E> {
        self.event.downcast_ref::<E>()
    }
}

/// The event queue consumed by a dispatcher.
///
/// Construct with `EventDispatcher::channel`. Idle input returns Pending without
/// self-waking. Closing the queue drains pending events before ending the stream;
/// dropping every producer also ends it. The receiver owns no sender to its queue.
pub struct EventReceiver {
    pub(super) registry: u64,
    receiver: Pin<Box<Receiver<ErasedEvent>>>,
}

impl EventReceiver {
    pub(super) fn from_receiver(receiver: Receiver<ErasedEvent>, registry: u64) -> Self {
        Self {
            registry,
            receiver: Box::pin(receiver),
        }
    }
}

impl Stream for EventReceiver {
    type Item = QueuedEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut()
            .receiver
            .as_mut()
            .poll_next(cx)
            .map(|event| event.map(|event| QueuedEvent { event }))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        task::{Wake, Waker},
    };

    use super::*;
    use crate::event::EventDispatcher;

    struct TestEvent(u32);
    impl Event for TestEvent {}

    #[derive(Default)]
    struct WakeCount(AtomicUsize);

    impl Wake for WakeCount {
        fn wake(self: Arc<Self>) {
            self.wake_by_ref();
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn idle_receiver_sleeps_and_emit_wakes_it() {
        let (_dispatcher, mut receiver, handle) = EventDispatcher::channel(1);
        let wakes = Arc::new(WakeCount::default());
        let waker = Waker::from(wakes.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(Pin::new(&mut receiver).poll_next(&mut cx).is_pending());
        assert_eq!(wakes.0.load(Ordering::Relaxed), 0);
        handle.try_emit(TestEvent(7)).unwrap();
        assert!(wakes.0.load(Ordering::Relaxed) > 0);
        handle.close();

        let Poll::Ready(Some(event)) = Pin::new(&mut receiver).poll_next(&mut cx) else {
            panic!("emitted event did not become ready");
        };
        assert_eq!(event.event::<TestEvent>().unwrap().0, 7);
        assert!(matches!(
            Pin::new(&mut receiver).poll_next(&mut cx),
            Poll::Ready(None)
        ));
    }
}
