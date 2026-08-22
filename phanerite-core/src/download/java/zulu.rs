use crate::download::Downloader;
use crate::download::extract::ExtractTask;
use crate::download::java::JavaDownload;
use crate::download::task::DownloadTask;
use crate::error::{Error, Result};
use crate::runtime::RuntimePath;
use crate::storage::Storage;
use serde::Deserialize;
use std::sync::LazyLock;
use url::Url;

pub struct Zulu;

static ZULU_PACKAGE_META: LazyLock<Url> = LazyLock::new(|| {
    "https://api.azul.com/metadata/v1/zulu/packages/"
        .parse()
        .unwrap()
});

impl JavaDownload for Zulu {
    async fn get_major<'cx>(
        major: u32,
        downloader: &impl Downloader,
        storage: &'cx Storage,
    ) -> Result<DownloadTask<'cx>> {
        let mut url = ZULU_PACKAGE_META.clone();

        url.query_pairs_mut()
            .append_pair("java_version", &major.to_string())
            .append_pair("os", std::env::consts::OS)
            .append_pair(
                "arch",
                match std::env::consts::ARCH {
                    "x86_64" => "x64",
                    "aarch64" => "arm",
                    arch => arch,
                },
            )
            .append_pair("java_package_type", "jre")
            .append_pair("javafx_bundled", "false")
            .append_pair("release_status", "ga")
            .append_pair("availability_types", "CA")
            .append_pair("certifications", "tck")
            .append_pair("latest", "true")
            .finish();

        let body = downloader.fetch(url, None).await?;
        let json: Vec<ZuluPackage> = serde_json::from_slice(&body)?;

        let choice = json
            .into_iter()
            .filter(|p| {
                p.name.ends_with(".zip")
                    || p.name.ends_with(".tar")
                    || p.name.ends_with(".tar.bz2")
                    || p.name.ends_with(".tar.gz")
                    || p.name.ends_with(".tar.xz")
                    || p.name.ends_with(".tar.zst")
            })
            .max_by_key(|p| (p.java_version.clone(), p.distro_version.clone()));

        let Some(choice) = choice else {
            return Err(Error::other("No available runtime"));
        };

        let extract = ExtractTask::builder()
            .target(
                storage
                    .runtime_dir()
                    .join(RuntimePath::new("jre", major as usize, "zulu").to_string()),
            )
            .flatten()
            .build();

        let download = DownloadTask::builder()
            .url(choice.download_url)
            .extract_to(extract, storage)
            .file_name(choice.name)
            .build();

        Ok(download)
    }
}

#[derive(Deserialize)]
struct ZuluPackage {
    // /// CA / SA
    // availability_type: String,
    // Azul Zulu 构建版本，例如 [21, 52, 15, 0]
    /// Azul Zulu build version, e.g. [21, 52, 15, 0]
    distro_version: Vec<u32>,
    // 下载地址
    /// Download URL
    download_url: Url,
    // Java 版本，例如 [21, 0, 12]
    /// Java version, e.g. [21, 0, 12]
    java_version: Vec<u32>,
    // // 是否最新版本
    // /// Whether this is the latest version
    // latest: bool,
    // 文件名
    /// File name
    name: String,
    // /// OpenJDK build number
    // openjdk_build_number: u32,
    // /// Azul package UUID
    // package_uuid: Uuid,
    // /// 产品名，一般 zulu
    // product: String,
}
