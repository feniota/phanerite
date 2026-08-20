//! Detail page for a selected Minecraft instance.

use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::{
    ActiveTheme as _, IconName, Sizable as _, StyledExt,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

use super::{back_button, instance_exists, missing_resource};
use crate::{
    route::{CrashRef, InstanceRef, Route},
    state::AppState,
};

pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    _: &mut Window,
    cx: &App,
) -> impl IntoElement {
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
    let row = |id: String, label: &'static str, route: Route, app: Entity<AppState>| {
        h_flex()
            .items_center()
            .justify_between()
            .p_4()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(div().font_medium().child(label))
            .child(super::route_button(id, "Open", route, app).ghost().xsmall())
    };
    let mut cards = v_flex()
        .gap_4()
        .child(back_button(app.clone()))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_lg().font_semibold().child(instance.name.clone()))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(instance.description.clone()),
                        ),
                )
                .child(
                    Button::new("detail-play")
                        .primary()
                        .icon(IconName::Play)
                        .label(format!("Play {}", instance.name)),
                ),
        )
        .child(
            v_flex()
                .rounded(cx.theme().radius)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().accordion)
                .child(row(
                    "detail-mods".into(),
                    "Mods",
                    Route::Mods(reference.clone()),
                    app.clone(),
                ))
                .child(row(
                    "detail-packs".into(),
                    "Resource packs",
                    Route::Packs(reference.clone()),
                    app.clone(),
                ))
                .child(row(
                    "detail-shaders".into(),
                    "Shader packs",
                    Route::Shaders(reference.clone()),
                    app.clone(),
                ))
                .child(row(
                    "detail-worlds".into(),
                    "Worlds",
                    Route::Worlds(reference.clone()),
                    app.clone(),
                )),
        )
        .child(
            v_flex()
                .gap_3()
                .p_4()
                .rounded(cx.theme().radius)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().accordion)
                .child(div().font_semibold().child("Instance information"))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Game version  {}", instance.mc_version)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Mod loader  {}", instance.loader_label())),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Created  {}", instance.created_at)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Launches  {}", instance.play_count)),
                ),
        );
    if let Some(crash) = instance.last_crash_id.clone() {
        cards = cards.child(
            super::route_button(
                "detail-crash",
                "View last crash report",
                Route::Crash(CrashRef::new(reference.storage_id, crash)),
                app.clone(),
            )
            .warning(),
        );
    }
    cards = cards.child(
        h_flex()
            .gap_2()
            .child(super::route_button(
                "detail-settings",
                "Launch settings",
                Route::LaunchSettings(reference.clone()),
                app.clone(),
            ))
            .child(super::route_button(
                "detail-logs",
                "Game logs",
                Route::Logs(reference.clone()),
                app,
            )),
    );
    cards.into_any_element()
}
