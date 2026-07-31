use crate::error::Result;
use crate::instance::instance_info::VersionType;
use crate::instance::{Instance, find_manifest};
use crate::storage::Storage;
use futures::StreamExt;
use serde::Deserialize;

/// 展示用的简化的版本清单
#[derive(Debug, Clone, Deserialize)]
pub struct SimpleInfo {
    pub id: String,
    pub version_type: VersionType,
}

impl Instance {
    /// 简要列出实例列表
    pub async fn list(storage: &Storage) -> Result<Vec<SimpleInfo>> {
        let stream = async_fs::read_dir(storage.versions_dir())
            .await?
            .filter_map(async |x| x.ok())
            .filter_map(async |x| {
                find_manifest(x.file_name().to_string_lossy(), &x.path())
                    .await
                    .ok()
            })
            .filter_map(async |x| serde_json::from_slice::<SimpleInfo>(&x).ok());
        Ok(stream.collect().await)
    }
}
