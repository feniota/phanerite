//! Mods page for browsing and managing an instance's installed mods.

use super::{back_button, instance_exists, missing_resource, page_shell, page_title};
use crate::{route::InstanceRef, state::AppState};
use gpui::{App, Entity, IntoElement as _, ParentElement as _, Styled as _, Window, div};
use gpui_component::button::ButtonVariants as _;
use gpui_component::{ActiveTheme as _, StyledExt as _, h_flex, v_flex};

pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    _window: &mut Window,
    cx: &App,
) -> gpui::AnyElement {
    if !instance_exists(reference, &app, cx) {
        return missing_resource("instance", app).into_any_element();
    }
    let instance = app
        .read(cx)
        .instances
        .read(cx)
        .find(reference)
        .unwrap()
        .clone();
    let content =
        v_flex()
            .gap_4()
            .child(back_button(app.clone()))
            .child(v_flex().gap_2().children(instance.mods.iter().map(|item| {
                h_flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().accordion)
                    .child(
                        v_flex()
                            .gap_1()
                            .child(div().font_medium().child(item.display_name().to_string()))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        item.version
                                            .clone()
                                            .unwrap_or_else(|| item.file_name.clone()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(if item.enabled { "Enabled" } else { "Disabled" }),
                    )
            })));
    let content = content.child(
        gpui_component::button::Button::new("add-resource")
            .primary()
            .label("Add mods")
            .on_click({
                let app = app.clone();
                move |_, window, cx| {
                    crate::components::add_resources_dialog::open(
                        window,
                        cx,
                        app.clone(),
                        crate::components::add_resources_dialog::ResourceMode::Mods,
                    )
                }
            }),
    );
    page_shell(
        Some(page_title(
            "Mods",
            format!("Manage the isolated mods folder for {}.", instance.name),
            cx,
        )),
        content,
        cx,
    )
    .into_any_element()
}
