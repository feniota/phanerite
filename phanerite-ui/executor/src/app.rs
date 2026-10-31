use async_executor::Executor;
use std::sync::Arc;
use taao::EventLoopBlockOn;
use taao::event::Event;
use tao::event_loop::EventLoopBuilder;
use wgpu::{Instance, InstanceDescriptor};

use crate::utils::exit::exit_guard;
use crate::window::Window;

pub struct Application {
    event_loop: EventLoopBuilder<()>,
    gpu_instance: InstanceDescriptor,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            event_loop: Default::default(),
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
    pub event: Event,
    pub instance: Instance,
}

impl Application {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn run(mut self) -> Result<(), taao::BlockOnError> {
        let mut event_loop = self.event_loop.build();
        let instance = Instance::new(self.gpu_instance);
        let executor = Executor::new();
        let (guard, exit) = exit_guard();

        event_loop.block_on_with_events(async move |event| {
            let cx = Context { event, instance };
            let cx = Arc::new(cx);

            let cx = cx.clone();
            let exit_clone = exit.clone();
            executor
                .spawn(async move {
                    let _w = Window::builder().build(&cx).await.unwrap();
                    exit_clone.await;
                    // guard.exit()
                })
                .detach();

            executor.run(exit).await
        })
    }
}
