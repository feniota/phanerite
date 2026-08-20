//! Compact Phanerite application mark used in the title bar.

use gpui::{IntoElement, ParentElement as _, Styled as _, div};
use gpui_component::v_flex;

use crate::palette;

/// The compact Phanerite mark used in the native title bar.
pub fn render() -> impl IntoElement {
    v_flex()
        .w(gpui::px(20.))
        .h(gpui::px(20.))
        .items_center()
        .justify_center()
        .rounded(gpui::px(5.))
        .bg(palette::color(palette::token::PRIMARY))
        .child(
            div()
                .w(gpui::px(10.))
                .h(gpui::px(10.))
                .rounded(gpui::px(2.))
                .bg(palette::color(palette::token::PRIMARY_FOREGROUND)),
        )
}
