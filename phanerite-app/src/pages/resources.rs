//! Instance resource collections share one row and action vocabulary.

use super::{missing_resource, page_shell, widgets::*};
use crate::{
    assets::PhaIcon,
    components::{
        add_resources_dialog::{self, ResourceMode},
        confirm_dialog, instance_actions,
    },
    route::InstanceRef,
    state::{AppState, LaunchField, LaunchValue, Loader, QuickPlayMode},
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IndexPath, StyledExt as _,
    button::Button,
    h_flex,
    input::{Input, InputState},
    menu::{DropdownMenu as _, PopupMenuItem},
    select::{Select, SelectState},
    switch::Switch,
    v_flex,
};
use gpui_kit::{
    App, Entity, IntoElement as _, ParentElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResourceKind {
    Mods,
    Packs,
    Shaders,
    Worlds,
}

impl ResourceKind {
    fn title(self) -> &'static str {
        match self {
            Self::Mods => "Mods",
            Self::Packs => "Resource packs",
            Self::Shaders => "Shader packs",
            Self::Worlds => "Worlds",
        }
    }
    fn icon(self) -> PhaIcon {
        match self {
            Self::Mods => PhaIcon::Puzzle,
            Self::Packs => PhaIcon::Palette,
            Self::Shaders => PhaIcon::Sparkles,
            Self::Worlds => PhaIcon::Globe,
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Mods => "Manage the isolated mods folder for this instance.",
            Self::Packs => "Texture packs for the instance you opened.",
            Self::Shaders => "Shader packs for the instance you opened.",
            Self::Worlds => "Saved worlds for the instance you opened.",
        }
    }
    fn mode(self) -> Option<ResourceMode> {
        match self {
            Self::Mods => Some(ResourceMode::Mods),
            Self::Packs => Some(ResourceMode::Packs),
            Self::Shaders => Some(ResourceMode::Shaders),
            Self::Worlds => None,
        }
    }
}

struct ResourceRow {
    id: String,
    name: String,
    detail: String,
    metadata: String,
    enabled: Option<bool>,
}

pub(crate) fn render(
    reference: &InstanceRef,
    kind: ResourceKind,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    let Some(instance) = app.read(cx).instances.read(cx).find(reference).cloned() else {
        return missing_resource("instance", app).into_any_element();
    };
    let query = window.use_keyed_state(instance_key(reference, "mod-query"), cx, |window, cx| {
        InputState::new(window, cx).placeholder("Search mods…")
    });
    let loader = window.use_keyed_state(instance_key(reference, "mod-loader"), cx, |window, cx| {
        SelectState::new(
            vec!["All loaders", "Fabric", "Forge", "NeoForge"],
            Some(IndexPath::new(0)),
            window,
            cx,
        )
    });
    let search = query.read(cx).value().trim().to_lowercase();
    let selected_loader = loader
        .read(cx)
        .selected_value()
        .copied()
        .unwrap_or("All loaders");
    let rows: Vec<ResourceRow> = match kind {
        ResourceKind::Mods => instance
            .mods
            .iter()
            .filter(|item| {
                (search.is_empty()
                    || item.display_name().to_lowercase().contains(&search)
                    || item.file_name.to_lowercase().contains(&search))
                    && (selected_loader == "All loaders"
                        || item.loader.map(Loader::label) == Some(selected_loader))
            })
            .map(|item| ResourceRow {
                id: item.id.clone(),
                name: item.display_name().into(),
                detail: item
                    .version
                    .as_ref()
                    .map(|version| format!("{version} · {}", item.file_name))
                    .unwrap_or_else(|| {
                        if item.name.is_some() {
                            item.file_name.clone()
                        } else {
                            String::new()
                        }
                    }),
                metadata: String::new(),
                enabled: Some(item.enabled),
            })
            .collect(),
        ResourceKind::Packs => instance
            .resource_packs
            .iter()
            .map(|item| ResourceRow {
                id: item.id.clone(),
                name: item.name.clone(),
                detail: item.description.clone(),
                metadata: format!("{} · {}", item.author, item.size),
                enabled: Some(item.enabled),
            })
            .collect(),
        ResourceKind::Shaders => instance
            .shader_packs
            .iter()
            .map(|item| ResourceRow {
                id: item.id.clone(),
                name: item.name.clone(),
                detail: format!("{} · {}", item.author, item.version),
                metadata: item.gpu.clone(),
                enabled: Some(item.enabled),
            })
            .collect(),
        ResourceKind::Worlds => instance
            .worlds
            .iter()
            .map(|item| ResourceRow {
                id: item.id.clone(),
                name: item.name.clone(),
                detail: format!(
                    "seed {} · {} · {} player{}",
                    item.seed,
                    item.last_played,
                    item.players,
                    if item.players == 1 { "" } else { "s" }
                ),
                metadata: format!("MC {}", item.version),
                enabled: None,
            })
            .collect(),
    };
    let count = match kind {
        ResourceKind::Mods => instance.mods.len(),
        ResourceKind::Packs => instance.resource_packs.len(),
        ResourceKind::Shaders => instance.shader_packs.len(),
        ResourceKind::Worlds => instance.worlds.len(),
    };
    let title =
        heading(kind.icon(), kind.title(), cx).child(badge(if kind == ResourceKind::Mods {
            format!("{}/{} enabled", instance.enabled_mods(), count)
        } else {
            count.to_string()
        }));
    let mut actions = h_flex().flex_shrink_0().gap_2();
    if let Some(mode) = kind.mode() {
        let target = reference.clone();
        let app = app.clone();
        actions = actions.child(
            Button::new("resource-add")
                .icon(PhaIcon::Plus)
                .label(if kind == ResourceKind::Mods {
                    "Add mods…"
                } else {
                    "Import…"
                })
                .on_click(move |_, window, cx| {
                    add_resources_dialog::open_for_instance(
                        window,
                        cx,
                        app.clone(),
                        target.clone(),
                        mode,
                    )
                }),
        );
    }
    let title = instance_header(
        &instance,
        title,
        kind.description(),
        actions,
        app.clone(),
        cx,
    );
    let mut content = v_flex().gap_4();
    if kind == ResourceKind::Mods {
        content = content.child(
            h_flex()
                .gap_3()
                .child(
                    div().flex_1().min_w_0().child(
                        Input::new(&query)
                            .prefix(Icon::new(PhaIcon::Search).size_4())
                            .cleanable(true),
                    ),
                )
                .child(
                    control_slot(
                        Select::new(&loader)
                            .w_full()
                            .accessibility_label("Filter mods by loader"),
                    )
                    .w_40(),
                ),
        );
    }
    if rows.is_empty() {
        let (title, description) = match kind {
            ResourceKind::Mods if count > 0 => {
                ("No mods match", "Try a different search or loader filter.")
            }
            ResourceKind::Mods => (
                "No mods installed",
                "Add .jar files to customize this instance.",
            ),
            ResourceKind::Packs => (
                "No resource packs",
                "Texture packs change how the game looks without touching the code.",
            ),
            ResourceKind::Shaders => (
                "No shader packs",
                "Shaders need Iris or OptiFine in the instance’s mod loader.",
            ),
            ResourceKind::Worlds => (
                "No worlds yet",
                "Saved games appear here once you create them.",
            ),
        };
        content = content.child(empty(kind.icon(), title, description, cx));
    } else {
        content = content.child(
            v_flex().gap_2().children(
                rows.into_iter()
                    .map(|row| resource_row(reference, kind, row, app.clone(), cx)),
            ),
        );
    }
    page_shell(Some(title), content, cx).into_any_element()
}

fn set_enabled(
    reference: &InstanceRef,
    kind: ResourceKind,
    id: &str,
    enabled: bool,
    app: &Entity<AppState>,
    cx: &mut App,
) {
    app.read(cx).instances.clone().update(cx, |store, cx| {
        let changed = match kind {
            ResourceKind::Mods => store.set_mod_enabled(reference, id, enabled),
            ResourceKind::Packs => store.set_resource_pack_enabled(reference, id, enabled),
            ResourceKind::Shaders => store.set_shader_pack_enabled(reference, id, enabled),
            ResourceKind::Worlds => false,
        };
        if changed {
            cx.notify();
        }
    });
}

fn resource_row(
    reference: &InstanceRef,
    kind: ResourceKind,
    row: ResourceRow,
    app: Entity<AppState>,
    cx: &App,
) -> gpui_kit::AnyElement {
    let id = instance_key(reference, &row.id);
    let mut line = h_flex()
        .gap_3()
        .p_3()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().group_box)
        .child(icon_tile(
            if kind == ResourceKind::Mods {
                PhaIcon::Box
            } else {
                kind.icon()
            },
            cx,
        ))
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_1()
                .child(
                    div()
                        .truncate()
                        .text_base()
                        .font_medium()
                        .text_color(if row.enabled == Some(false) {
                            cx.theme().muted_foreground
                        } else {
                            cx.theme().foreground
                        })
                        .child(row.name.clone()),
                )
                .when(!row.detail.is_empty(), |element| {
                    element.child(muted(row.detail.clone(), cx).text_xs().truncate())
                }),
        );
    if !row.metadata.is_empty() {
        line = line.child(badge(row.metadata.clone()));
    }
    if let Some(enabled) = row.enabled {
        let target = reference.clone();
        let resource = row.id.clone();
        let app = app.clone();
        line = line.child(
            Switch::new(format!("{id}-enabled"))
                .checked(enabled)
                .accessibility_label(format!("Enable {}", row.name))
                .on_click(move |checked, _, cx| {
                    set_enabled(&target, kind, &resource, *checked, &app, cx)
                }),
        );
    } else {
        let target = reference.clone();
        let name = row.name.clone();
        let app = app.clone();
        line = line.child(
            Button::new(format!("{id}-play"))
                .icon(PhaIcon::Play)
                .label("Play")
                .on_click(move |_, window, cx| {
                    app.read(cx).instances.clone().update(cx, |store, cx| {
                        store.set_launch_override(
                            &target,
                            LaunchField::QuickPlayMode,
                            LaunchValue::QuickPlay(QuickPlayMode::World),
                        );
                        store.set_launch_override(
                            &target,
                            LaunchField::QuickPlayTarget,
                            LaunchValue::Text(name.clone()),
                        );
                        cx.notify();
                    });
                    instance_actions::start(&target, &app, window, cx);
                }),
        );
    }
    let target = reference.clone();
    line.child(
        icon_button(
            format!("{id}-menu"),
            PhaIcon::Ellipsis,
            format!("Actions for {}", row.name),
        )
        .dropdown_menu(move |mut menu, _, _| {
            if let Some(enabled) = row.enabled {
                let target = target.clone();
                let resource = row.id.clone();
                let app = app.clone();
                menu = menu
                    .item(
                        PopupMenuItem::new(if enabled { "Disable" } else { "Enable" }).on_click(
                            move |_, _, cx| {
                                set_enabled(&target, kind, &resource, !enabled, &app, cx)
                            },
                        ),
                    )
                    .separator();
            }
            let target = target.clone();
            let resource = row.id.clone();
            let name = row.name.clone();
            let app = app.clone();
            menu.item(
                PopupMenuItem::new(if kind == ResourceKind::Worlds {
                    "Delete world…"
                } else {
                    "Delete…"
                })
                .icon(PhaIcon::Trash2)
                .on_click(move |_, window, cx| {
                    let target = target.clone();
                    let resource = resource.clone();
                    let app = app.clone();
                    confirm_dialog::open(
                        window,
                        cx,
                        format!("Delete “{name}”?"),
                        if kind == ResourceKind::Worlds {
                            "The world and its saved progress are removed from this instance."
                        } else {
                            "The resource is removed from this instance."
                        },
                        move |_, _, cx| {
                            app.read(cx).instances.clone().update(cx, |store, cx| {
                                let changed = match kind {
                                    ResourceKind::Mods => store.remove_mod(&target, &resource),
                                    ResourceKind::Packs => {
                                        store.remove_resource_pack(&target, &resource)
                                    }
                                    ResourceKind::Shaders => {
                                        store.remove_shader_pack(&target, &resource)
                                    }
                                    ResourceKind::Worlds => store.remove_world(&target, &resource),
                                };
                                if changed {
                                    cx.notify();
                                }
                            });
                            true
                        },
                    );
                }),
            )
        }),
    )
    .into_any_element()
}
