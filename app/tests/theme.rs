use gpui_component::Theme;
use phanerite::{palette, theme};

#[test]
fn applying_accent_preserves_neutral_theme_tokens() {
    let mut component_theme = Theme::default();
    let background = component_theme.colors.background;
    let border = component_theme.colors.border;
    theme::apply(&mut component_theme, "gold");
    assert_eq!(component_theme.colors.background, background);
    assert_eq!(component_theme.colors.border, border);
    assert_eq!(
        component_theme.colors.primary,
        palette::hsla_from_rgb(0xDCAF61, 1.0)
    );
}
