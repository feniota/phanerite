use std::{cell::RefCell, rc::Rc, sync::Arc};

use futures_lite::future::{block_on, yield_now, zip};
use phanerite_executor::event::{EmitError, Event, EventDispatcher, On};

#[derive(Event)]
struct TestEvent(u32);

#[derive(Event)]
struct Other;

#[derive(Event)]
struct Generic<T>(T);

#[test]
fn infers_sync_async_and_function_observers_and_preserves_order() {
    block_on(async {
        let output = Rc::new(RefCell::new(Vec::new()));
        let mut events = EventDispatcher::new();
        let first = output.clone();
        events.observe(move |event: On<TestEvent>| {
            first.borrow_mut().push(event.0);
        });
        let second = output.clone();
        events.observe(async move |event: On<TestEvent>| {
            second.borrow_mut().push(10 + event.0);
            yield_now().await;
            second.borrow_mut().push(20 + event.0);
        });
        let third = output.clone();
        events.observe(move |event: On<TestEvent>| {
            third.borrow_mut().push(30 + event.0);
        });
        async fn function(event: On<TestEvent>) {
            assert_eq!(event.0, 1);
        }
        events.observe(function);
        events.observe(|_: On<Other>| -> () { panic!("wrong event type") });
        events
            .register_event::<TestEvent>()
            .register_event::<TestEvent>();
        events.trigger(TestEvent(1)).await;
        assert_eq!(*output.borrow(), [1, 11, 21, 31]);
    });
}

#[test]
fn async_observer_can_borrow_mutable_capture_across_await() {
    block_on(async {
        let output = Rc::new(RefCell::new(Vec::new()));
        let sink = output.clone();
        let mut state = Vec::new();
        let mut events = EventDispatcher::new();
        events.observe(async move |event: On<TestEvent>| {
            state.push(event.0);
            let value = state.last_mut().unwrap();
            yield_now().await;
            *value += 1;
            *sink.borrow_mut() = state.clone();
        });
        events.trigger(TestEvent(1)).await;
        events.trigger(TestEvent(2)).await;
        assert_eq!(*output.borrow(), [2, 3]);
    });
}

#[test]
fn payload_is_shared_without_clone_bound_and_can_outlive_dispatch() {
    block_on(async {
        let received = Rc::new(RefCell::new(Vec::new()));
        let mut events = EventDispatcher::new();
        for _ in 0..2 {
            let received = received.clone();
            events.observe(move |event: On<Generic<String>>| {
                received.borrow_mut().push(event.clone().into_inner());
            });
        }
        events.trigger(Generic(String::from("owned"))).await;
        drop(events);
        let received = received.borrow();
        assert!(Arc::ptr_eq(&received[0], &received[1]));
        assert_eq!(received[0].0, "owned");
    });
}

#[test]
fn handles_are_scoped_to_dispatcher_and_not_reused() {
    block_on(async {
        let mut first = EventDispatcher::new();
        let mut second = EventDispatcher::new();
        let id = first.add_observer(|_: On<TestEvent>| -> () { panic!("removed observer") });
        second.observe(|_: On<TestEvent>| {});
        assert!(!second.remove_observer(id));
        assert!(first.remove_observer(id));
        assert!(!first.remove_observer(id));
        let received = Rc::new(RefCell::new(None));
        let sink = received.clone();
        let replacement = first.add_observer(move |event: On<TestEvent>| {
            *sink.borrow_mut() = Some(event.observer());
        });
        assert_ne!(replacement, id);
        first.trigger(TestEvent(1)).await;
        assert_eq!(*received.borrow(), Some(replacement));
        first.trigger(Other).await;
    });
}

#[test]
fn mailbox_applies_backpressure_broadcasts_and_drains_on_close() {
    block_on(async {
        let (mut events, receiver, sender) = EventDispatcher::channel(1);
        let received = Rc::new(RefCell::new(Vec::new()));
        let sink = received.clone();
        events.observe(move |event: On<TestEvent>| sink.borrow_mut().push(event.0));
        sender.try_emit(TestEvent(1)).unwrap();
        assert_eq!(sender.try_emit(Other), Err(EmitError::Full));
        zip(events.pump(receiver), async {
            sender.emit(TestEvent(2)).await.unwrap();
            sender.close();
            assert_eq!(sender.emit(Other).await, Err(EmitError::Closed));
        })
        .await;
        assert_eq!(*received.borrow(), [1, 2]);
    });
}

#[test]
fn receiver_finishes_when_producers_drop_and_rejects_after_receiver_drops() {
    block_on(async {
        let (events, receiver, sender) = EventDispatcher::channel(256);
        sender.try_emit(Other).unwrap();
        drop(sender);
        events.pump(receiver).await;

        let (_events, receiver, sender) = EventDispatcher::channel(256);
        drop(receiver);
        assert_eq!(sender.try_emit(Other), Err(EmitError::Closed));
    });
}

#[test]
fn observer_can_enqueue_follow_up_without_recursive_dispatch() {
    block_on(async {
        let (mut events, receiver, sender) = EventDispatcher::channel(256);
        let received = Rc::new(RefCell::new(Vec::new()));
        let sink = received.clone();
        events.observe(move |event: On<TestEvent>| {
            sink.borrow_mut().push(event.0);
            if event.0 == 1 {
                sender.try_emit(TestEvent(2)).unwrap();
            } else {
                sender.close();
            }
        });
        events.trigger(TestEvent(1)).await;
        assert_eq!(*received.borrow(), [1]);
        events.pump(receiver).await;
        assert_eq!(*received.borrow(), [1, 2]);
    });
}

#[test]
fn commands_wake_idle_pump_and_apply_without_an_event() {
    use std::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicUsize, Ordering},
        task::{Context, Wake, Waker},
    };
    struct Wakes(AtomicUsize);
    impl Wake for Wakes {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
    let (dispatcher, receiver, sender) = EventDispatcher::channel(4);
    let commands = sender.clone();
    let wakes = Arc::new(Wakes(AtomicUsize::new(0)));
    let waker = Waker::from(wakes.clone());
    let mut cx = Context::from_waker(&waker);
    let mut pump = std::pin::pin!(dispatcher.pump(receiver));
    assert!(pump.as_mut().poll(&mut cx).is_pending());
    assert_eq!(wakes.0.load(Ordering::Relaxed), 0);
    let capture = Rc::new(());
    let state = capture.clone();
    let id = commands
        .observe(move |_: On<TestEvent>| {
            let _ = &state;
        })
        .unwrap();
    assert!(wakes.0.load(Ordering::Relaxed) > 0);
    block_on(futures_lite::future::poll_once(pump.as_mut()));
    assert_eq!(Rc::strong_count(&capture), 2);
    commands.remove_observer(id).unwrap();
    block_on(futures_lite::future::poll_once(pump.as_mut()));
    assert_eq!(Rc::strong_count(&capture), 1);
    // Finish the cooperative yield; an idle pump must then stop waking itself.
    block_on(futures_lite::future::poll_once(pump.as_mut()));
    let before = wakes.0.load(Ordering::Relaxed);
    assert!(Pin::new(&mut pump).poll(&mut cx).is_pending());
    assert_eq!(wakes.0.load(Ordering::Relaxed), before);
    sender.close();
    block_on(pump);
}

#[test]
fn observer_commands_change_next_event_and_preserve_current_observers() {
    use std::sync::Mutex;
    block_on(async {
        let (mut dispatcher, receiver, sender) = EventDispatcher::channel(4);
        let commands = sender.clone();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        dispatcher.observe(async move |event: On<TestEvent>| {
            sink.lock().unwrap().push(("old", event.0));
            yield_now().await;
            commands.remove_observer(event.observer()).unwrap();
            let sink = sink.clone();
            commands
                .observe(move |event: On<TestEvent>| {
                    sink.lock().unwrap().push(("new", event.0));
                })
                .unwrap();
        });
        let sink = seen.clone();
        dispatcher
            .observe(move |event: On<TestEvent>| sink.lock().unwrap().push(("other", event.0)));
        sender.try_emit(TestEvent(1)).unwrap();
        sender.try_emit(TestEvent(2)).unwrap();
        sender.close();
        dispatcher.pump(receiver).await;
        assert_eq!(
            *seen.lock().unwrap(),
            [("old", 1), ("other", 1), ("other", 2), ("new", 2)]
        );
    });
}

#[test]
fn command_ids_share_namespace_and_dropping_dispatcher_releases_pending_callbacks() {
    use phanerite_executor::event::ObserveError;
    let (mut dispatcher, _source, commands) = EventDispatcher::channel(4);
    let id = commands.observe(|_: On<TestEvent>| {}).unwrap();
    let direct = dispatcher.add_observer(|_: On<TestEvent>| {});
    assert_ne!(id, direct);
    assert!(dispatcher.remove_observer(id));
    let (_other, _other_source, other_handle) = EventDispatcher::channel(4);
    assert_eq!(
        other_handle.remove_observer(direct),
        Err(ObserveError::WrongDispatcher)
    );
    let capture = Arc::new(());
    let state = capture.clone();
    let nested = commands.clone();
    commands
        .observe(move |_: On<TestEvent>| {
            let _ = (&state, &nested);
        })
        .unwrap();
    assert_eq!(Arc::strong_count(&capture), 2);
    drop(dispatcher);
    assert_eq!(Arc::strong_count(&capture), 1);
    assert_eq!(
        commands.observe(|_: On<TestEvent>| {}),
        Err(ObserveError::DispatcherDropped)
    );
    assert_eq!(
        commands.remove_observer(direct),
        Err(ObserveError::DispatcherDropped)
    );
}

#[test]
fn one_handle_registers_emits_and_removes_while_pump_runs() {
    block_on(async {
        let (dispatcher, receiver, handle) = EventDispatcher::channel(4);
        let seen = Rc::new(RefCell::new(0));
        let sink = seen.clone();
        let observer_handle = handle.clone();
        zip(dispatcher.pump(receiver), async move {
            handle
                .observe(async move |event: On<TestEvent>| {
                    yield_now().await;
                    *sink.borrow_mut() += event.0;
                    observer_handle.remove_observer(event.observer()).unwrap();
                    observer_handle.try_emit(TestEvent(100)).unwrap();
                    observer_handle.close();
                })
                .unwrap();
            handle.emit(TestEvent(1)).await.unwrap();
        })
        .await;
        assert_eq!(*seen.borrow(), 1);
    });
}
