//! Instance list page and instance-level actions.

use gpui::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px, text,
};
use gpui_base::motion::{Transition, transition};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, IndexPath, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    menu::{DropdownMenu as _, PopupMenuItem},
    select::{Select, SelectState},
    tag::Tag,
    v_flex,
};
use std::time::Duration;

use crate::{assets::PhaIcon, route::Route, state::AppState};

use super::page_shell;

fn instance_card(
    instance: &crate::state::InstanceSummary,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    let reference = instance.reference();
    let card_id = format!("instance-card-{}", instance.id);
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
        .items_center()
        .gap_3()
        .p_3()
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
        .on_click({
            let app = app.clone();
            move |_, _, cx| {
                app.update(cx, |state, cx| {
                    state.push(Route::InstanceDetail(reference.clone()), cx)
                })
            }
        })
        .child(crate::components::instance_icon::render(instance, cx))
        .child(
            v_flex()
                .child(
                    h_flex()
                        .items_center()
                        .font_medium()
                        .gap_2()
                        .child(instance.name.clone())
                        .when(instance.aphanite, |e| {
                            e.child(
                                Icon::new(PhaIcon::Flame).text_color(crate::palette::color(
                                    crate::palette::token::FLAME,
                                )),
                            )
                        }),
                )
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
        .child(div().flex_1())
        .child(
            h_flex()
                .gap_2()
                .child(Button::new(format!("instance-play-{}", instance.id)).icon(IconName::Play))
                .child(
                    Button::new(format!("instance-favorite-{}", instance.id))
                        .icon(IconName::Star)
                        .ghost(),
                )
                .child(
                    Button::new(format!("instance-menu-{}", instance.id))
                        .icon(IconName::EllipsisVertical)
                        .ghost()
                        .dropdown_menu(|menu, _window, _cx| {
                            menu.item(PopupMenuItem::new("Launch").icon(IconName::Play))
                                .separator()
                                .item(PopupMenuItem::new("Duplicate").icon(IconName::Copy))
                                .item(PopupMenuItem::new("Export…"))
                                .item(PopupMenuItem::new("Delete").icon(PhaIcon::Trash2))
                        }),
                ),
        )
        .into_any_element()
}

pub fn render(app: Entity<AppState>, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
            .children(
                instances
                    .iter()
                    .map(|instance| instance_card(instance, app.clone(), window, cx)),
            )
            .into_any_element()
    };
    let content = v_flex().gap_3().child(content);
    let instance_search_input =
        window.use_keyed_state("instances-search-input", cx, |window, cx| {
            InputState::new(window, cx).placeholder("Search instances…")
        });
    let loader_select_state =
        window.use_keyed_state("instances-loader-select", cx, |window, cx| {
            SelectState::new(
                vec!["All loaders", "Vanilla", "Fabric", "NeoForge", "Forge"],
                Some(IndexPath::new(0)),
                window,
                cx,
            )
        });

    let title = h_flex()
        .px_6()
        .pt_6()
        .pb_4()
        .items_center()
        .border_b_1()
        .border_color(cx.theme().border)
        .w_full()
        .relative()
        .child(
            div()
                .min_w_0()
                .w_full()
                .flex_grow_1()
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Icon::from(PhaIcon::Layers)
                                .text_color(crate::palette::color(crate::palette::token::PRIMARY)),
                        )
                        .child(div().child("Instances").text_lg().font_semibold())
                        .child(
                            Tag::secondary()
                                .xsmall()
                                .rounded_full()
                                .border_0()
                                .child(format!("{}", instances.len())),
                        ),
                )
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .whitespace_normal()
                        .child(text!(
                            "Lorem ipsum dolor sit amet, consectetur adipiscing elit."
                        )),
                ),
        )
        .child(
            h_flex()
                .flex_grow_0()
                .items_center()
                .gap_2()
                .child(
                    Input::new(&instance_search_input)
                        .min_w(px(224.))
                        .cleanable(true)
                        .prefix(Icon::new(IconName::Search).small()),
                )
                .child(Select::new(&loader_select_state).w(px(128.)))
                .child(
                    Button::new("instance-new_instance_button")
                        .icon(Icon::new(IconName::Plus))
                        .primary()
                        .compact()
                        .child(div().text_sm().child("New instance"))
                        .text_sm(),
                ),
        );
    page_shell(Some(title), content, cx)
}
