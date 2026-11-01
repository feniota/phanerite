use phanerite_executor::event::{Event, EventDispatcher, On};

#[derive(Event)]
struct ExampleEvent {
    value: u32,
}

fn main() {
    futures_lite::future::block_on(async {
        let (mut events, receiver, handle) = EventDispatcher::channel(256);
        events
            .register_event::<ExampleEvent>()
            .observe(|event: On<ExampleEvent>| {
                println!("{}", event.value);
            })
            .observe(async |event: On<ExampleEvent>| {
                futures_lite::future::yield_now().await;
                println!("async observer: {}", event.value);
            });

        // Direct triggering awaits every observer in registration order.
        events.trigger(ExampleEvent { value: 1 }).await;

        // One cloneable handle submits both observer changes and events.
        let observer_handle = handle.clone();
        let _id = handle
            .observe(move |event: On<ExampleEvent>| {
                println!("dynamic observer: {}", event.value);
                observer_handle.remove_observer(event.observer()).unwrap();
            })
            .unwrap();
        handle.emit(ExampleEvent { value: 2 }).await.unwrap();
        handle.close();
        events.pump(receiver).await;
    });
}
