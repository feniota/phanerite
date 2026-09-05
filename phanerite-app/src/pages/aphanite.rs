//! Aphanite service page showing the connected server and its status.

use gpui_kit::component::{ActiveTheme as _, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};

use super::{page_shell, page_title, route_button};
use crate::{route::Route, state::AppState};

pub fn render(app: Entity<AppState>, _: &mut Window, cx: &App) -> impl IntoElement {
    let instances = app
        .read(cx)
        .instances
        .read(cx)
        .aphanite()
        .cloned()
        .collect::<Vec<_>>();
    let content = v_flex()
        .gap_4()
        .child(
            div()
                .p_4()
                .rounded(cx.theme().radius)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().accordion)
                .child("Currently connected to Enita's Aphanite Server"),
        )
        .children(instances.iter().map(|instance| {
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
                                    "{} · MC {} · {} mods",
                                    instance.loader.label(),
                                    instance.mc_version,
                                    instance.enabled_mods()
                                )),
                        ),
                )
                .child(
                    route_button(
                        format!("aphanite-{}", instance.id),
                        "View",
                        Route::InstanceDetail(reference),
                        app.clone(),
                    )
                    .xsmall(),
                )
        }));
    page_shell(
        Some(page_title(
            "Aphanite configurations",
            "Modpack configurations provided by your connected Aphanite server.",
            cx,
        )),
        content,
        cx,
    )
}
