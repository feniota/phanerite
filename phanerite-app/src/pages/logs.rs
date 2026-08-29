//! Log viewer page for captured instance output and log files.

use super::{back_button, instance_exists, missing_resource, page_shell, page_title};
use crate::{route::InstanceRef, state::AppState};
use gpui::{App, Entity, IntoElement as _, ParentElement as _, Styled as _, Window, div};
use gpui_component::{ActiveTheme as _, h_flex, v_flex};
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
    let subtitle = "Live output is available when the instance is running.\n\nlatest.log and debug.log retain file-backed output separately.";
    let content = v_flex()
        .gap_4()
        .child(back_button(app))
        .child(
            h_flex()
                .gap_2()
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .rounded(cx.theme().radius)
                        .bg(cx.theme().accent)
                        .child("Live output"),
                )
                .child(div().px_3().py_2().child("latest.log"))
                .child(div().px_3().py_2().child("debug.log")),
        )
        .child(
            div()
                .min_h_80()
                .p_4()
                .rounded(cx.theme().radius)
                .border_1()
                .border_color(cx.theme().border)
                .bg(crate::palette::color_alpha(
                    crate::palette::token::TERMINAL,
                    0.4,
                ))
                .font_family(crate::theme::MONO_FONT_FAMILY)
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(subtitle),
        );
    page_shell(
        Some(page_title(
            "Game logs",
            format!("Inspect output captured for {}.", instance.name),
            cx,
        )),
        content,
        cx,
    )
    .into_any_element()
}
