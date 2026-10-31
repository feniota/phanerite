use async_executor::Executor;
use taao::EventLoopBlockOn;
use tao::event_loop::EventLoopBuilder;
use wgpu::Instance;

use crate::utils::exit::exit_guard;

#[derive(Default)]
pub struct Application<'ex> {
    builder: EventLoopBuilder<()>,
    executor: Executor<'ex>,
    instance: Instance,
}

impl Application<'_> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn run(&mut self) -> Result<(), taao::BlockOnError> {
        let mut event_loop = self.builder.build();
        let (guard, exit) = exit_guard();

        event_loop.block_on_with_events(async |event| {
            self.executor
                .spawn(async {
                    let _ = event;
                    guard.exit()
                })
                .detach();

            self.executor.run(exit).await
        })
    }
}
