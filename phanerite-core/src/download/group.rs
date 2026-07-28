use crate::download::downloader::Downloader;
use crate::download::mirror::Mirror;
use crate::download::task::{DownloadProcess, DownloadTask};
use crate::error::Error;
use futures::StreamExt;

pub struct DownloadGroup {
    tasks: Vec<DownloadTask>,
}

pub struct ProcessGroup {
    processes: Vec<DownloadProcess>,
}

impl Default for DownloadGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Extend<DownloadTask> for DownloadGroup {
    fn extend<T: IntoIterator<Item = DownloadTask>>(&mut self, iter: T) {
        self.tasks.extend(iter)
    }
}

impl DownloadGroup {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
    pub fn push(&mut self, task: DownloadTask) {
        self.tasks.push(task)
    }
    pub fn processes(&self) -> ProcessGroup {
        ProcessGroup {
            processes: self.tasks.iter().map(|x| &x.process).cloned().collect(),
        }
    }
    pub async fn exec(self, downloader: &Downloader) -> Vec<Error> {
        downloader
            .download_concurrent(self.tasks.into_iter())
            .await
            .collect()
            .await
    }
    pub async fn exec_with_mirror(
        self,
        downloader: &Downloader,
        mirror: impl Mirror,
    ) -> Vec<Error> {
        let tasks = mirror.resolve_all(self.tasks.into_iter());
        downloader.download_concurrent(tasks).await.collect().await
    }
}

impl ProcessGroup {
    pub fn total(&self) -> u64 {
        self.processes.iter().filter_map(|x| x.total()).sum()
    }
    pub fn current(&self) -> u64 {
        self.processes.iter().map(|x| x.current()).sum()
    }
    pub fn downloading(&self) -> usize {
        self.processes
            .iter()
            .filter(|x| x.is_started() && !x.is_finished() && !x.is_canceled() && !x.is_failed())
            .count()
    }
    pub fn is_finished(&self) -> bool {
        self.processes.iter().all(|x| x.is_finished())
    }
    pub async fn speed_by_timer(&self, timer: impl Future) -> u64 {
        let start = self.current();
        timer.await;
        self.current() - start
    }
    pub fn iter(&self) -> impl Iterator<Item = &DownloadProcess> {
        self.processes.iter()
    }
}
