//! Home page containing launch controls and launcher overview content.

use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::{
    ActiveTheme as _, IconName, Sizable as _, StyledExt,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

use crate::{route::Route, state::AppState};

use super::{page_shell, route_button};

pub fn render(app: Entity<AppState>, _: &mut Window, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let instances = state.instances.read(cx);
    let all = instances.all();
    let recommended = all
        .iter()
        .find(|instance| instance.name == "Old Faithful")
        .or_else(|| all.first());
    let content = v_flex()
        .gap_6()
        .children(recommended.map(|instance| {
            let reference = instance.reference();
            v_flex()
                .gap_3()
                .p_5()
                .rounded(cx.theme().radius)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().accordion)
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child("RECOMMENDED FOR YOU"),
                )
                .child(
                    h_flex()
                        .items_center()
                        .gap_4()
                        .child(crate::components::instance_icon::render(instance, cx))
                        .child(
                            v_flex()
                                .gap_1()
                                .flex_1()
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_semibold()
                                                .child(instance.name.clone()),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded_full()
                                                .bg(cx.theme().secondary)
                                                .text_xs()
                                                .child(format!("MC {}", instance.mc_version)),
                                        )
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded_full()
                                                .bg(cx.theme().secondary)
                                                .text_xs()
                                                .child(instance.loader.label()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(instance.description.clone()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!(
                                            "{} mods    {} packs    {} worlds    Java {}",
                                            instance.enabled_mods(),
                                            instance.resource_packs.len(),
                                            instance.worlds.len(),
                                            instance.java
                                        )),
                                ),
                        )
                        .child(
                            div()
                                .w(gpui::px(176.))
                                .h(gpui::px(160.))
                                .items_center()
                                .justify_center()
                                .bg(crate::palette::color(crate::palette::token::LAUNCH))
                                .child(
                                    Button::new("quick-play")
                                        .ghost()
                                        .icon(IconName::Play)
                                        .label("Play")
                                        .on_click({
                                            let app = app.clone();
                                            move |_, _, cx| {
                                                app.update(cx, |state, cx| {
                                                    state.push(
                                                        Route::InstanceDetail(reference.clone()),
                                                        cx,
                                                    )
                                                })
                                            }
                                        }),
                                ),
                        ),
                )
        }))
        .child(
            div()
                .text_xs()
                .font_semibold()
                .text_color(cx.theme().muted_foreground)
                .child("ALL INSTANCES"),
        )
        .child(v_flex().gap_2().children(all.iter().map(|instance| {
            let reference = instance.reference();
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
                        .child(div().font_medium().child(instance.name.clone()))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "{} · MC {} · {}",
                                    instance.loader.label(),
                                    instance.mc_version,
                                    instance.last_played.as_deref().unwrap_or("never played")
                                )),
                        ),
                )
                .child(
                    route_button(
                        format!("play-open-{}", instance.id),
                        "View",
                        Route::InstanceDetail(reference),
                        app.clone(),
                    )
                    .ghost()
                    .xsmall(),
                )
        })))
        .child(
            route_button(
                "play-manage",
                "Manage instances",
                Route::Instances,
                app.clone(),
            )
            .icon(IconName::ArrowRight)
            .ghost(),
        );
    page_shell(
        "Good afternoon.",
        format!(
            "{} instance{}",
            all.len(),
            if all.len() == 1 { "" } else { "s" }
        ),
        content,
        cx,
    )
}
