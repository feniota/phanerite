use crate::error::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use std::pin::Pin;

pub struct ConcurrentTask<'a> {
    pending: Vec<Pin<Box<dyn Future<Output = Result<()>> + 'a>>>,
    max_concurrent: usize,
}

impl<'a> ConcurrentTask<'a> {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            pending: Vec::new(),
            max_concurrent,
        }
    }

    pub fn push<F>(&mut self, task: F)
    where
        F: Future<Output = Result<()>> + 'a,
    {
        self.pending.push(Box::pin(task));
    }

    pub async fn exec(mut self) -> Result<()> {
        let mut running = FuturesUnordered::new();

        loop {
            while running.len() < self.max_concurrent {
                match self.pending.pop() {
                    Some(task) => {
                        running.push(task);
                    }
                    None => break,
                }
            }

            if running.is_empty() {
                break;
            }

            match running.next().await {
                Some(result) => {
                    result?;
                }
                None => break,
            }
        }

        Ok(())
    }
}
