//! Asynchronous forwarding from Taao into the generic event queue.

use std::pin::Pin;

use futures_lite::{Stream, StreamExt};
use taao::event::{Event as TaaoEvent, EventKind};

use crate::{
    Context,
    event::{EmitError, EventHandle},
};

/// Polls backend input and emits raw and typed events into an event dispatcher.
///
/// Run concurrently with the dispatcher pump, not inside one of its observers:
/// forwarding waits for queue capacity. Ending the backend stream releases this
/// producer without closing other producers. Closing event input stops forwarding
/// with `EmitError::Closed` when the next event is submitted.
pub struct TaaoEventSource<S = TaaoEvent> {
    source: Pin<Box<S>>,
    events: EventHandle,
}

impl<S: Stream<Item = EventKind>> TaaoEventSource<S> {
    /// Clones the event handle from the application context without borrowing it.
    pub fn new(source: S, cx: &Context) -> Self {
        Self::from_handle(source, cx.events.clone())
    }

    fn from_handle(source: S, events: EventHandle) -> Self {
        Self {
            source: Box::pin(source),
            events,
        }
    }

    /// Emits each raw input before its typed route, preserving backend order.
    /// Completion means enqueued, not that observers have finished running.
    pub async fn run(mut self) -> Result<(), EmitError> {
        let mut processed = 0;
        while let Some(event) = self.source.next().await {
            let routed = super::routing::route(&event);
            self.events.emit(event).await?;
            if let Some(routed) = routed {
                self.events.emit_erased(routed).await?;
            }
            processed += 1;
            if processed == 64 {
                futures_lite::future::yield_now().await;
                processed = 0;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use futures_lite::{
        future::{block_on, poll_once, yield_now, zip},
        stream,
    };

    use super::*;
    use crate::{
        event::{Event, EventDispatcher, On},
        tao_event::{lifecycle::RedrawRequested, window::ScaleFactorChanged},
    };

    #[derive(Event)]
    struct Custom;

    #[test]
    fn forwards_raw_then_typed_with_backpressure_without_rerouting_manual_events() {
        block_on(async {
            // SAFETY: this ID is only carried as test data, never used in Tao operations.
            let window_id = unsafe { tao::window::WindowId::dummy() };
            let (mut dispatcher, source, handle) = EventDispatcher::channel(1);
            let seen = Rc::new(RefCell::new(Vec::new()));
            let raw_seen = seen.clone();
            dispatcher.observe(async move |_: On<EventKind>| {
                yield_now().await;
                raw_seen.borrow_mut().push("raw");
            });
            let typed_seen = seen.clone();
            dispatcher.observe(move |event: On<RedrawRequested>| {
                assert_eq!(event.window_id, window_id);
                typed_seen.borrow_mut().push("typed");
            });
            let input = || EventKind::Tao(tao::event::Event::RedrawRequested(window_id));
            let backend =
                TaaoEventSource::from_handle(stream::iter([input(), input()]), handle.clone());
            let mut forwarding = Box::pin(backend.run());
            // One raw event fits, then forwarding waits for capacity for its typed route.
            assert!(poll_once(forwarding.as_mut()).await.is_none());
            zip(dispatcher.pump(source), async {
                forwarding.await.unwrap();
                // Emitted backend types are ordinary events, not another routing entry.
                handle.emit(input()).await.unwrap();
                handle.emit(RedrawRequested { window_id }).await.unwrap();
                handle.close();
            })
            .await;
            assert_eq!(
                *seen.borrow(),
                ["raw", "typed", "raw", "typed", "raw", "typed"]
            );
        });
    }

    #[test]
    fn backend_completion_leaves_other_producers_open_and_scale_data_owned() {
        block_on(async {
            // SAFETY: this ID is only compared as test data.
            let window_id = unsafe { tao::window::WindowId::dummy() };
            let size = tao::dpi::PhysicalSize::new(1600, 900);
            let input = EventKind::TaoScaleFactorChanged {
                window_id,
                scale_factor: 1.5,
                suggested_inner_size: size,
            };
            let (mut dispatcher, source, handle) = EventDispatcher::channel(3);
            let seen = Rc::new(RefCell::new(Vec::new()));
            let typed_seen = seen.clone();
            dispatcher.observe(move |event: On<ScaleFactorChanged>| {
                assert_eq!(event.window_id, window_id);
                assert_eq!(event.scale_factor, 1.5);
                assert_eq!(event.suggested_inner_size, size);
                typed_seen.borrow_mut().push("scale");
            });
            let custom_seen = seen.clone();
            dispatcher.observe(move |_: On<Custom>| custom_seen.borrow_mut().push("custom"));
            TaaoEventSource::from_handle(stream::iter([input]), handle.clone())
                .run()
                .await
                .unwrap();
            handle.emit(Custom).await.unwrap();
            drop(handle);
            dispatcher.pump(source).await;
            assert_eq!(*seen.borrow(), ["scale", "custom"]);
        });
    }

    #[test]
    fn closed_queue_rejects_backend_forwarding() {
        block_on(async {
            let (dispatcher, source, handle) = EventDispatcher::channel(1);
            handle.close();
            let backend = TaaoEventSource::from_handle(
                stream::iter([EventKind::Tao(tao::event::Event::MainEventsCleared)]),
                handle,
            );
            assert_eq!(backend.run().await, Err(EmitError::Closed));
            dispatcher.pump(source).await;
        });
    }

    #[test]
    fn closing_full_queue_unblocks_forwarding_and_drains_queued_event() {
        block_on(async {
            // SAFETY: this ID is test data only.
            let window_id = unsafe { tao::window::WindowId::dummy() };
            let (_dispatcher, mut source, handle) = EventDispatcher::channel(1);
            let backend = TaaoEventSource::from_handle(
                stream::iter([EventKind::Tao(tao::event::Event::RedrawRequested(
                    window_id,
                ))]),
                handle.clone(),
            );
            let mut forwarding = Box::pin(backend.run());
            assert!(poll_once(forwarding.as_mut()).await.is_none());
            handle.close();
            assert_eq!(forwarding.await, Err(EmitError::Closed));
            assert!(source.next().await.unwrap().event::<EventKind>().is_some());
            assert!(source.next().await.is_none());
        });
    }
}
