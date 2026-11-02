use async_channel::Sender;
use async_executor::{Executor, Task};
use std::num::NonZeroU8;
use std::sync::Arc;

pub(crate) struct BackgroundRunner {
    background: Arc<Executor<'static>>,
    unblock: Arc<Executor<'static>>,
    shutdown: Sender<()>,
}

const STACK_SIZE: usize = 2 * 1024 * 1024;

impl BackgroundRunner {
    pub(crate) fn new(async_ex: NonZeroU8, blocking: NonZeroU8) -> Self {
        let background = Arc::new(Executor::new());
        let unblock = Arc::new(Executor::new());
        let (shutdown, exit) = async_channel::bounded(0);

        let exec = |ex: Arc<Executor<'static>>, name: String| {
            let exit = exit.clone();
            std::thread::Builder::new()
                .name(name)
                .stack_size(STACK_SIZE)
                .spawn(move || {
                    futures_lite::future::block_on(async move {
                        ex.run(async move {
                            let _ = exit.recv().await;
                        })
                        .await;
                    })
                })
                .expect("Failed to create worker thread.")
        };

        for i in 0..async_ex.get() {
            exec(background.clone(), format!("phaui-bg-{i}"));
        }

        for i in 0..blocking.get() {
            exec(unblock.clone(), format!("phaui-blk-{i}"));
        }

        Self {
            background,
            unblock,
            shutdown,
        }
    }
    pub(crate) fn shutdown(&self) {
        self.shutdown.close();
    }
    pub(crate) fn spawn<F, R>(&self, future: F) -> Task<R>
    where
        F: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        self.background.spawn(future)
    }
    pub(crate) fn unblock<F, R>(&self, blocking: F) -> Task<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.unblock.spawn(async move { blocking() })
    }
}
