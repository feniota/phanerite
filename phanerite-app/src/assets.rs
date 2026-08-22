//! Custom assets

mod icons;

pub use icons::PhaIcon;

use anyhow::anyhow;
use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;
use zstd::bulk::decompress as zstd_decompress;

/// Application assets, with gpui-component's built-in assets as a fallback.
#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
#[include = "fonts/**/*.zst"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        let data = Self::get(path)
            .map(|file| Some(file.data))
            .or_else(|| {
                gpui_component_assets::Assets
                    .load(path)
                    .ok()
                    .flatten()
                    .map(Some)
            })
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""));

        if !path.ends_with(".zst") {
            data
        } else {
            match data? {
                None => Ok(None),
                Some(compressed) => {
                    let decompressed = zstd_decompress(
                        &compressed,
                        // capacity: "The decompressed data should be at most capacity bytes"
                        // Likely designed to avoid attack. However the decompressed source file
                        // is controlled by ourself. So this is not needed
                        usize::MAX,
                    )?;
                    Ok(Some(Cow::from(decompressed)))
                }
            }
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut assets = gpui_component_assets::Assets.list(path)?;
        assets
            .extend(Self::iter().filter_map(|asset| asset.starts_with(path).then(|| asset.into())));
        Ok(assets)
    }
}
