//! Application settings page for preferences and appearance.

use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::{
    ActiveTheme as _, IconName, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

use crate::state::AppState;

use super::{page_shell, page_title};

pub fn render(app: Entity<AppState>, _: &mut Window, cx: &App) -> impl IntoElement {
    let app_state = app.read(cx);
    let settings = app_state.settings.read(cx);
    let runtimes = settings.runtimes().to_vec();

    let runtime_rows = v_flex().gap_2().children(runtimes.iter().map(|runtime| {
        h_flex()
            .items_center()
            .justify_between()
            .p_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().accordion)
            .child(
                v_flex()
                    .gap_1()
                    .child(div().font_medium().child(runtime.name.clone()))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(runtime.path.to_string_lossy().into_owned()),
                    ),
            )
            .child(div().text_sm().child(format!("Java {}", runtime.version)))
    }));

    let content = v_flex()
        .gap_4()
        .child(section(
            "Java & runtime",
            "Phanerite manages Java runtimes for your instances.",
            runtime_rows,
            cx,
        ))
        .child(section(
            "Launch",
            "Global defaults used by every instance unless it overrides them.",
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "Memory allocation: {} GB",
                    settings.launch().memory
                )),
            cx,
        ))
        .child(section(
            "Appearance",
            "Choose the launcher appearance and density.",
            h_flex()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .child(format!("Accent: {}", settings.accent())),
                )
                .child(
                    Button::new("settings-accent")
                        .ghost()
                        .icon(IconName::Palette)
                        .label("Change accent"),
                ),
            cx,
        ))
        .child(section(
            "Aphanite",
            "Configure the server that provides shared configurations.",
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(settings.preferences().aphanite_server.clone()),
            cx,
        ))
        .child(Button::new("settings-updates").label("Check for updates"));
    page_shell(
        Some(page_title(
            "Settings",
            "Global preferences. Instance-specific options live in each instance's detail panel.",
            cx,
        )),
        content,
        cx,
    )
}

fn section(
    title: &'static str,
    description: &'static str,
    body: impl IntoElement,
    cx: &App,
) -> impl IntoElement {
    v_flex()
        .gap_3()
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
        .child(body)
}
