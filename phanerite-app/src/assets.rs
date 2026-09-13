//! Custom assets

mod icons;
mod logo;

pub use icons::PhaIcon;
pub use logo::render as phanerite_logo;
#[cfg(target_os = "linux")]
pub(crate) use logo::window_icon;

use anyhow::anyhow;
use gpui_kit::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;
use zstd::bulk::decompress as zstd_decompress;

/// Application assets, with GPUI Kit's built-in assets as a fallback.
#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "phanerite-logo.svg"]
#[include = "icons/**/*.svg"]
#[include = "fonts/**/*.zst"]
pub struct Assets;

/// Register the same bundled typefaces in the launcher and native gallery.
pub fn load_fonts(cx: &gpui_kit::App) -> Result<()> {
    let mut fonts = Vec::new();
    for path in [
        "fonts/SarasaAdwaitaUiSC-Regular.ttf.zst",
        "fonts/AdwaitaMono-Regular.ttf.zst",
    ] {
        fonts.push(
            cx.asset_source()
                .load(path)?
                .ok_or_else(|| anyhow!("bundled font is missing: {path}"))?,
        );
    }
    cx.text_system().add_fonts(fonts)
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        let data = Self::get(path)
            .map(|file| Some(file.data))
            .or_else(|| gpui_kit::assets::Assets.load(path).ok().flatten().map(Some))
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""));

        if !path.ends_with(".zst") {
            data
        } else {
            match data? {
                None => Ok(None),
                Some(compressed) => {
                    let decompressed = zstd_decompress(
                        &compressed,
                        // 64MB to avoid OOM caused by unexpected asset corruption
                        64 * 1024 * 1024,
                    )?;
                    Ok(Some(Cow::from(decompressed)))
                }
            }
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut assets = gpui_kit::assets::Assets.list(path)?;
        assets
            .extend(Self::iter().filter_map(|asset| asset.starts_with(path).then(|| asset.into())));
        Ok(assets)
    }
}
