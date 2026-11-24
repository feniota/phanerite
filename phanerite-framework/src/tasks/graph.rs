use crate::tasks::graph::executor::{AsyncExecutor, CurrentThread, Scheduler};
use futures::stream::{FuturesUnordered, StreamExt};
use slotmap::SlotMap;
use std::collections::HashMap;

slotmap::new_key_type! {
    pub struct NodeId;
}

pub(crate) enum TaskType {
    AccountLogin,
    AccountRefresh,

    InstanceVerify,
    InstanceInstall,
    InstanceLaunch,
}

pub enum TaskOutput {
    AccountLogin,
    AccountRefresh,

    InstanceVerify,
    InstanceInstall,
    InstanceLaunch,
}

pub struct TaskError;

pub type TaskResult = Result<TaskOutput, TaskError>;

impl TaskType {
    async fn exec(self) -> TaskResult {
        match self {
            Self::AccountLogin => {
                // TODO
                Ok(TaskOutput::AccountLogin)
            }

            Self::AccountRefresh => {
                // TODO
                Ok(TaskOutput::AccountRefresh)
            }

            Self::InstanceVerify => {
                // TODO
                Ok(TaskOutput::InstanceVerify)
            }

            Self::InstanceInstall => {
                // TODO
                Ok(TaskOutput::InstanceInstall)
            }

            Self::InstanceLaunch => {
                // TODO
                Ok(TaskOutput::InstanceLaunch)
            }
        }
    }
}

pub struct TaskNode {
    content: TaskType,
    dependents: Vec<NodeId>,
    remaining: usize,
}

#[derive(Default)]
pub struct TasksGraph {
    tasks: SlotMap<NodeId, TaskNode>,
}

impl TasksGraph {
    pub fn new() -> Self {
        Self::default()
    }

    async fn exec<S: Scheduler>(
        self,
        scheduler: S,
    ) -> Result<HashMap<NodeId, TaskOutput>, TaskError> {
        let mut tasks = self.tasks;
        let mut ready = tasks
            .iter()
            .filter_map(|(id, node)| (node.remaining == 0).then_some(id))
            .collect::<Vec<_>>();
        let mut running = FuturesUnordered::new();
        let mut outputs = HashMap::new();

        loop {
            for id in ready.drain(..) {
                let TaskNode {
                    content,
                    dependents,
                    ..
                } = tasks.remove(id).unwrap();

                let scheduler = &scheduler;
                running.push(async move {
                    let result = scheduler.run(content.exec()).await;
                    (id, dependents, result)
                });
            }

            let Some((id, dependents, result)) = running.next().await else {
                break;
            };

            outputs.insert(id, result?);

            for dependent in dependents {
                let node = tasks
                    .get_mut(dependent)
                    .expect("dependent node should exist");
                node.remaining -= 1;
                if node.remaining == 0 {
                    ready.push(dependent);
                }
            }
        }

        Ok(outputs)
    }

    pub async fn run(self) -> Result<HashMap<NodeId, TaskOutput>, TaskError> {
        self.exec(CurrentThread).await
    }

    pub fn schedule<'a>(
        self,
        ex: &'a async_executor::Executor<'a>,
    ) -> async_executor::Task<Result<HashMap<NodeId, TaskOutput>, TaskError>> {
        ex.spawn(self.exec(AsyncExecutor::new(ex)))
    }
}

mod executor {
    use async_executor::Executor;

    pub(super) trait Scheduler {
        async fn run<F, R>(&self, fut: F) -> R
        where
            F: Future<Output = R> + Send + 'static,
            R: Send + 'static;
    }

    pub(super) struct CurrentThread;
    impl Scheduler for CurrentThread {
        async fn run<F, R>(&self, fut: F) -> R
        where
            F: Future<Output = R> + Send + 'static,
            R: Send + 'static,
        {
            fut.await
        }
    }
    pub(super) struct AsyncExecutor<'a, 'ex> {
        ex: &'a Executor<'ex>,
    }
    impl Scheduler for AsyncExecutor<'_, '_> {
        async fn run<F, R>(&self, fut: F) -> R
        where
            F: Future<Output = R> + Send + 'static,
            R: Send + 'static,
        {
            self.ex.spawn(fut).await
        }
    }
    impl<'a, 'ex> AsyncExecutor<'a, 'ex> {
        pub(super) fn new(ex: &'a Executor<'ex>) -> Self {
            Self { ex }
        }
    }
}
