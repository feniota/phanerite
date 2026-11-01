//! Application initialization and runtime orchestration.

mod context;
mod shutdown;

pub use context::Context;
pub use shutdown::Shutdown;
use std::rc::Rc;

use self::shutdown::exit_guard;
use crate::app::shutdown::Exit;
use crate::event::{EventDispatcher, EventHandle, EventReceiver, On};
use crate::tao_event::TaaoEventSource;
use async_executor::LocalExecutor;
use taao::EventLoopBlockOn;
use tao::event_loop::EventLoopBuilder;
use wgpu::{Instance, InstanceDescriptor};

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

    pub fn configure_gpu(mut self, config: InstanceDescriptor) -> Self {
        self.instance_descriptor = config;
        self
    }

    pub fn event_handle(&self) -> EventHandle {
        self.event_handle.clone()
    }

    pub fn run<F>(mut self, init: F) -> Result<(), taao::BlockOnError>
    where
        F: AsyncFnOnce(Rc<Context>, Exit),
    {
        let mut event_loop = self.event_loop.build();
        let instance = Instance::new(self.instance_descriptor);
        let executor = LocalExecutor::new();
        let (guard, exit) = exit_guard();

        event_loop.block_on_with_events(async move |event| {
            let cx = Context {
                submitter: event.submitter(),
                events: self.event_handle,
                instance,
            };
            let cx = Rc::new(cx);

            let mut guard = Some(guard);
            cx.events
                .observe(move |_: On<Shutdown>| {
                    drop(guard.take());
                })
                .expect("event dispatcher dropped during shutdown registration");

            let init = init(cx.clone(), exit.clone());
            executor.spawn(init).detach();

            let taao_source = TaaoEventSource::new(event, &cx);
            executor.spawn(taao_source.run()).detach();

            executor
                .run(exit.race(self.events.pump(self.event_receiver)))
                .await;
        })
    }
}
