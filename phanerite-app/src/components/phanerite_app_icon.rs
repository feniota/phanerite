//! Compact Phanerite application mark used in the title bar.

use gpui::{IntoElement, ParentElement as _, Styled as _, div, px};
use gpui_component::v_flex;

use crate::palette;

/// The compact Phanerite mark used in the native title bar.
pub fn render() -> impl IntoElement {
    v_flex()
        .w(px(20.))
        .h(px(20.))
        .items_center()
        .justify_center()
        .rounded(px(5.))
        .bg(palette::color(palette::token::PRIMARY))
        .child(
            div()
                .w(px(10.))
                .h(px(10.))
                .rounded(px(2.))
                .bg(palette::color(palette::token::PRIMARY_FOREGROUND)),
        )
}
