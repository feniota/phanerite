use crate::download::downloader::Downloader;
use crate::download::task::filter_existed;
use crate::download::vanilla::version_info::VersionInfo;
use crate::error::{Error, Result};
use crate::instance::instance_info::VersionManifest;
use crate::storage::Storage;
use futures::AsyncWriteExt;
use tracing::error;

pub mod arguments;
pub mod instance_info;

pub struct Instance;

impl Instance {
    pub async fn create(
        version: VersionInfo,
        name: &str,
        storage: &Storage,
        downloader: &Downloader,
    ) -> Result<()> {
        // 准备实例目录
        let path = storage.versions_dir.join(name);
        if path.exists() {
            return Err(Error::other("Instance exists"));
        }
        async_fs::create_dir_all(&path).await?;

        // 创建需要的文件
        let info_file = path.join(format!("{}.json", name));
        let mut info_file = async_fs::File::create(info_file).await?;
        let index_file = storage
            .assets_indexes
            .join(format!("{}.json", version.asset_index.id));
        let mut index_file = async_fs::File::create(index_file).await?;

        // versions/{name}/{name}.json
        let manifest = VersionManifest::from_remote(version.clone()).rename(name.to_string());
        let info_json = serde_json::to_vec_pretty(&manifest)?;
        info_file.write_all(&info_json).await?;
        drop(manifest);
        drop(info_json);

        // 构造下载任务
        let game_file = path.join(format!("{}.jar", name));
        let (downloads, assets_index) = version
            .build_all_task(game_file, storage, downloader)
            .await?;
        let downloads = filter_existed(downloads);

        // assets/indexes/{id}.json
        let index_json = serde_json::to_vec_pretty(&assets_index)?;
        index_file.write_all(&index_json).await?;
        drop(index_json);

        // 执行下载
        let errors = downloader.download_concurrent(downloads).await;

        if errors.is_empty() {
            Ok(())
        } else {
            errors.iter().for_each(|e| error!("{e}"));
            Err(Error::other("download errors"))
        }
    }
}
