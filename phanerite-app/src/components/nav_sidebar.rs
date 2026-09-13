//! Primary navigation, flat instance groups, and independent account/settings controls.

use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    menu::{DropdownMenu as _, PopupMenuItem},
    scroll::ScrollableElement as _,
    v_flex,
};
use gpui_kit::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Styled as _,
    Window, div, prelude::FluentBuilder as _, rems,
};

use crate::{
    assets::PhaIcon,
    components::sidebar_instance_item::SidebarInstanceItem,
    route::Route,
    state::{AppState, InstanceSummary},
};

fn activate(
    app: Entity<AppState>,
    route: Route,
) -> impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) {
    move |_, _, cx| app.update(cx, |state, cx| state.push(route.clone(), cx))
}

fn navigation_button(id: &'static str, active: bool, cx: &App) -> Button {
    Button::new(id)
        .ghost()
        .w_full()
        .h_8()
        .px_2p5()
        .selected(active)
        .text_color(if active {
            cx.theme().sidebar_accent_foreground
        } else {
            cx.theme().muted_foreground
        })
        .when(active, |button| button.bg(cx.theme().sidebar_accent))
}

#[derive(IntoElement)]
struct InstanceMenu {
    app: Entity<AppState>,
    current: Route,
    favorites: Vec<InstanceSummary>,
    local: Vec<InstanceSummary>,
    aphanite: Vec<InstanceSummary>,
    running: Vec<crate::route::InstanceRef>,
}

#[derive(Clone, Copy)]
enum Section {
    Local,
    Aphanite,
}

impl InstanceMenu {
    fn instance_rows(&self, instances: &[InstanceSummary]) -> impl IntoElement {
        v_flex()
            .gap_0p5()
            .children(instances.iter().cloned().map(|instance| {
                let reference = instance.reference();
                SidebarInstanceItem::new(
                    instance,
                    matches!(&self.current,
                        Route::InstanceDetail(active) | Route::Mods(active) | Route::Packs(active)
                        | Route::Shaders(active) | Route::Worlds(active) | Route::Logs(active)
                        | Route::LaunchSettings(active) if active == &reference),
                    self.running.contains(&reference),
                    self.app.clone(),
                )
            }))
    }

    fn section(&self, section: Section, open: Entity<bool>, cx: &App) -> impl IntoElement {
        let (id, label, icon, instances, route, toggle_id, toggle_label) = match section {
            Section::Local => (
                "sidebar-instances",
                "Instances",
                PhaIcon::Layers,
                &self.local,
                Route::Instances,
                "sidebar-instances-toggle",
                "local instances",
            ),
            Section::Aphanite => (
                "sidebar-aphanite",
                "Aphanite",
                PhaIcon::Flame,
                &self.aphanite,
                Route::Aphanite,
                "sidebar-aphanite-toggle",
                "Aphanite instances",
            ),
        };
        let is_open = *open.read(cx);
        let active = self.current == route;
        let disclosure = format!(
            "{} {toggle_label}",
            if is_open { "Collapse" } else { "Expand" }
        );

        v_flex()
            .mt_3()
            .gap_0p5()
            .child(
                h_flex()
                    .rounded(cx.theme().radius)
                    .when(active, |row| row.bg(cx.theme().sidebar_accent))
                    .child(
                        navigation_button(id, active, cx)
                            .debug_selector(move || id.into())
                            .w_auto()
                            .flex_1()
                            .min_w_0()
                            .rounded_r(rems(0.))
                            .accessibility_label(label)
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_2p5()
                                    .child(
                                        Icon::new(icon)
                                            .size_4()
                                            .when(matches!(section, Section::Aphanite), |icon| {
                                                icon.text_color(crate::theme::flame())
                                            }),
                                    )
                                    .child(div().flex_1().text_left().text_sm().child(label))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(instances.len().to_string()),
                                    ),
                            )
                            .on_click(activate(self.app.clone(), route)),
                    )
                    .child(
                        Button::new(toggle_id)
                            .debug_selector(move || toggle_id.into())
                            .ghost()
                            .size_8()
                            .rounded_l(rems(0.))
                            .text_color(cx.theme().muted_foreground)
                            .icon(
                                Icon::new(if is_open {
                                    PhaIcon::ChevronDown
                                } else {
                                    PhaIcon::ChevronRight
                                })
                                .size_3p5(),
                            )
                            .tooltip(disclosure.clone())
                            .accessibility_label(disclosure)
                            .on_click(move |_, _, cx| {
                                open.update(cx, |is_open, cx| {
                                    *is_open = !*is_open;
                                    cx.notify();
                                });
                            }),
                    ),
            )
            .when(is_open, |section| {
                section.child(self.instance_rows(instances))
            })
    }
}

impl RenderOnce for InstanceMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let instances_open = window.use_keyed_state("sidebar-instances-open", cx, |_, _| true);
        let aphanite_open = window.use_keyed_state("sidebar-aphanite-open", cx, |_, _| true);
        v_flex()
            .px_2()
            .py_3()
            .child(
                navigation_button("sidebar-play", matches!(self.current, Route::Play), cx)
                    .debug_selector(|| "sidebar-play".into())
                    .accessibility_label("Quick Play")
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2p5()
                            .child(Icon::new(PhaIcon::Play).size_4())
                            .child(div().flex_1().text_left().text_sm().child("Quick Play"))
                            .when(!self.running.is_empty(), |row| {
                                row.child(
                                    h_flex()
                                        .gap_1()
                                        .text_xs()
                                        .text_color(cx.theme().primary)
                                        .child(
                                            div().size_1p5().rounded_full().bg(cx.theme().primary),
                                        )
                                        .child(self.running.len().to_string()),
                                )
                            }),
                    )
                    .on_click(activate(self.app.clone(), Route::Play)),
            )
            .child(
                v_flex()
                    .mt_3()
                    .gap_0p5()
                    .child(
                        h_flex()
                            .h_7()
                            .px_2p5()
                            .gap_2()
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().muted_foreground)
                            .child(Icon::new(PhaIcon::Star).size_3p5())
                            .child(div().flex_1().child("FAVORITES"))
                            .child(self.favorites.len().to_string()),
                    )
                    .when(self.favorites.is_empty(), |section| {
                        section.child(
                            div()
                                .px_2p5()
                                .py_1()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("No favorites yet"),
                        )
                    })
                    .child(self.instance_rows(&self.favorites)),
            )
            .child(self.section(Section::Local, instances_open, cx))
            .child(self.section(Section::Aphanite, aphanite_open, cx))
    }
}

pub fn render(app: Entity<AppState>, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let instances = state.instances.read(cx);
    let sessions = state.sessions.read(cx);
    let account = state.accounts.read(cx).active();
    let menu = InstanceMenu {
        app: app.clone(),
        current: state.route().clone(),
        favorites: instances.favorites().cloned().collect(),
        local: instances.local().cloned().collect(),
        aphanite: instances.aphanite_unfavorited().cloned().collect(),
        running: sessions
            .running()
            .map(|session| session.instance.clone())
            .collect(),
    };
    let account_app = app.clone();

    v_flex()
        .size_full()
        .bg(cx.theme().sidebar)
        .text_color(cx.theme().sidebar_foreground)
        .child(div().flex_1().min_h_0().overflow_y_scrollbar().child(menu))
        .child(
            v_flex()
                .flex_shrink_0()
                .px_2()
                .pb_2()
                .gap_2()
                .child(
                    Button::new("sidebar-account")
                        .debug_selector(|| "sidebar-account".into())
                        .ghost()
                        .w_full()
                        .h(rems(3.5))
                        .px_2p5()
                        .py_2()
                        .bg(crate::theme::sidebar_card())
                        .border_1()
                        .border_color(cx.theme().border)
                        .accessibility_label("Switch account")
                        .child(
                            h_flex()
                                .w_full()
                                .gap_2p5()
                                .child(
                                    super::minecraft_avatar::render(
                                        account
                                            .as_ref()
                                            .and_then(|account| account.active_profile()),
                                        cx,
                                    )
                                    .size_7(),
                                )
                                .child(
                                    v_flex()
                                        .min_w_0()
                                        .flex_1()
                                        .text_left()
                                        .child(
                                            div().text_sm().font_medium().truncate().child(
                                                account
                                                    .as_ref()
                                                    .map(|account| account.username.clone())
                                                    .unwrap_or_else(|| "Add an account".into()),
                                            ),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .truncate()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(
                                                    account
                                                        .as_ref()
                                                        .map(|account| {
                                                            format!(
                                                                "{} account",
                                                                account.account_type.label()
                                                            )
                                                        })
                                                        .unwrap_or_else(|| {
                                                            "Choose a player profile".into()
                                                        }),
                                                ),
                                        ),
                                )
                                .child(
                                    Icon::new(PhaIcon::ChevronDown)
                                        .size_3p5()
                                        .text_color(cx.theme().muted_foreground),
                                ),
                        )
                        .dropdown_menu(move |mut menu, _, cx| {
                            let accounts = account_app.read(cx).accounts.clone();
                            let active = accounts.read(cx).active_id();
                            menu = menu.label("Switch account");
                            for account in accounts.read(cx).all() {
                                let accounts = accounts.clone();
                                menu = menu.item(
                                    PopupMenuItem::new(account.username)
                                        .checked(active.as_ref() == Some(&account.id))
                                        .on_click(move |_, _, cx| {
                                            accounts.update(cx, |store, cx| {
                                                if store.set_active(&account.id) {
                                                    cx.notify();
                                                }
                                            });
                                        }),
                                );
                            }
                            menu.separator().item(
                                PopupMenuItem::new("Manage accounts")
                                    .icon(PhaIcon::Users)
                                    .on_click(activate(account_app.clone(), Route::Accounts)),
                            )
                        }),
                )
                .child(
                    navigation_button(
                        "sidebar-settings",
                        matches!(state.route(), Route::Settings),
                        cx,
                    )
                    .debug_selector(|| "sidebar-settings".into())
                    .accessibility_label("Settings")
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2p5()
                            .child(Icon::new(PhaIcon::Settings).size_4())
                            .child(div().text_sm().child("Settings")),
                    )
                    .on_click(activate(app, Route::Settings)),
                ),
        )
}
