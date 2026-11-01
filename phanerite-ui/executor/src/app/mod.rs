//! Application initialization and runtime orchestration.

mod context;
mod shutdown;

pub use context::Context;

use async_executor::Executor;
use std::sync::Arc;
use taao::EventLoopBlockOn;
use tao::event_loop::EventLoopBuilder;
use wgpu::{Instance, InstanceDescriptor};

use self::shutdown::exit_guard;
use crate::event::{EventDispatcher, EventHandle, EventReceiver, On};
use crate::tao_event::TaaoEventSource;
use crate::tao_event::window::CloseRequested;
use crate::window::Window;

pub struct Application {
    event_loop: EventLoopBuilder<()>,
    instance_descriptor: InstanceDescriptor,
    events: EventDispatcher,
    event_receiver: EventReceiver,
    event_handle: EventHandle,
}

impl Default for Application {
    fn default() -> Self {
        let (events, event_receiver, event_handle) = EventDispatcher::channel(256);
        Self {
            event_loop: Default::default(),
            events,
            event_receiver,
            event_handle,
            instance_descriptor: InstanceDescriptor {
                backends: Default::default(),
                flags: Default::default(),
                memory_budget_thresholds: Default::default(),
                backend_options: Default::default(),
                display: None,
            },
        }
    }
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event_handle(&self) -> EventHandle {
        self.event_handle.clone()
    }

    pub fn run(mut self) -> Result<(), taao::BlockOnError> {
        let mut event_loop = self.event_loop.build();
        let instance = Instance::new(self.instance_descriptor);
        let executor = Executor::new();
        let (guard, exit) = exit_guard();

        event_loop.block_on_with_events(async move |event| {
            let cx = Context {
                submitter: event.submitter(),
                events: self.event_handle,
                instance,
            };
            let cx = Arc::new(cx);
            let taao_source = TaaoEventSource::new(event, &cx);

            let window_cx = cx.clone();
            let window_exit = exit.clone();
            executor
                .spawn(async move {
                    let _window = Window::builder().build(&window_cx).await.unwrap();
                    let mut guard = Some(guard);
                    cx.events
                        .observe(move |_: On<CloseRequested>| {
                            drop(guard.take());
                        })
                        .expect("event dispatcher dropped during window initialization");
                    window_exit.await;
                })
                .detach();

            executor.spawn(taao_source.run()).detach();

            executor
                .run(futures_lite::future::race(
                    self.events.pump(self.event_receiver),
                    exit,
                ))
                .await;
        })
    }
}
