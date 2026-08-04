use futures::{StreamExt, TryStreamExt};
use phanerite_core::download::vanilla::version_index::VersionIndex;
use phanerite_core::error::Error;
use phanerite_core::storage::ShareStrategy::Force;
use phanerite_core::*;
use std::collections::HashSet;
use tracing::{Level, error};

#[derive(Eq, PartialEq, Hash)]
struct Resource {
    identifier: String,
    size: u64,
}

const CONCURRENCY: usize = 64;
fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    if let Err(e) = smol::block_on(async {
        let storage = storage::Storage::new(".minecraft")?.share_strategy(Force);
        let downloader = download::downloader::Downloader::builder(&storage)
            .build()
            .await?;

        let index = VersionIndex::sync(&downloader).await?;
        let versions = index
            .iter()
            .inspect(|x| println!("Fetching version: {}", x.id))
            .map(|x| x.get_manifest(&downloader));
        let versions = futures::stream::iter(versions)
            .buffer_unordered(CONCURRENCY)
            .try_collect::<Vec<_>>()
            .await?;
        let assets = versions
            .iter()
            .inspect(|x| println!("Fetching assets: {}", x.id))
            .map(|x| x.asset_index.get_list(&downloader));
        let assets = futures::stream::iter(assets)
            .buffer_unordered(CONCURRENCY)
            .try_collect::<Vec<_>>()
            .await?;

        let mut size = 0;
        let mut resources = HashSet::new();
        for v in versions {
            if let Some(c) = v.downloads.client {
                size += c.size;
            }
            if let Some(s) = v.downloads.server {
                size += s.size;
            }
            for l in v.libraries {
                if let Some(d) = l.downloads
                    && let Some(a) = d.artifact
                {
                    resources.insert(Resource {
                        identifier: a.sha1.to_string(),
                        size: a.size,
                    });
                }
            }
            size += v.asset_index.size;
        }
        for a in assets {
            for (_, o) in a.objects {
                resources.insert(Resource {
                    identifier: o.hash.to_string(),
                    size: o.size,
                });
            }
        }
        for r in resources {
            size += r.size
        }

        println!("Total: {:.2} GB", size as f64 / 1024.0 / 1024.0 / 1024.0);

        Ok::<(), Error>(())
    }) {
        error!("{}", e)
    }
}
