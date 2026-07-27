use crate::download::downloader::Downloader;
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

impl DownloadGroup {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
    #[inline]
    pub fn push(&mut self, task: DownloadTask) {
        self.tasks.push(task)
    }
    #[inline]
    pub fn extend(&mut self, tasks: impl Iterator<Item = DownloadTask>) {
        self.tasks.extend(tasks)
    }
    pub fn processes(&self) -> ProcessGroup {
        ProcessGroup {
            processes: self.tasks.iter().map(|x| &x.process).cloned().collect(),
        }
    }
    #[inline]
    pub async fn exec(self, downloader: &Downloader) -> Vec<Error> {
        downloader
            .download_concurrent(self.tasks.into_iter())
            .await
            .collect()
            .await
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
            .filter(|x| x.is_started() && !x.is_finished() && !x.is_canceled())
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
