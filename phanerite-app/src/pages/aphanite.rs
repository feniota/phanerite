//! Server-provided configurations and installation status.
use super::{page_shell, route_button, widgets::*};
use crate::{assets::PhaIcon, components::instance_icon, route::Route, state::AppState};
use gpui_kit::component::{
    ActiveTheme as _, Icon, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};

pub fn render(app: Entity<AppState>, _: &mut Window, cx: &App) -> impl IntoElement {
    let instances = app
        .read(cx)
        .instances
        .read(cx)
        .aphanite()
        .cloned()
        .collect::<Vec<_>>();
    let installed = instances
        .iter()
        .filter(|instance| instance.last_played.is_some())
        .count();
    let server = app
        .read(cx)
        .settings
        .read(cx)
        .preferences()
        .aphanite_server
        .clone();
    let title = h_flex()
        .gap_2()
        .child(
            Icon::new(PhaIcon::Flame)
                .size_5()
                .text_color(crate::theme::flame()),
        )
        .child(
            div()
                .text_lg()
                .font_semibold()
                .child("Aphanite configurations"),
        )
        .child(badge(instances.len().to_string()));
    let title = header(
        title,
        "Modpack configurations provided by your connected Aphanite server.",
        div(),
        cx,
    );
    let mut content = v_flex().gap_4().child(
        panel(cx).p_4().child(
            h_flex()
                .gap_4()
                .child(
                    v_flex()
                        .min_w_0()
                        .flex_1()
                        .gap_1()
                        .child(muted("Currently connected to", cx))
                        .child(
                            div()
                                .text_sm()
                                .font_semibold()
                                .text_color(cx.theme().primary)
                                .child(if server.contains("aphanite.enita.cn") {
                                    "Enita’s Aphanite Server".into()
                                } else {
                                    server.clone()
                                }),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Icon::new(PhaIcon::Package)
                                .size_5()
                                .text_color(cx.theme().primary),
                        )
                        .child(div().text_sm().child(format!(
                            "{} configurations, {installed} installed",
                            instances.len()
                        ))),
                ),
        ),
    );
    if instances.is_empty() {
        content = content
            .child(empty(
                PhaIcon::Flame,
                "No Aphanite configurations",
                "Connect a server in Settings to discover modpack configurations.",
                cx,
            ))
            .child(route_button(
                "aphanite-settings",
                "Open settings",
                Route::Settings,
                app.clone(),
            ));
    }
    for instance in instances {
        let target = instance.reference();
        let favorite_target = target.clone();
        let app_open = app.clone();
        let app_favorite = app.clone();
        content = content.child(
            panel(cx).child(
                h_flex()
                    .child(
                        Button::new(format!("aphanite-open-{}", instance.id))
                            .ghost()
                            .h_auto()
                            .p_3()
                            .min_w_0()
                            .flex_1()
                            .accessibility_label(format!("Open {}", instance.name))
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_3()
                                    .child(instance_icon::render(&instance, cx))
                                    .child(
                                        v_flex()
                                            .min_w_0()
                                            .flex_1()
                                            .gap_1()
                                            .text_left()
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_medium()
                                                    .truncate()
                                                    .child(instance.name.clone()),
                                            )
                                            .child(
                                                muted(
                                                    format!(
                                                        "{} · MC {} · {} mods",
                                                        instance.loader.label(),
                                                        instance.mc_version,
                                                        instance.enabled_mods()
                                                    ),
                                                    cx,
                                                )
                                                .text_xs(),
                                            ),
                                    )
                                    .child(badge(if instance.last_played.is_some() {
                                        "Installed"
                                    } else {
                                        "Not installed"
                                    }))
                                    .child(
                                        Icon::new(PhaIcon::ArrowRight)
                                            .size_4()
                                            .text_color(cx.theme().muted_foreground),
                                    ),
                            )
                            .on_click(move |_, _, cx| {
                                app_open.update(cx, |app, cx| app.open_instance(target.clone(), cx))
                            }),
                    )
                    .child(
                        div().pr_2().child(
                            favorite_button(
                                format!("aphanite-favorite-{}", instance.id),
                                instance.favorite,
                                cx,
                            )
                            .on_click(move |_, _, cx| {
                                app_favorite
                                    .read(cx)
                                    .instances
                                    .clone()
                                    .update(cx, |store, cx| {
                                        if store.toggle_favorite(&favorite_target) {
                                            cx.notify();
                                        }
                                    });
                            }),
                        ),
                    ),
            ),
        );
    }
    page_shell(Some(title), content, cx)
}
