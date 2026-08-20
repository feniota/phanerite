//! Initial setup page for configuring the launcher environment.

use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::button::ButtonVariants as _;
use gpui_component::{ActiveTheme as _, IconName, StyledExt, button::Button, v_flex};

use crate::state::AppState;

pub fn render(app: Entity<AppState>, _: &mut Window, cx: &App) -> impl IntoElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_4()
        .child(
            div()
                .text_lg()
                .font_semibold()
                .child("Welcome to Phanerite"),
        )
        .child(
            div()
                .w_96()
                .text_center()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Choose a Minecraft directory to create your first storage location."),
        )
        .child(
            Button::new("setup-storage")
                .primary()
                .icon(IconName::FolderOpen)
                .label("Choose game directory")
                .on_click(move |_, _, cx| {
                    app.update(cx, |state, cx| state.replace(crate::route::Route::Play, cx));
                }),
        )
}
