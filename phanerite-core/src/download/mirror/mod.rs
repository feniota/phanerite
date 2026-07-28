pub mod bmclapi;
pub mod granodiorite;

use crate::download::task::DownloadTask;
use url::Url;

pub trait Mirror {
    const NAME: &str;
    const ATTRIBUTION: &str;
    const NOTICE: &str;
    fn resolve(&self, url: &mut Url);
    fn resolve_task(&self, task: &mut DownloadTask) {
        self.resolve(&mut task.url)
    }
    fn resolve_all(
        &self,
        tasks: impl Iterator<Item = DownloadTask>,
    ) -> impl Iterator<Item = DownloadTask> {
        tasks.map(|mut x| {
            self.resolve_task(&mut x);
            x
        })
    }
}
