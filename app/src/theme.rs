use gpui_component::Theme;

use crate::palette::{self, token};

/// Apply the selected accent without replacing the existing neutral palette.
pub fn apply(theme: &mut Theme, accent_name: &str) {
    let (primary, primary_foreground) = palette::ACCENTS
        .iter()
        .find(|(name, _, _)| *name == accent_name)
        .map(|(_, primary, foreground)| (*primary, *foreground))
        .unwrap_or((token::PRIMARY, token::PRIMARY_FOREGROUND));
    let colors = &mut theme.colors;
    colors.primary = palette::hsla_from_rgb(primary, 1.0);
    colors.primary_foreground = palette::hsla_from_rgb(primary_foreground, 1.0);
    colors.button_primary = colors.primary;
    colors.button_primary_foreground = colors.primary_foreground;
    colors.sidebar_primary = colors.primary;
    colors.sidebar_primary_foreground = colors.primary_foreground;
}
