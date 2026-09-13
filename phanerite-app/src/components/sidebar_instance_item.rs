//! Sidebar leaf item for a Minecraft instance.

use gpui_kit::component::{
    ActiveTheme as _, Selectable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_kit::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, Styled as _,
    Window, div, prelude::FluentBuilder as _,
};

use crate::{
    route::Route,
    state::{AppState, InstanceSummary},
};

#[derive(IntoElement)]
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
        let selector = format!("sidebar-instance-{}", self.instance.id);
        Button::new(format!(
            "sidebar-instance-{}:{}",
            reference.storage.root_dir.display(),
            self.instance.id
        ))
        .debug_selector(move || selector.clone())
        .ghost()
        .w_full()
        .h_7()
        .px_2p5()
        .selected(self.active)
        .accessibility_label(self.instance.name.clone())
        .text_color(cx.theme().muted_foreground)
        .when(self.active, |item| {
            item.bg(cx.theme().sidebar_accent)
                .text_color(cx.theme().sidebar_accent_foreground)
                .font_medium()
        })
        .child(
            h_flex()
                .w_full()
                .gap_2()
                .child(crate::components::instance_icon::render_sized(
                    &self.instance,
                    window.rem_size(),
                    cx,
                ))
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .truncate()
                        .text_left()
                        .text_sm()
                        .child(self.instance.name),
                )
                .when(self.running, |row| {
                    row.child(div().size_1p5().rounded_full().bg(cx.theme().primary))
                }),
        )
        .on_click(move |_, _, cx| {
            self.app.update(cx, |state, cx| {
                state.push(Route::InstanceDetail(reference.clone()), cx)
            })
        })
    }
}
