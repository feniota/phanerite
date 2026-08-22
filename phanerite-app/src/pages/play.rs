//! Home page containing launch controls and launcher overview content.

use crate::{
    assets::PhaIcon,
    route::Route,
    state::{AppState, InstanceSummary},
};
use gpui::{
    App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    scroll::ScrollableElement as _,
    v_flex,
};

use super::route_button;

fn instance_card(
    instance: &InstanceSummary,
    flame: bool,
    app: Entity<AppState>,
    cx: &App,
) -> impl IntoElement {
    let reference = instance.reference();
    let sessions = app.read(cx).sessions.read(cx);
    let running = sessions.is_running(&reference);

    h_flex()
        .w_full()
        .items_center()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().accordion)
        .child(
            h_flex()
                .min_w_0()
                .flex_1()
                .items_center()
                .gap_3()
                .p_3()
                .child(crate::components::instance_icon::render(instance, cx))
                .child(
                    v_flex()
                        .min_w_0()
                        .flex_1()
                        .gap_1()
                        .child(
                            h_flex()
                                .min_w_0()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_xs()
                                        .font_medium()
                                        .child(instance.name.clone()),
                                )
                                .when(instance.aphanite && flame, |this| {
                                    this.child(Icon::new(PhaIcon::Flame).size_3().text_color(
                                        crate::palette::color(crate::palette::token::FLAME),
                                    ))
                                }),
                        )
                        .child(
                            div()
                                .truncate()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "MC {} · {}{}",
                                    instance.mc_version,
                                    instance.loader.label(),
                                    instance
                                        .last_played
                                        .as_deref()
                                        .map(|last| format!(" · {last}"))
                                        .unwrap_or_default()
                                )),
                        ),
                )
                .when(running, |this| {
                    this.child(
                        div()
                            .size_2()
                            .flex_shrink_0()
                            .rounded_full()
                            .bg(cx.theme().primary),
                    )
                }),
        )
        .child(
            Button::new(format!("play-open-{}", instance.id))
                .mr_3()
                .ghost()
                .icon(IconName::Play)
                .on_click({
                    let app = app.clone();
                    move |_, _, cx| {
                        app.update(cx, |state, cx| {
                            state.push(Route::InstanceDetail(reference.clone()), cx)
                        })
                    }
                }),
        )
}

pub fn render(app: Entity<AppState>, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = app.read(cx);
    let all = state.instances.read(cx).all().to_vec();
    let recommended = all
        .iter()
        .find(|instance| instance.name == "Old Faithful")
        .or_else(|| all.first())
        .cloned();
    let server_owner_recommendations: Vec<_> = all
        .iter()
        .filter(|instance| instance.aphanite)
        .cloned()
        .collect();
    let account = state.accounts.read(cx).active().cloned();
    let greeting = "Good afternoon.";

    v_flex()
        .h_full()
        .gap_6()
        .overflow_y_scrollbar()
        .p_6()
        .child(
            h_flex()
                .flex_shrink_0()
                .items_center()
                .justify_between()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_lg().font_semibold().child(greeting))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "{} instance{}",
                                    all.len(),
                                    if all.len() == 1 { "" } else { "s" }
                                )),
                        ),
                )
                .child(
                    Button::new("play-account")
                        .outline()
                        .small()
                        .label(
                            account
                                .map(|item| item.username)
                                .unwrap_or_else(|| "Offline".into()),
                        )
                        .icon(IconName::ChevronRight)
                        .on_click({
                            let app = app.clone();
                            move |_, _, cx| {
                                app.update(cx, |state, cx| state.push(Route::Accounts, cx))
                            }
                        }),
                ),
        )
        .when_some(recommended, |this, instance| {
            this.child(
                v_flex()
                    .flex_shrink_0()
                    .child(
                        div()
                            .mb_2()
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().muted_foreground)
                            .child("RECOMMENDED FOR YOU"),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .overflow_hidden()
                            .pl_4()
                            .child(crate::components::instance_icon::render(&instance, cx))
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .flex_1()
                                    .gap_1()
                                    .py_4()
                                    .child(
                                        h_flex()
                                            .min_w_0()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .min_w_0()
                                                    .truncate()
                                                    .text_base()
                                                    .font_semibold()
                                                    .child(instance.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .px_2()
                                                    .rounded_full()
                                                    .bg(cx.theme().secondary)
                                                    .text_xs()
                                                    .child(format!("MC {}", instance.mc_version)),
                                            )
                                            .child(
                                                div()
                                                    .px_2()
                                                    .rounded_full()
                                                    .bg(cx.theme().secondary)
                                                    .text_xs()
                                                    .child(instance.loader.label()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(instance.description.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
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
                                crate::components::animated_button::render(
                                    "recommended-play",
                                    crate::palette::color(crate::palette::token::LAUNCH),
                                    crate::palette::color_alpha(
                                        crate::palette::token::LAUNCH,
                                        0.85,
                                    ),
                                    v_flex()
                                        .items_center()
                                        .justify_center()
                                        .size_full()
                                        .child(Icon::new(PhaIcon::PlayFilled).size_6()),
                                    {
                                        let app = app.clone();
                                        let reference = instance.reference();
                                        move |_, _, cx| {
                                            app.update(cx, |state, cx| {
                                                state.push(
                                                    Route::InstanceDetail(reference.clone()),
                                                    cx,
                                                )
                                            })
                                        }
                                    },
                                    window,
                                    cx,
                                )
                                .size(px(176.))
                                .h_full()
                                .rounded_none()
                                .rounded_r_xl(),
                            ),
                    ),
            )
        })
        .when(!server_owner_recommendations.is_empty(), |this| {
            this.child(
                v_flex()
                    .flex_shrink_0()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .mb_2()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new(PhaIcon::Flame).size_4().text_color(
                                        crate::palette::color(crate::palette::token::FLAME),
                                    ))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("RECOMMENDED BY YOUR SERVER OWNER"),
                                    ),
                            )
                            .child(
                                route_button(
                                    "play-aphanite",
                                    "Check out",
                                    Route::Aphanite,
                                    app.clone(),
                                )
                                .ghost()
                                .xsmall()
                                .icon(IconName::ArrowRight),
                            ),
                    )
                    .child(
                        v_flex().gap_2().children(
                            server_owner_recommendations
                                .iter()
                                .map(|instance| instance_card(instance, false, app.clone(), cx)),
                        ),
                    ),
            )
        })
        .child(
            v_flex()
                .flex_shrink_0()
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .mb_2()
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(cx.theme().muted_foreground)
                                .child("ALL INSTANCES"),
                        )
                        .child(
                            route_button("play-manage", "Manage", Route::Instances, app.clone())
                                .ghost()
                                .xsmall()
                                .icon(IconName::ArrowRight),
                        ),
                )
                .child(v_flex().gap_3().children(all.chunks(2).map(|row| {
                    h_flex()
                        .w_full()
                        .gap_3()
                        .children(row.iter().map(|instance| {
                            h_flex()
                                .flex_1()
                                .child(instance_card(instance, true, app.clone(), cx))
                        }))
                        .when(row.len() == 1, |row| row.child(div().flex_1()))
                }))),
        )
}
