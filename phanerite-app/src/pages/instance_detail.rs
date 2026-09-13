//! Instance overview, resource navigation and pinned launch controls.

use super::{back_button, missing_resource, route_button, widgets::*};
use crate::{
    assets::PhaIcon,
    components::{
        form::{self, Choice},
        instance_actions, instance_icon,
    },
    route::{CrashRef, InstanceRef, Route},
    state::{AppState, LaunchField, LaunchValue},
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, StyledExt as _,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex,
    menu::{DropdownMenu as _, PopupMenuItem},
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::{
    App, Entity, IntoElement as _, ParentElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _,
};

pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    let Some(instance) = app.read(cx).instances.read(cx).find(reference).cloned() else {
        return missing_resource("instance", app).into_any_element();
    };
    let running = app.read(cx).sessions.read(cx).is_running(reference);
    let mut name = h_flex().gap_2().min_w_0().child(
        div()
            .text_lg()
            .font_semibold()
            .truncate()
            .child(instance.name.clone()),
    );
    if instance.aphanite {
        name = name.child(
            Icon::new(PhaIcon::Flame)
                .size_4()
                .text_color(crate::theme::flame()),
        );
    }
    if running {
        name = name.child(badge("Running"));
    }
    let title = v_flex()
        .gap_3()
        .flex_shrink_0()
        .px_6()
        .pt_4()
        .pb_4()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(h_flex().child(back_button(app.clone())))
        .child(
            h_flex()
                .gap_3()
                .child(instance_icon::render(&instance, cx))
                .child(
                    v_flex()
                        .gap_1()
                        .min_w_0()
                        .flex_1()
                        .child(name)
                        .child(muted(instance.description.clone(), cx)),
                )
                .child(
                    favorite_button("detail-favorite", instance.favorite, cx)
                        .when(!instance.favorite, |button| button.outline())
                        .on_click({
                            let app = app.clone();
                            let reference = reference.clone();
                            move |_, _, cx| {
                                app.read(cx).instances.clone().update(cx, |store, cx| {
                                    if store.toggle_favorite(&reference) {
                                        cx.notify();
                                    }
                                });
                            }
                        }),
                ),
        );
    let mut body = v_flex().gap_6().child(
        h_flex()
            .gap_2()
            .child(badge(format!("MC {}", instance.mc_version)))
            .child(badge(instance.loader_label()))
            .child(badge(format!("Java {}", instance.java))),
    );
    if let Some(report) = instance.last_crash_id.as_ref().and_then(|id| {
        app.read(cx)
            .crashes
            .read(cx)
            .find(&CrashRef::new(reference.storage.clone(), id.clone()))
    }) {
        body = body.child(
            panel(cx)
                .p_4()
                .border_color(cx.theme().danger.opacity(0.4))
                .child(
                    h_flex()
                        .gap_3()
                        .child(
                            Icon::new(PhaIcon::TriangleAlert)
                                .size_5()
                                .text_color(cx.theme().danger),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .flex_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_medium()
                                        .text_color(cx.theme().danger)
                                        .child(format!("Last launch crashed · {}", report.when)),
                                )
                                .child(
                                    muted(format!("Exit code {}", report.exit_code), cx).text_xs(),
                                ),
                        )
                        .child(
                            route_button(
                                "detail-crash",
                                "View report",
                                Route::Crash(report.reference()),
                                app.clone(),
                            )
                            .icon(PhaIcon::ChevronRight),
                        ),
                ),
        );
    }
    let configurations = [
        (
            "mods",
            PhaIcon::Puzzle,
            "Mods",
            format!(
                "{}/{} enabled",
                instance.enabled_mods(),
                instance.mods.len()
            ),
            Route::Mods(reference.clone()),
        ),
        (
            "packs",
            PhaIcon::Palette,
            "Resource packs",
            format!("{} installed", instance.resource_packs.len()),
            Route::Packs(reference.clone()),
        ),
        (
            "shaders",
            PhaIcon::Sparkles,
            "Shader packs",
            format!("{} installed", instance.shader_packs.len()),
            Route::Shaders(reference.clone()),
        ),
        (
            "worlds",
            PhaIcon::Globe,
            "Worlds",
            format!("{} saved", instance.worlds.len()),
            Route::Worlds(reference.clone()),
        ),
    ];
    body = body.child(
        panel(cx).children(configurations.into_iter().enumerate().map(
            |(ix, (id, icon, label, description, route))| {
                let app = app.clone();
                Button::new(format!("detail-{id}"))
                    .ghost()
                    .w_full()
                    .h_auto()
                    .p_4()
                    .when(ix > 0, |button| {
                        button.border_t_1().border_color(cx.theme().border)
                    })
                    .accessibility_label(format!("Open {label}"))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .child(icon_tile(icon, cx))
                            .child(
                                v_flex()
                                    .gap_1()
                                    .min_w_0()
                                    .flex_1()
                                    .text_left()
                                    .child(div().text_base().font_medium().child(label))
                                    .child(muted(description, cx)),
                            )
                            .child(
                                Icon::new(PhaIcon::ChevronRight)
                                    .size_4()
                                    .text_color(cx.theme().muted_foreground),
                            ),
                    )
                    .on_click(move |_, _, cx| app.update(cx, |app, cx| app.push(route.clone(), cx)))
            },
        )),
    );
    let information = v_flex().gap_3().children(
        [
            ("Game version", instance.mc_version.clone()),
            ("Mod loader", instance.loader_label()),
            ("Created", instance.created_at.clone()),
            (
                "Last played",
                instance
                    .last_played
                    .clone()
                    .unwrap_or_else(|| "Never".into()),
            ),
            ("Launches", instance.play_count.to_string()),
        ]
        .into_iter()
        .map(|(label, value)| {
            h_flex()
                .gap_4()
                .justify_between()
                .child(muted(label, cx))
                .child(div().text_sm().font_medium().child(value))
        }),
    );
    let runtimes = app.read(cx).settings.read(cx).runtimes().to_vec();
    let current_runtime = runtimes
        .iter()
        .find(|runtime| runtime.id == instance.java_runtime_id);
    let target_read = reference.clone();
    let target_write = reference.clone();
    let runtime = form::select(
        instance_key(reference, &format!("detail-java-{}", runtimes.len())),
        app.clone(),
        runtimes
            .iter()
            .map(|runtime| {
                Choice::new(
                    runtime.id.clone(),
                    format!(
                        "{} (Java {}){}",
                        runtime.name,
                        runtime.version,
                        if runtime.managed { " · Managed" } else { "" }
                    ),
                )
            })
            .collect(),
        move |app, cx| {
            app.instances
                .read(cx)
                .find(&target_read)
                .map(|instance| instance.java_runtime_id.clone())
                .unwrap_or_default()
        },
        move |app, value, cx| {
            if let Some(version) = app
                .settings
                .read(cx)
                .runtime(&value)
                .map(|runtime| runtime.version)
            {
                app.instances.update(cx, |store, cx| {
                    if store.set_java_runtime(&target_write, &value, version) {
                        cx.notify();
                    }
                });
            }
        },
        window,
        cx,
    )
    .w_full()
    .accessibility_label("Java runtime");
    body = body.child(section(
        "Instance information",
        "",
        information
            .child(field("Java runtime", control_slot(runtime)))
            .child(
                muted(
                    current_runtime
                        .map(|runtime| runtime.path.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Select a Java runtime in Settings.".into()),
                    cx,
                )
                .text_xs()
                .font_family(crate::theme::MONO_FONT_FAMILY),
            ),
        cx,
    ));
    let launch = instance
        .launch_overrides
        .resolve(app.read(cx).settings.read(cx).launch());
    let overridden = instance.launch_overrides.is_overridden(LaunchField::Memory);
    let read_target = reference.clone();
    let write_target = reference.clone();
    let slider = form::slider(
        instance_key(reference, "detail-memory"),
        app.clone(),
        move |app, cx| super::launch_settings::effective(app, Some(&read_target), cx).memory,
        move |app, value, cx| {
            app.instances.update(cx, |store, cx| {
                if store.set_launch_override(
                    &write_target,
                    LaunchField::Memory,
                    LaunchValue::Number(value),
                ) {
                    cx.notify();
                }
            });
        },
        window,
        cx,
    );
    let mut memory = panel(cx)
        .p_4()
        .gap_4()
        .child(
            h_flex()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .font_medium()
                        .child(format!("Maximum memory, {} GB", launch.memory)),
                )
                .child(
                    muted(
                        if overridden {
                            "Overridden"
                        } else {
                            "Inherited from global"
                        },
                        cx,
                    )
                    .text_xs(),
                ),
        )
        .child(slider);
    if overridden {
        let app = app.clone();
        let target = reference.clone();
        memory = memory.child(
            div().child(
                Button::new("detail-memory-reset")
                    .ghost()
                    .small()
                    .label("Reset to global")
                    .on_click(move |_, _, cx| {
                        app.read(cx).instances.clone().update(cx, |store, cx| {
                            if store.clear_launch_override(&target, LaunchField::Memory) {
                                cx.notify();
                            }
                        });
                    }),
            ),
        );
    }
    body = body.child(memory);
    let play_app = app.clone();
    let target = reference.clone();
    let launch_color = if running {
        cx.theme().danger.opacity(0.12)
    } else {
        crate::theme::launch()
    };
    let launch_foreground = if running {
        cx.theme().danger
    } else {
        crate::theme::launch_foreground()
    };
    let footer = h_flex()
        .flex_shrink_0()
        .gap_2()
        .p_4()
        .border_t_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().group_box)
        .child(
            Button::new("detail-play")
                .large()
                .h(gpui_kit::rems(3.5))
                .flex_1()
                .min_w_0()
                .custom(
                    ButtonCustomVariant::new(cx)
                        .color(launch_color)
                        .foreground(launch_foreground)
                        .hover(launch_color.opacity(0.85))
                        .active(launch_color.opacity(0.7)),
                )
                // Keep the prototype's launch surface opaque at rest.
                .bg(launch_color)
                .icon(if running {
                    PhaIcon::Square
                } else {
                    PhaIcon::PlayFilled
                })
                .label(format!(
                    "{} {}",
                    if running { "Stop" } else { "Play" },
                    instance.name
                ))
                .on_click(move |_, window, cx| {
                    instance_actions::play(&target, &play_app, window, cx)
                }),
        )
        .child(
            route_button(
                "detail-settings",
                "",
                Route::LaunchSettings(reference.clone()),
                app.clone(),
            )
            .icon(PhaIcon::Settings)
            .large()
            .size(gpui_kit::rems(3.5))
            .tooltip("Instance launch settings")
            .accessibility_label("Instance launch settings"),
        )
        .child(
            icon_button("detail-actions", PhaIcon::Ellipsis, "Instance actions")
                .large()
                .size(gpui_kit::rems(3.5))
                .border_1()
                .border_color(cx.theme().border)
                .dropdown_menu({
                    let reference = reference.clone();
                    let name = instance.name.clone();
                    move |menu, _, _| {
                        let folder = reference.clone();
                        let duplicate = reference.clone();
                        let delete = reference.clone();
                        let logs = reference.clone();
                        let app_duplicate = app.clone();
                        let app_delete = app.clone();
                        let app_logs = app.clone();
                        let name = name.clone();
                        menu.item(
                            PopupMenuItem::new("Open folder")
                                .icon(PhaIcon::FolderOpen)
                                .on_click(move |_, window, cx| {
                                    instance_actions::open_folder(&folder, window, cx)
                                }),
                        )
                        .item(
                            PopupMenuItem::new("Duplicate")
                                .icon(PhaIcon::Copy)
                                .on_click(move |_, _, cx| {
                                    instance_actions::duplicate(&duplicate, &app_duplicate, cx)
                                }),
                        )
                        .item(
                            PopupMenuItem::new("Game logs")
                                .icon(PhaIcon::ScrollText)
                                .on_click(move |_, _, cx| {
                                    app_logs.update(cx, |app, cx| {
                                        app.push(Route::Logs(logs.clone()), cx)
                                    })
                                }),
                        )
                        .separator()
                        .item(
                            PopupMenuItem::new("Delete instance…")
                                .icon(PhaIcon::Trash2)
                                .on_click(move |_, window, cx| {
                                    instance_actions::delete(
                                        delete.clone(),
                                        name.clone(),
                                        app_delete.clone(),
                                        window,
                                        cx,
                                    )
                                }),
                        )
                    }
                }),
        );
    v_flex()
        .size_full()
        .min_w_0()
        .min_h_0()
        .child(title)
        .child(
            div()
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .child(div().p_6().child(body)),
        )
        .child(footer)
        .into_any_element()
}
