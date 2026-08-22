//! Crash report page with local findings and shareable diagnostics.

use super::{back_button, crash_exists, missing_resource, page_shell, page_title};
use crate::{route::CrashRef, state::AppState};
use gpui::{App, Entity, IntoElement as _, ParentElement as _, Styled as _, Window, div};
use gpui_component::{
    ActiveTheme as _, IconName, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
pub fn render(
    reference: &CrashRef,
    app: Entity<AppState>,
    _: &mut Window,
    cx: &App,
) -> gpui::AnyElement {
    if !crash_exists(reference, &app, cx) {
        return missing_resource("crash report", app).into_any_element();
    }
    let report = app
        .read(cx)
        .crashes
        .read(cx)
        .find(reference)
        .unwrap()
        .clone();
    let findings = if report.findings.is_empty() {
        v_flex()
            .gap_2()
            .p_4()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().accordion)
            .child(
                div()
                    .font_semibold()
                    .child("No known crash pattern matched"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Inspect the report and environment below, then retry the instance."),
            )
            .into_any_element()
    } else {
        v_flex()
            .gap_2()
            .children(report.findings.iter().map(|finding| {
                v_flex()
                    .gap_1()
                    .p_3()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().danger)
                    .bg(cx.theme().accordion)
                    .child(
                        div()
                            .font_medium()
                            .text_color(cx.theme().danger)
                            .child(finding.title.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(finding.explanation.clone()),
                    )
            }))
            .into_any_element()
    };
    let content = v_flex()
        .gap_4()
        .child(back_button(app))
        .child(
            div()
                .text_lg()
                .font_semibold()
                .child(format!("{} crashed", report.instance_id)),
        )
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!("Exit code {} · {}", report.exit_code, report.when)),
        )
        .child(div().font_semibold().child("Known crash patterns"))
        .child(findings)
        .child(div().font_semibold().child("Crash report"))
        .child(
            div()
                .min_h_64()
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
                .child(report.source_text()),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("crash-copy")
                        .icon(IconName::Copy)
                        .label("Copy redacted report"),
                )
                .child(Button::new("crash-retry").primary().label("Retry launch")),
        );
    page_shell(
        Some(page_title(
            "Crash report",
            "Local diagnostic findings and redacted output for this failed launch.",
            cx,
        )),
        content,
        cx,
    )
    .into_any_element()
}
