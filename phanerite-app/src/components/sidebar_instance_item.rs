//! Sidebar leaf item for a Minecraft instance.

use gpui::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _,
};
use gpui_component::{ActiveTheme as _, StyledExt as _, h_flex};

use crate::{
    route::Route,
    state::{AppState, InstanceSummary},
};

pub struct SidebarInstanceItem {
    instance: InstanceSummary,
    active: bool,
    running: bool,
    app: Entity<AppState>,
}

impl SidebarInstanceItem {
    pub fn new(
        instance: InstanceSummary,
        active: bool,
        running: bool,
        app: Entity<AppState>,
    ) -> Self {
        Self {
            instance,
            active,
            running,
            app,
        }
    }
}

impl RenderOnce for SidebarInstanceItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reference = self.instance.reference();
        let hovered = window.use_keyed_state(
            format!("sidebar-instance-hovered-{}", self.instance.id),
            cx,
            |_, _| false,
        );
        let is_hovered = *hovered.read(cx);
        h_flex()
            .id(format!("sidebar-instance-{}", self.instance.id))
            .w_full()
            .h_7()
            .items_center()
            .gap_2()
            .rounded(cx.theme().radius)
            .px_2()
            .when(self.active, |item| {
                item.bg(cx.theme().sidebar_accent)
                    .text_color(cx.theme().sidebar_accent_foreground)
                    .font_medium()
            })
            .when(!self.active && is_hovered, |item| {
                item.bg(cx.theme().sidebar_accent.opacity(0.8))
                    .text_color(cx.theme().sidebar_accent_foreground)
            })
            .on_hover(move |is_hovered, _, cx| {
                hovered.update(cx, |hovered, cx| {
                    if *hovered != *is_hovered {
                        *hovered = *is_hovered;
                        cx.notify();
                    }
                })
            })
            .child(crate::components::instance_icon::render_sized(
                &self.instance,
                gpui::px(16.),
                cx,
            ))
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .truncate()
                    .text_sm()
                    .child(self.instance.name),
            )
            .when(self.running, |item| {
                item.child(div().size_2().rounded_full().bg(cx.theme().primary))
            })
            .on_click(move |_, _, cx| {
                self.app.update(cx, |state, cx| {
                    state.push(Route::InstanceDetail(reference.clone()), cx)
                })
            })
    }
}
