use async_executor::Executor;
use taao::EventLoopBlockOn;
use tao::event_loop::EventLoopBuilder;

use crate::utils::exit::exit_guard;

#[derive(Default)]
pub struct Application<'ex> {
    builder: EventLoopBuilder<()>,
    executor: Executor<'ex>,
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
                    let _ = guard;
                    let _ = event;
                })
                .detach();

            self.executor.run(exit).await
        })
    }
}
