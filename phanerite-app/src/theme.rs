//! Theme application, typography scale, and shared visual constants.

use gpui::{px, App, Hsla, Window};
use gpui_component::{Theme, ThemeMode};

use crate::palette::{self, token};

pub const FONT_FAMILY: &str = "Sarasa Adwaita UI SC";
pub const MONO_FONT_FAMILY: &str = "Adwaita Mono";

/// Type scale, in pixels.
///
/// The prototype's ramp is 11 / 12 / 14 / 16 / 18 px, tuned for a browser at
/// 1200 px. A native window is read from further away, so every step is nudged
/// up while the relationships between the steps are preserved.
pub mod text {
    /// Meta, counts and timestamps — the floor, nothing smaller.
    pub const MICRO: f32 = 12.0;
    /// Dense body: list rows, descriptions, most of the phanerite-app chrome.
    pub const XS: f32 = 13.0;
    /// Emphasized body and form labels.
    pub const SM: f32 = 15.0;
    pub const BASE: f32 = 16.0;
    /// View titles.
    pub const LG: f32 = 20.0;
}

/// Border radius of the general elements, matching `--radius`.
pub const RADIUS: f32 = 6.0;

/// Applies the complete Phanerite palette to a component theme. Dark only,
/// like the prototype; the light configuration receives the same values so a
/// system appearance change cannot leave the window half-styled.
pub fn apply_palette(theme: &mut Theme) {
    let colors = &mut theme.colors;

    let background = palette::color(token::BACKGROUND);
    let foreground = palette::color(token::FOREGROUND);
    let card = palette::color(token::CARD);
    let popover = palette::color(token::POPOVER);
    let primary = palette::color(token::PRIMARY);
    let primary_foreground = palette::color(token::PRIMARY_FOREGROUND);
    let secondary = palette::color(token::SECONDARY);
    let muted = palette::color(token::MUTED);
    let muted_foreground = palette::color(token::MUTED_FOREGROUND);
    let accent = palette::color(token::ACCENT);
    let accent_foreground = palette::color(token::ACCENT_FOREGROUND);
    let destructive = palette::color(token::DESTRUCTIVE);
    let sidebar = palette::color(token::SIDEBAR);
    let border = palette::color_alpha(0xFFFFFF, token::BORDER_ALPHA as f32 / 255.0);
    let input = palette::color_alpha(0xFFFFFF, token::INPUT_ALPHA as f32 / 255.0);
    let scrollbar_thumb = palette::color_alpha(0xFFFFFF, token::SCROLLBAR_ALPHA as f32 / 255.0);

    colors.background = background;
    colors.foreground = foreground;
    colors.border = border;
    colors.input = input;
    colors.ring = primary;
    colors.selection = palette::color_alpha(token::PRIMARY, 0.35);
    colors.caret = foreground;
    colors.drag_border = primary;
    colors.drop_target = palette::color_alpha(token::PRIMARY, 0.2);
    colors.overlay = palette::color_alpha(0x000000, 0.6);
    colors.window_border = border;

    colors.popover = popover;
    colors.popover_foreground = foreground;
    colors.accordion = card;
    colors.group_box = card;
    colors.group_box_foreground = foreground;

    colors.primary = primary;
    colors.primary_foreground = primary_foreground;
    colors.primary_hover = lighten(primary, 0.04);
    colors.primary_active = darken(primary, 0.04);

    colors.secondary = secondary;
    colors.secondary_foreground = foreground;
    colors.secondary_hover = accent;
    colors.secondary_active = muted;

    colors.muted = muted;
    colors.muted_foreground = muted_foreground;
    colors.accent = accent;
    colors.accent_foreground = accent_foreground;

    colors.danger = destructive;
    colors.danger_foreground = palette::color(0xFFFFFF);
    colors.danger_hover = lighten(destructive, 0.04);
    colors.danger_active = darken(destructive, 0.04);

    colors.success = primary;
    colors.success_foreground = primary_foreground;
    colors.success_hover = lighten(primary, 0.04);
    colors.success_active = darken(primary, 0.04);

    colors.warning = palette::color(token::WARNING);
    colors.warning_foreground = palette::color(token::BACKGROUND);
    colors.warning_hover = lighten(palette::color(token::WARNING), 0.04);
    colors.warning_active = darken(palette::color(token::WARNING), 0.04);

    colors.info = palette::color(token::CHART_3);
    colors.info_foreground = foreground;
    colors.info_hover = lighten(palette::color(token::CHART_3), 0.04);
    colors.info_active = darken(palette::color(token::CHART_3), 0.04);

    // Buttons: the default variant is the prototype's `outline` button.
    colors.button = card;
    colors.button_foreground = foreground;
    colors.button_hover = accent;
    colors.button_active = muted;
    colors.button_primary = primary;
    colors.button_primary_foreground = primary_foreground;
    colors.button_primary_hover = lighten(primary, 0.04);
    colors.button_primary_active = darken(primary, 0.04);
    colors.button_secondary = secondary;
    colors.button_secondary_foreground = foreground;
    colors.button_secondary_hover = accent;
    colors.button_secondary_active = muted;
    colors.button_danger = destructive;
    colors.button_danger_foreground = palette::color(0xFFFFFF);
    colors.button_danger_hover = lighten(destructive, 0.04);
    colors.button_danger_active = darken(destructive, 0.04);
    colors.button_success = primary;
    colors.button_success_foreground = primary_foreground;
    colors.button_success_hover = lighten(primary, 0.04);
    colors.button_success_active = darken(primary, 0.04);
    colors.button_warning = palette::color(token::WARNING);
    colors.button_warning_foreground = palette::color(token::BACKGROUND);
    colors.button_warning_hover = lighten(palette::color(token::WARNING), 0.04);
    colors.button_warning_active = darken(palette::color(token::WARNING), 0.04);
    colors.button_info = palette::color(token::CHART_3);
    colors.button_info_foreground = foreground;
    colors.button_info_hover = lighten(palette::color(token::CHART_3), 0.04);
    colors.button_info_active = darken(palette::color(token::CHART_3), 0.04);

    colors.link = primary;
    colors.link_hover = lighten(primary, 0.06);
    colors.link_active = darken(primary, 0.06);

    colors.list = background;
    colors.list_even = background;
    colors.list_head = card;
    colors.list_hover = accent;
    colors.list_active = accent;
    colors.list_active_border = primary;

    colors.table = background;
    colors.table_even = background;
    colors.table_head = card;
    colors.table_head_foreground = muted_foreground;
    colors.table_foot = card;
    colors.table_foot_foreground = muted_foreground;
    colors.table_hover = accent;
    colors.table_active = accent;
    colors.table_active_border = primary;
    colors.table_row_border = border;

    colors.tab = background;
    colors.tab_bar = card;
    colors.tab_bar_segmented = secondary;
    colors.tab_active = accent;
    colors.tab_active_foreground = accent_foreground;
    colors.tab_foreground = muted_foreground;

    colors.title_bar = card;
    colors.title_bar_border = border;
    colors.status_bar = background;
    colors.status_bar_border = border;
    colors.tiles = background;

    colors.sidebar = sidebar;
    colors.sidebar_foreground = muted_foreground;
    colors.sidebar_accent = accent;
    colors.sidebar_accent_foreground = accent_foreground;
    colors.sidebar_border = border;
    colors.sidebar_primary = primary;
    colors.sidebar_primary_foreground = primary_foreground;

    colors.skeleton = muted;
    colors.slider_bar = primary;
    colors.slider_thumb = foreground;
    colors.progress_bar = primary;
    colors.switch = secondary;
    colors.switch_thumb = foreground;
    colors.scrollbar = palette::color_alpha(0x000000, 0.0);
    colors.scrollbar_thumb = scrollbar_thumb;
    colors.scrollbar_thumb_hover = palette::color_alpha(0xFFFFFF, 0.24);

    colors.description_list_label = card;
    colors.description_list_label_foreground = muted_foreground;

    colors.chart_1 = primary;
    colors.chart_2 = palette::color(token::CHART_2);
    colors.chart_3 = palette::color(token::CHART_3);
    colors.chart_4 = palette::color(token::CHART_4);
    colors.chart_5 = palette::color(token::CHART_5);

    theme.radius = px(RADIUS);
    theme.radius_lg = px(RADIUS + 2.0);
    theme.font_size = px(text::BASE);
    theme.shadow = false;
}

/// Apply the selected accent without replacing the existing neutral palette.
pub fn apply(theme: &mut Theme, accent_name: &str) {
    let (primary, primary_foreground, chart_3, chart_5) = palette::ACCENTS
        .iter()
        .find(|(name, ..)| *name == accent_name)
        .map(|(_, primary, foreground, chart_3, chart_5)| {
            (*primary, *foreground, *chart_3, *chart_5)
        })
        .unwrap_or((
            token::PRIMARY,
            token::PRIMARY_FOREGROUND,
            token::CHART_3,
            token::CHART_5,
        ));
    let colors = &mut theme.colors;
    colors.primary = palette::hsla_from_rgb(primary, 1.0);
    colors.primary_foreground = palette::hsla_from_rgb(primary_foreground, 1.0);
    colors.primary_hover = lighten(colors.primary, 0.04);
    colors.primary_active = darken(colors.primary, 0.04);
    colors.ring = colors.primary;
    colors.drag_border = colors.primary;
    colors.selection = with_alpha(colors.primary, 0.35);
    colors.button_primary = colors.primary;
    colors.button_primary_foreground = colors.primary_foreground;
    colors.button_primary_hover = colors.primary_hover;
    colors.button_primary_active = colors.primary_active;
    colors.success = colors.primary;
    colors.success_foreground = colors.primary_foreground;
    colors.button_success = colors.primary;
    colors.button_success_foreground = colors.primary_foreground;
    colors.sidebar_primary = colors.primary;
    colors.sidebar_primary_foreground = colors.primary_foreground;
    colors.slider_bar = colors.primary;
    colors.progress_bar = colors.primary;
    colors.link = colors.primary;
    colors.list_active_border = colors.primary;
    colors.table_active_border = colors.primary;
    colors.chart_1 = colors.primary;
    colors.chart_3 = palette::hsla_from_rgb(chart_3, 1.0);
    colors.chart_5 = palette::hsla_from_rgb(chart_5, 1.0);
}

/// Installs the palette plus the chosen accent as the global theme, and keeps
/// the Base token projection in sync so unstyled primitives match.
pub fn install(accent_name: &str, window: Option<&mut Window>, cx: &mut App) {
    // Phanerite is dark-only. Pinning the mode means a system appearance change
    // cannot swap in the component library's light defaults underneath us.
    Theme::change(ThemeMode::Dark, None, cx);
    let theme = Theme::global_mut(cx);
    theme.font_family = FONT_FAMILY.into();
    theme.mono_font_family = MONO_FONT_FAMILY.into();
    apply_palette(theme);
    apply(theme, accent_name);
    sync_tokens(cx);
    if let Some(window) = window {
        window.refresh();
    }
}

/// Re-derives the component and Base token snapshots from `theme.colors`.
/// Mutating the color fields alone leaves both projections stale.
pub fn sync_tokens(cx: &mut App) {
    let theme = Theme::global_mut(cx);
    theme.tokens = theme.colors.into();
    let tokens = theme.semantic_tokens();
    gpui_base::Theme::global_mut(cx).tokens = tokens;
}

fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

fn lighten(color: Hsla, amount: f32) -> Hsla {
    Hsla {
        l: (color.l + amount).min(1.0),
        ..color
    }
}

fn darken(color: Hsla, amount: f32) -> Hsla {
    Hsla {
        l: (color.l - amount).max(0.0),
        ..color
    }
}
