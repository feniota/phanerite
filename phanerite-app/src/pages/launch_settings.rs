//! Per-instance launch settings page.

use super::{back_button, instance_exists, missing_resource, page_shell, page_title};
use crate::{
    route::InstanceRef,
    state::{AppState, LaunchSettings},
};
use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::{ActiveTheme as _, StyledExt as _, h_flex, v_flex};
pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    _: &mut Window,
    cx: &App,
) -> gpui::AnyElement {
    if !instance_exists(reference, &app, cx) {
        return missing_resource("instance", app).into_any_element();
    }
    let instance = app
        .read(cx)
        .instances
        .read(cx)
        .find(reference)
        .unwrap()
        .clone();
    let effective = instance
        .launch_overrides
        .resolve(&LaunchSettings::default())
        .memory;
    let content = v_flex()
        .gap_4()
        .child(back_button(app))
        .child(setting_card(
            "Memory & window",
            format!("Memory allocation: {effective} GB"),
            cx,
        ))
        .child(setting_card(
            "Quick play",
            "Open a destination as soon as the game starts.".into(),
            cx,
        ))
        .child(setting_card(
            "Advanced",
            "Commands, JVM options, and native library overrides.".into(),
            cx,
        ))
        .child(
            h_flex()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Close launcher after game starts"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Unavailable until platform detachment is supported."),
                ),
        );
    page_shell(
        Some(page_title(
            "Launch settings",
            "Values inherit global defaults until you override them for this instance.",
            cx,
        )),
        content,
        cx,
    )
    .into_any_element()
}
fn setting_card(title: &'static str, description: String, cx: &App) -> impl IntoElement {
    v_flex()
        .gap_2()
        .p_4()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().accordion)
        .child(div().font_semibold().child(title))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(description),
        )
}
