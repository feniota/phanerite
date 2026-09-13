//! Instance list page and instance-level actions.

use gpui_kit::base::motion::{Transition, transition};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IndexPath, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    menu::{DropdownMenu as _, PopupMenuItem},
    select::{Select, SelectState},
    tag::Tag,
    v_flex,
};
use gpui_kit::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
    text,
};
use std::time::Duration;

use crate::{
    assets::PhaIcon,
    components::{instance_actions, instance_create_dialog},
    route::Route,
    state::AppState,
};

use super::{page_shell, widgets::favorite_button};

fn instance_card(
    instance: &crate::state::InstanceSummary,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    let reference = instance.reference();
    let running = app.read(cx).sessions.read(cx).is_running(&reference);
    let play_reference = reference.clone();
    let favorite_reference = reference.clone();
    let menu_reference = reference.clone();
    let instance_name = instance.name.clone();
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
                        .text_xs()
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
                .child(
                    Button::new(format!("instance-play-{}", instance.id))
                        .icon(if running {
                            PhaIcon::Square
                        } else {
                            PhaIcon::Play
                        })
                        .tooltip(if running {
                            "Stop instance"
                        } else {
                            "Play instance"
                        })
                        .accessibility_label(if running {
                            "Stop instance"
                        } else {
                            "Play instance"
                        })
                        .on_click({
                            let app = app.clone();
                            move |_, window, cx| {
                                cx.stop_propagation();
                                instance_actions::play(&play_reference, &app, window, cx);
                            }
                        }),
                )
                .child(
                    favorite_button(
                        format!("instance-favorite-{}", instance.id),
                        instance.favorite,
                        cx,
                    )
                    .on_click({
                        let app = app.clone();
                        move |_, _, cx| {
                            cx.stop_propagation();
                            app.read(cx).instances.clone().update(cx, |store, cx| {
                                if store.toggle_favorite(&favorite_reference) {
                                    cx.notify();
                                }
                            });
                        }
                    }),
                )
                .child(
                    Button::new(format!("instance-menu-{}", instance.id))
                        .icon(PhaIcon::EllipsisVertical)
                        .ghost()
                        .accessibility_label("Instance actions")
                        .on_click(|_, _, cx| cx.stop_propagation())
                        .dropdown_menu(move |menu, _window, _cx| {
                            let launch = menu_reference.clone();
                            let duplicate = menu_reference.clone();
                            let logs = menu_reference.clone();
                            let folder = menu_reference.clone();
                            let delete = menu_reference.clone();
                            let launch_app = app.clone();
                            let duplicate_app = app.clone();
                            let logs_app = app.clone();
                            let delete_app = app.clone();
                            let name = instance_name.clone();
                            menu.item(
                                PopupMenuItem::new(if running { "Stop" } else { "Launch" })
                                    .icon(if running {
                                        PhaIcon::Square
                                    } else {
                                        PhaIcon::Play
                                    })
                                    .on_click(move |_, window, cx| {
                                        instance_actions::play(&launch, &launch_app, window, cx)
                                    }),
                            )
                            .separator()
                            .item(
                                PopupMenuItem::new("Duplicate")
                                    .icon(PhaIcon::Copy)
                                    .on_click(move |_, _, cx| {
                                        instance_actions::duplicate(&duplicate, &duplicate_app, cx)
                                    }),
                            )
                            .item(
                                PopupMenuItem::new("Open folder")
                                    .icon(PhaIcon::FolderOpen)
                                    .on_click(move |_, window, cx| {
                                        instance_actions::open_folder(&folder, window, cx)
                                    }),
                            )
                            .item(
                                PopupMenuItem::new("Game logs")
                                    .icon(PhaIcon::ScrollText)
                                    .on_click(move |_, _, cx| {
                                        logs_app.update(cx, |app, cx| {
                                            app.push(Route::Logs(logs.clone()), cx)
                                        })
                                    }),
                            )
                            .separator()
                            .item(
                                PopupMenuItem::new("Delete…")
                                    .icon(PhaIcon::Trash2)
                                    .on_click(move |_, window, cx| {
                                        instance_actions::delete(
                                            delete.clone(),
                                            name.clone(),
                                            delete_app.clone(),
                                            window,
                                            cx,
                                        )
                                    }),
                            )
                        }),
                ),
        )
        .into_any_element()
}

pub fn render(app: Entity<AppState>, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
    let search = instance_search_input.read(cx).value().trim().to_lowercase();
    let loader = loader_select_state
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or("All loaders");
    let count = app.read(cx).instances.read(cx).len();
    let instances: Vec<_> = app
        .read(cx)
        .instances
        .read(cx)
        .all()
        .iter()
        .filter(|instance| {
            (search.is_empty() || instance.name.to_lowercase().contains(&search))
                && (loader == "All loaders" || instance.loader.label() == loader)
        })
        .cloned()
        .collect();
    let content = if instances.is_empty() {
        v_flex()
            .items_center()
            .justify_center()
            .h_full()
            .gap_3()
            .child(div().text_lg().font_semibold().child(if count == 0 {
                "No instances found"
            } else {
                "No matching instances"
            }))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(if count == 0 {
                        "Create your first instance to get started."
                    } else {
                        "Try a different search or loader filter."
                    }),
            )
            .child(
                Button::new("create-instance-empty")
                    .primary()
                    .icon(PhaIcon::Plus)
                    .label("Create instance")
                    .on_click({
                        let app = app.clone();
                        move |_, window, cx| instance_create_dialog::open(window, cx, app.clone())
                    }),
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
                                .child(format!("{count}")),
                        ),
                )
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .whitespace_normal()
                        .child(text!("Your isolated Minecraft installations.")),
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
                        .prefix(Icon::new(PhaIcon::Search).small()),
                )
                .child(
                    super::widgets::control_slot(Select::new(&loader_select_state).w_full())
                        .w(gpui_kit::rems(8.)),
                )
                .child(
                    Button::new("instance-new_instance_button")
                        .icon(Icon::new(PhaIcon::Plus))
                        .primary()
                        .compact()
                        .child(div().text_sm().child("New instance"))
                        .text_sm()
                        .on_click(move |_, window, cx| {
                            instance_create_dialog::open(window, cx, app.clone())
                        }),
                ),
        );
    page_shell(Some(title), content, cx)
}
