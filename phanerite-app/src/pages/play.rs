//! Home page containing launch controls and launcher overview content.

use crate::{
    assets::PhaIcon,
    route::Route,
    state::{AppState, InstanceSummary},
};
use chrono::{Local, Timelike as _};
use gpui_kit::base::motion::{Transition, transition};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use std::time::Duration;

use super::route_button;

fn instance_card(
    scope: &str,
    instance: &InstanceSummary,
    flame: bool,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    let reference = instance.reference();
    let sessions = app.read(cx).sessions.read(cx);
    let running = sessions.is_running(&reference);
    let card_id = format!("{scope}-instance-card-{}", instance.id);
    let button_reference = reference.clone();
    let hovered = window.use_keyed_state(format!("{card_id}-hover"), cx, |_, _| false);
    let background = transition(
        (card_id.clone(), "background"),
        if *hovered.read(cx) {
            cx.theme().sidebar_accent.opacity(0.8)
        } else {
            cx.theme().accordion
        },
        Transition::new(Duration::from_millis(250)),
        window,
        cx,
    );

    h_flex()
        .id(card_id)
        .cursor_pointer()
        .w_full()
        .items_center()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(background)
        .on_hover(move |is_hovered, _, cx| {
            hovered.update(cx, |hovered, cx| {
                if *hovered != *is_hovered {
                    *hovered = *is_hovered;
                    cx.notify();
                }
            })
        })
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
        .on_click({
            let app = app.clone();
            move |_, _, cx| {
                app.update(cx, |state, cx| {
                    state.push(Route::InstanceDetail(reference.clone()), cx)
                })
            }
        })
        .child(
            Button::new(format!("{scope}-play-open-{}", instance.id))
                .mr_3()
                .ghost()
                .icon(IconName::Play)
                .on_click({
                    let app = app.clone();
                    move |_, _, cx| {
                        app.update(cx, |state, cx| {
                            state.push(Route::InstanceDetail(button_reference.clone()), cx)
                        })
                    }
                }),
        )
        .into_any_element()
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
    let account = state.accounts.read(cx).active();
    let greeting = match Local::now().hour() {
        5..12 => "Good morning.",
        12..18 => "Good afternoon.",
        18..22 => "Good evening.",
        _ => "Welcome back.",
    };

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
            let card_id = format!("recommended-instance-card-{}", instance.id);
            let hovered = window.use_keyed_state(format!("{card_id}-hover"), cx, |_, _| false);
            let background = transition(
                (card_id.clone(), "background"),
                if *hovered.read(cx) {
                    cx.theme().sidebar_accent.opacity(0.8)
                } else {
                    cx.theme().accordion
                },
                Transition::new(Duration::from_millis(250)),
                window,
                cx,
            );
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
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .bg(background)
                            .overflow_hidden()
                            .child(
                                h_flex()
                                    .id(card_id)
                                    .cursor_pointer()
                                    .min_w_0()
                                    .flex_1()
                                    .items_center()
                                    .gap_4()
                                    .pl_4()
                                    .on_hover(move |is_hovered, _, cx| {
                                        hovered.update(cx, |hovered, cx| {
                                            if *hovered != *is_hovered {
                                                *hovered = *is_hovered;
                                                cx.notify();
                                            }
                                        })
                                    })
                                    .on_click({
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
                                    })
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
                                                            .child(format!(
                                                                "MC {}",
                                                                instance.mc_version
                                                            )),
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
                                .map(|instance| {
                                    instance_card(
                                        "server-owner",
                                        instance,
                                        false,
                                        app.clone(),
                                        window,
                                        cx,
                                    )
                                }),
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
                                .child(instance_card(
                                    "all",
                                    instance,
                                    true,
                                    app.clone(),
                                    window,
                                    cx,
                                ))
                        }))
                        .when(row.len() == 1, |row| row.child(div().flex_1()))
                }))),
        )
}
