//! Custom assets

use anyhow::anyhow;
use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

/// Application assets, with gpui-component's built-in assets as a fallback.
#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|file| Some(file.data))
            .or_else(|| {
                gpui_component_assets::Assets
                    .load(path)
                    .ok()
                    .flatten()
                    .map(Some)
            })
            .ok_or_else(|| anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut assets = gpui_component_assets::Assets.list(path)?;
        assets
            .extend(Self::iter().filter_map(|asset| asset.starts_with(path).then(|| asset.into())));
        Ok(assets)
    }
}

/// Icon assets, most of which are from Lucide Icons (https://lucide.dev/).
pub enum PhaIcon {
    Layers,
    PlayFilled,
}

impl Into<gpui_component::Icon> for PhaIcon {
    fn into(self) -> gpui_component::Icon {
        match self {
            Self::Layers => gpui_component::Icon::default().path("icons/layers.svg"),
            Self::PlayFilled => gpui_component::Icon::default().path("icons/play-filled.svg"),
        }
    }
}
