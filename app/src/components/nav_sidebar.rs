//! Primary navigation sidebar and its instance navigation entries.

use gpui::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, Collapsible, Icon, IconName, StyledExt,
    button::ButtonVariants as _,
    sidebar::{Sidebar, SidebarFooter, SidebarItem},
    v_flex,
};

use crate::{
    components::sidebar_instance_item::SidebarInstanceItem,
    route::Route,
    state::{AppState, InstanceSummary},
};

fn activate(
    app: Entity<AppState>,
    route: Route,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) {
    move |_, _, cx| app.update(cx, |state, cx| state.push(route.clone(), cx))
}

#[derive(Clone)]
struct InstanceMenu {
    app: Entity<AppState>,
    current: Route,
    favorites: Vec<InstanceSummary>,
    local: Vec<InstanceSummary>,
    aphanite: Vec<InstanceSummary>,
    running: Vec<crate::route::InstanceRef>,
}

impl Collapsible for InstanceMenu {
    fn is_collapsed(&self) -> bool {
        false
    }

    fn collapsed(self, _: bool) -> Self {
        self
    }
}

fn instance_section(
    id: &gpui::ElementId,
    label: &str,
    icon: IconName,
    instances: &[InstanceSummary],
    current: &Route,
    running: &[crate::route::InstanceRef],
    app: Entity<AppState>,
    open: Entity<bool>,
    route: Option<Route>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let is_open = *open.read(cx);
    v_flex()
        .gap_1()
        .child(
            div()
                .id(format!("{id}-{label}"))
                .h_7()
                .px_2()
                .flex()
                .items_center()
                .gap_2()
                .rounded(cx.theme().radius)
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(Icon::new(icon))
                .child(div().flex_1().child(label.to_string()))
                .child(div().child(instances.len().to_string()))
                .child(Icon::new(if is_open {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                }))
                .on_click({
                    let app = app.clone();
                    move |_, _, cx| {
                        open.update(cx, |is_open, cx| {
                            *is_open = !*is_open;
                            cx.notify();
                        });
                        if let Some(route) = &route {
                            app.update(cx, |state, cx| state.push(route.clone(), cx));
                        }
                    }
                }),
        )
        .when(is_open, |section| {
            section.child(
                v_flex()
                    .border_l_1()
                    .border_color(cx.theme().sidebar_border)
                    .ml_3p5()
                    .pl_2p5()
                    .py_0p5()
                    .gap_1()
                    .children(instances.iter().cloned().map(|instance| {
                        let reference = instance.reference();
                        SidebarInstanceItem::new(
                            instance,
                            current == &Route::InstanceDetail(reference.clone()),
                            running.contains(&reference),
                            app.clone(),
                        )
                        .render(window, cx)
                        .into_any_element()
                    })),
            )
        })
}

impl SidebarItem for InstanceMenu {
    fn render(
        self,
        id: impl Into<gpui::ElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let id = id.into();
        let favorites_open =
            window.use_keyed_state(format!("{id}-favorites-open"), cx, |_, _| true);
        let instances_open =
            window.use_keyed_state(format!("{id}-instances-open"), cx, |_, _| false);
        let aphanite_open = window.use_keyed_state(format!("{id}-aphanite-open"), cx, |_, _| false);
        v_flex()
            .id(id.clone())
            .gap_2()
            .child(
                div()
                    .id("sidebar-play")
                    .h_7()
                    .px_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded(cx.theme().radius)
                    .text_sm()
                    .when(matches!(self.current, Route::Play), |item| {
                        item.bg(cx.theme().sidebar_accent)
                            .text_color(cx.theme().sidebar_accent_foreground)
                            .font_medium()
                    })
                    .child(Icon::new(IconName::Play))
                    .child("Play")
                    .on_click(activate(self.app.clone(), Route::Play)),
            )
            .child(instance_section(
                &id,
                "Favorites",
                IconName::Star,
                &self.favorites,
                &self.current,
                &self.running,
                self.app.clone(),
                favorites_open,
                None,
                window,
                cx,
            ))
            .child(instance_section(
                &id,
                "Instances",
                IconName::Folder,
                &self.local,
                &self.current,
                &self.running,
                self.app.clone(),
                instances_open,
                Some(Route::Instances),
                window,
                cx,
            ))
            .child(instance_section(
                &id,
                "Aphanite",
                IconName::Folder,
                &self.aphanite,
                &self.current,
                &self.running,
                self.app.clone(),
                aphanite_open,
                Some(Route::Aphanite),
                window,
                cx,
            ))
    }
}

pub fn render(app: Entity<AppState>, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let instances = state.instances.read(cx);
    let sessions = state.sessions.read(cx);
    let account = state
        .accounts
        .read(cx)
        .active()
        .map(|item| item.username.clone());
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

    v_flex()
        .h_full()
        .w_full()
        .flex_shrink_0()
        .border_r_1()
        .border_color(cx.theme().sidebar_border)
        .child(
            Sidebar::new("launcher-sidebar")
                .border_0()
                .w_full()
                .child(menu)
                .footer(
                    SidebarFooter::new().child(
                        v_flex()
                            .gap_2()
                            .child(
                                gpui_component::button::Button::new("sidebar-account")
                                    .ghost()
                                    .label(account.unwrap_or_else(|| "Offline".into()))
                                    .on_click({
                                        let app = app.clone();
                                        move |_, _, cx| {
                                            app.update(cx, |state, cx| {
                                                state.push(Route::Accounts, cx)
                                            })
                                        }
                                    }),
                            )
                            .child(
                                gpui_component::button::Button::new("sidebar-settings")
                                    .ghost()
                                    .icon(IconName::Settings)
                                    .label("Settings")
                                    .on_click(move |_, _, cx| {
                                        app.update(cx, |state, cx| state.push(Route::Settings, cx))
                                    }),
                            ),
                    ),
                ),
        )
}
