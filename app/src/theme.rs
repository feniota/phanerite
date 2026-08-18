use gpui_component::Theme;

use crate::palette::{self, token};

/// Apply the palette to an existing component theme, changing only the
/// application colors and the selected primary accent.
pub fn apply(theme: &mut Theme, accent_name: &str) {
    let (primary, primary_foreground) = palette::ACCENTS
        .iter()
        .find(|(name, _, _)| *name == accent_name)
        .map(|(_, primary, foreground)| (*primary, *foreground))
        .unwrap_or((token::PRIMARY, token::PRIMARY_FOREGROUND));
    let colors = &mut theme.colors;
    colors.background = palette::hsla_from_rgb(token::BACKGROUND, 1.0);
    colors.foreground = palette::hsla_from_rgb(token::FOREGROUND, 1.0);
    colors.accordion = palette::hsla_from_rgb(token::CARD, 1.0);
    colors.popover = palette::hsla_from_rgb(token::POPOVER, 1.0);
    colors.primary = palette::hsla_from_rgb(primary, 1.0);
    colors.primary_foreground = palette::hsla_from_rgb(primary_foreground, 1.0);
    colors.secondary = palette::hsla_from_rgb(token::SECONDARY, 1.0);
    colors.muted = palette::hsla_from_rgb(token::MUTED, 1.0);
    colors.muted_foreground = palette::hsla_from_rgb(token::MUTED_FOREGROUND, 1.0);
    colors.accent = palette::hsla_from_rgb(token::ACCENT, 1.0);
    colors.accent_foreground = palette::hsla_from_rgb(token::ACCENT_FOREGROUND, 1.0);
    colors.danger = palette::hsla_from_rgb(token::DESTRUCTIVE, 1.0);
    colors.sidebar = palette::hsla_from_rgb(token::SIDEBAR, 1.0);
    colors.chart_3 = palette::hsla_from_rgb(token::CHART_3, 1.0);
    colors.chart_5 = palette::hsla_from_rgb(token::CHART_5, 1.0);
    colors.border = palette::hsla_from_rgb(0xFFFFFF, 0x17 as f32 / 255.0);
    colors.input = palette::hsla_from_rgb(0xFFFFFF, 0x1F as f32 / 255.0);
}
