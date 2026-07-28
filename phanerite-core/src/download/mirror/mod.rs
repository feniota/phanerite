pub mod bmclapi;

use crate::download::task::DownloadTask;

pub trait Mirror {
    const NAME: &str;
    const ATTRIBUTION: &str;
    const NOTICE: &str;
    fn resolve(&self, task: &mut DownloadTask);
    fn resolve_all(
        &self,
        tasks: impl Iterator<Item = DownloadTask>,
    ) -> impl Iterator<Item = DownloadTask> {
        tasks.map(|mut x| {
            self.resolve(&mut x);
            x
        })
    }
}
