//! Instance list page and instance-level actions.

use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::{
    ActiveTheme as _, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

use crate::{route::Route, state::AppState};

use super::{page_shell, route_button};

pub fn render(app: Entity<AppState>, _window: &mut Window, cx: &App) -> impl IntoElement {
    let instances = app.read(cx).instances.read(cx).all().to_vec();
    let content = if instances.is_empty() {
        v_flex()
            .items_center()
            .justify_center()
            .h_full()
            .gap_3()
            .child(div().text_lg().font_semibold().child("No instances found"))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Create your first instance to get started."),
            )
            .child(
                Button::new("create-instance-empty")
                    .primary()
                    .icon(IconName::Plus)
                    .label("Create instance"),
            )
            .into_any_element()
    } else {
        v_flex()
            .gap_2()
            .children(instances.iter().map(|instance| {
                let reference = instance.reference();
                h_flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .p_3()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().accordion)
                    .child(crate::components::instance_icon::render(instance, cx))
                    .child(
                        v_flex()
                            .gap_1()
                            .child(div().font_medium().child(instance.name.clone()))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{} · MC {} · {} launches",
                                        instance.loader.label(),
                                        instance.mc_version,
                                        instance.play_count
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                route_button(
                                    format!("instance-detail-{}", instance.id),
                                    "Details",
                                    Route::InstanceDetail(reference),
                                    app.clone(),
                                )
                                .xsmall(),
                            )
                            .child(
                                Button::new(format!("instance-play-{}", instance.id))
                                    .primary()
                                    .icon(IconName::Play)
                                    .label("Play")
                                    .xsmall(),
                            ),
                    )
            }))
            .into_any_element()
    };
    let content = v_flex()
        .gap_3()
        .child(
            Button::new("create-instance")
                .primary()
                .icon(IconName::Plus)
                .label("New instance")
                .on_click({
                    let app = app.clone();
                    move |_, window, cx| {
                        crate::components::instance_create_dialog::open(window, cx, app.clone())
                    }
                }),
        )
        .child(content);
    page_shell(
        "Instances",
        "Each instance is an isolated game installation with its own mods, packs and settings.",
        content,
        cx,
    )
}
