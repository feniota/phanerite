use async_executor::Executor;
use std::sync::Arc;
use taao::EventLoopBlockOn;
use taao::event::EventSubmitter;
use tao::event_loop::EventLoopBuilder;
use wgpu::{Instance, InstanceDescriptor};

use crate::event::{EventDispatcher, EventHandle, EventSource};
use crate::utils::exit::exit_guard;

pub struct Application {
    event_loop: EventLoopBuilder<()>,
    gpu_instance: InstanceDescriptor,
    events: EventDispatcher,
    event_source: EventSource,
    event_handle: EventHandle,
}

impl Default for Application {
    fn default() -> Self {
        let (events, event_source, event_handle) = EventDispatcher::channel(256);
        Self {
            event_loop: Default::default(),
            events,
            event_source,
            event_handle,
            gpu_instance: InstanceDescriptor {
                backends: Default::default(),
                flags: Default::default(),
                memory_budget_thresholds: Default::default(),
                backend_options: Default::default(),
                display: None,
            },
        }
    }
}

pub struct Context {
    pub submitter: EventSubmitter,
    pub events: EventHandle,
    pub instance: Instance,
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event_handle(&self) -> EventHandle {
        self.event_handle.clone()
    }

    /// Registers application observers before entering the event loop.
    pub fn event_dispatcher_mut(&mut self) -> &mut EventDispatcher {
        &mut self.events
    }

    pub fn run(mut self) -> Result<(), taao::BlockOnError> {
        let mut event_loop = self.event_loop.build();
        let instance = Instance::new(self.gpu_instance);
        let executor = Executor::new();
        let (guard, exit) = exit_guard();

        event_loop.block_on_with_events(async move |event| {
            let cx = Context {
                submitter: event.submitter(),
                events: self.event_handle,
                instance,
            };
            let cx = Arc::new(cx);

            executor
                .spawn(async move {
                    let _ = cx;
                })
                .detach();

            let source = self.event_source.with_taao(event);
            executor
                .run(futures_lite::future::race(self.events.pump(source), exit))
                .await;
        })
    }
}
