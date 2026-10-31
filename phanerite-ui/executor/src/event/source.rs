//! Waker-driven merging of Taao input and explicitly emitted custom events.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_channel::Receiver;
use futures_lite::Stream;
use taao::event::{Event as TaaoEvent, EventKind};

use super::{Event, payload::ErasedEvent};

// Backend events pass through unchanged, without UI-specific translation.
impl Event for EventKind {}

/// An input payload from either Taao or the custom-event queue.
pub struct SourceEvent {
    pub(super) event: ErasedEvent,
    pub(super) routed: Option<ErasedEvent>,
}

impl SourceEvent {
    /// Inspects the payload using its original event type.
    pub fn event<E: Event>(&self) -> Option<&E> {
        self.event
            .downcast_ref::<E>()
            .or_else(|| self.routed.as_ref()?.downcast_ref::<E>())
    }
}

type TaaoStream = Pin<Box<dyn Stream<Item = EventKind>>>;

/// The sole merged input consumed by an event dispatcher.
///
/// Construct together with its dispatcher and handle via `EventDispatcher::channel`.
/// Both inputs register the same task waker. Idle sources return Pending without
/// waking themselves. Ready inputs alternate priority; ended inputs are retired.
/// The source ends only when both inputs end. It owns no sender to its own queue.
pub struct EventSource {
    pub(super) registry: u64,
    taao: Option<TaaoStream>,
    custom: Option<Pin<Box<Receiver<ErasedEvent>>>>,
    custom_first: bool,
}

impl EventSource {
    pub(super) fn from_receiver(receiver: Receiver<ErasedEvent>, registry: u64) -> Self {
        Self {
            registry,
            taao: None,
            custom: Some(Box::pin(receiver)),
            custom_first: false,
        }
    }

    /// Attaches the application's sole external event stream.
    /// Panics if a Taao input is already attached.
    pub fn with_taao(mut self, taao: TaaoEvent) -> Self {
        assert!(self.taao.is_none(), "Taao input already attached");
        self.taao = Some(Box::pin(taao));
        self
    }

    fn poll_taao(&mut self, cx: &mut Context<'_>) -> Poll<Option<SourceEvent>> {
        let Some(taao) = &mut self.taao else {
            return Poll::Ready(None);
        };
        match taao.as_mut().poll_next(cx) {
            Poll::Ready(Some(event)) => Poll::Ready(Some(SourceEvent {
                routed: crate::tao_event::route(&event),
                event: ErasedEvent::new(event),
            })),
            Poll::Ready(None) => {
                self.taao = None;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_custom(&mut self, cx: &mut Context<'_>) -> Poll<Option<SourceEvent>> {
        let Some(custom) = &mut self.custom else {
            return Poll::Ready(None);
        };
        match custom.as_mut().poll_next(cx) {
            Poll::Ready(Some(event)) => Poll::Ready(Some(SourceEvent {
                event,
                routed: None,
            })),
            Poll::Ready(None) => {
                self.custom = None;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Stream for EventSource {
    type Item = SourceEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        for custom in [this.custom_first, !this.custom_first] {
            let result = if custom {
                this.poll_custom(cx)
            } else {
                this.poll_taao(cx)
            };
            if let Poll::Ready(Some(event)) = result {
                this.custom_first = !custom;
                return Poll::Ready(Some(event));
            }
        }
        if this.taao.is_none() && this.custom.is_none() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        future::Future,
        rc::Rc,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        task::{Wake, Waker},
    };

    use futures_lite::{StreamExt, future::block_on};

    use super::*;
    use crate::event::{EventDispatcher, EventHandle, On};

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

    // Exercise the Taao input without constructing an OS event loop.
    fn source() -> (
        EventDispatcher,
        EventSource,
        EventHandle,
        async_channel::Sender<EventKind>,
    ) {
        let (dispatcher, mut source, custom) = EventDispatcher::channel(4);
        let (taao, receiver) = async_channel::bounded(4);
        source.taao = Some(Box::pin(receiver));
        (dispatcher, source, custom, taao)
    }

    fn backend_event() -> EventKind {
        EventKind::Tao(tao::event::Event::MainEventsCleared)
    }

    #[test]
    fn idle_source_does_not_self_wake_and_both_inputs_wake_it() {
        let (_dispatcher, mut source, custom, taao) = source();
        let wakes = Arc::new(WakeCount::default());
        let waker = Waker::from(wakes.clone());
        let mut cx = Context::from_waker(&waker);

        assert!(Pin::new(&mut source).poll_next(&mut cx).is_pending());
        assert_eq!(wakes.0.load(Ordering::Relaxed), 0);
        custom.try_emit(TestEvent(7)).unwrap();
        assert!(wakes.0.load(Ordering::Relaxed) > 0);
        let Poll::Ready(Some(event)) = Pin::new(&mut source).poll_next(&mut cx) else {
            panic!("custom input did not become ready");
        };
        assert_eq!(event.event::<TestEvent>().unwrap().0, 7);

        assert!(Pin::new(&mut source).poll_next(&mut cx).is_pending());
        let before = wakes.0.load(Ordering::Relaxed);
        taao.try_send(backend_event()).unwrap();
        assert!(wakes.0.load(Ordering::Relaxed) > before);
        let Poll::Ready(Some(event)) = Pin::new(&mut source).poll_next(&mut cx) else {
            panic!("Taao input did not become ready");
        };
        assert!(event.event::<EventKind>().is_some());
    }

    #[test]
    fn ready_inputs_alternate_priority() {
        block_on(async {
            let (_dispatcher, mut source, custom, taao) = source();
            for value in 0..2 {
                taao.try_send(backend_event()).unwrap();
                custom.try_emit(TestEvent(value)).unwrap();
            }
            for value in 0..2 {
                assert!(source.next().await.unwrap().event::<EventKind>().is_some());
                assert_eq!(
                    source.next().await.unwrap().event::<TestEvent>().unwrap().0,
                    value
                );
            }
        });
    }

    #[test]
    fn closing_either_input_keeps_the_other_alive_and_both_closed_ends() {
        block_on(async {
            for close_custom_first in [true, false] {
                let (_dispatcher, mut source, custom, taao) = source();
                let wakes = Arc::new(WakeCount::default());
                let waker = Waker::from(wakes.clone());
                let mut cx = Context::from_waker(&waker);
                if close_custom_first {
                    custom.close();
                } else {
                    taao.close();
                }
                assert!(Pin::new(&mut source).poll_next(&mut cx).is_pending());
                assert_eq!(wakes.0.load(Ordering::Relaxed), 0);
                if close_custom_first {
                    taao.try_send(backend_event()).unwrap();
                } else {
                    custom.try_emit(TestEvent(1)).unwrap();
                }
                assert!(source.next().await.is_some());
                custom.close();
                taao.close();
                assert!(source.next().await.is_none());
                assert!(source.next().await.is_none());
            }
        });
    }

    #[test]
    fn pump_sleeps_when_idle_and_dispatches_both_original_payload_types() {
        let (_dispatcher, source, custom, taao) = source();
        let mut dispatcher = _dispatcher;
        let count = Rc::new(Cell::new(0));
        let custom_count = count.clone();
        dispatcher.observe(move |_: On<TestEvent>| custom_count.set(custom_count.get() + 1));
        let backend_count = count.clone();
        dispatcher.observe(move |_: On<EventKind>| backend_count.set(backend_count.get() + 1));

        let wakes = Arc::new(WakeCount::default());
        let waker = Waker::from(wakes.clone());
        let mut cx = Context::from_waker(&waker);
        let mut pump = std::pin::pin!(dispatcher.pump(source));
        assert!(pump.as_mut().poll(&mut cx).is_pending());
        assert_eq!(wakes.0.load(Ordering::Relaxed), 0);
        custom.try_emit(TestEvent(1)).unwrap();
        taao.try_send(backend_event()).unwrap();
        let before = wakes.0.load(Ordering::Relaxed);
        assert!(pump.as_mut().poll(&mut cx).is_pending());
        assert_eq!(count.get(), 2);
        assert_eq!(wakes.0.load(Ordering::Relaxed), before);
        custom.close();
        taao.close();
        assert!(pump.as_mut().poll(&mut cx).is_ready());
    }

    #[test]
    fn pump_routes_backend_events_in_order_and_keeps_custom_events_unchanged() {
        block_on(async {
            use crate::tao_event::lifecycle::RedrawRequested;
            use std::cell::RefCell;

            let (_dispatcher, source, custom, taao) = source();
            // SAFETY: dummy IDs are only compared and passed as test event data,
            // never supplied to any Tao window or event-loop operation.
            let window_id = unsafe { tao::window::WindowId::dummy() };
            let seen = Rc::new(RefCell::new(Vec::new()));
            let mut dispatcher = _dispatcher;
            let raw_seen = seen.clone();
            dispatcher.observe(move |_: On<EventKind>| raw_seen.borrow_mut().push("raw"));
            let typed_seen = seen.clone();
            dispatcher.observe(async move |event: On<RedrawRequested>| {
                assert_eq!(event.window_id, window_id);
                futures_lite::future::yield_now().await;
                typed_seen.borrow_mut().push("typed");
            });
            taao.try_send(EventKind::Tao(tao::event::Event::RedrawRequested(
                window_id,
            )))
            .unwrap();
            taao.close();
            // Emitting the typed event must dispatch it once, without routing again.
            custom.try_emit(RedrawRequested { window_id }).unwrap();
            custom.close();
            dispatcher.pump(source).await;
            assert_eq!(*seen.borrow(), ["raw", "typed", "typed"]);
        });
    }

    #[test]
    fn scale_factor_input_is_inspectable_and_observable_as_owned_typed_data() {
        block_on(async {
            use crate::tao_event::window::ScaleFactorChanged;
            // SAFETY: this ID is test data only and never used in a Tao operation.
            let window_id = unsafe { tao::window::WindowId::dummy() };
            let size = tao::dpi::PhysicalSize::new(1600, 900);
            let (_dispatcher, mut source, custom, taao) = source();
            taao.try_send(EventKind::TaoScaleFactorChanged {
                window_id,
                scale_factor: 1.5,
                suggested_inner_size: size,
            })
            .unwrap();
            taao.close();
            custom.close();
            let input = source.next().await.unwrap();
            assert!(input.event::<EventKind>().is_some());
            let typed = input.event::<ScaleFactorChanged>().unwrap();
            assert_eq!(
                (
                    typed.window_id,
                    typed.scale_factor,
                    typed.suggested_inner_size
                ),
                (window_id, 1.5, size)
            );

            let (_dispatcher, source, custom, taao) = self::source();
            taao.try_send(EventKind::TaoScaleFactorChanged {
                window_id,
                scale_factor: 1.5,
                suggested_inner_size: size,
            })
            .unwrap();
            taao.close();
            custom.close();
            let seen = Rc::new(Cell::new(false));
            let sink = seen.clone();
            let mut dispatcher = _dispatcher;
            dispatcher.observe(move |event: On<ScaleFactorChanged>| {
                assert_eq!(event.scale_factor, 1.5);
                sink.set(true);
            });
            dispatcher.pump(source).await;
            assert!(seen.get());
        });
    }
}
