//! Resource-pack management page for a selected instance.

use super::{back_button, instance_exists, missing_resource, page_shell};
use crate::{route::InstanceRef, state::AppState};
use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
use gpui_component::button::ButtonVariants as _;
use gpui_component::{ActiveTheme as _, Sizable as _, StyledExt, h_flex, v_flex};
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
            .child(
                v_flex()
                    .gap_2()
                    .children(instance.resource_packs.iter().map(|item| {
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
                                    .child(div().font_medium().child(item.name.clone()))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!(
                                                "{} · {}",
                                                item.author, item.description
                                            )),
                                    ),
                            )
                            .child(div().text_sm().child(if item.enabled {
                                "Enabled"
                            } else {
                                "Disabled"
                            }))
                    })),
            );
    let content = content.child(
        gpui_component::button::Button::new("add-resource")
            .primary()
            .label("Import resource packs")
            .on_click({
                let app = app.clone();
                move |_, window, cx| {
                    crate::components::add_resources_dialog::open(
                        window,
                        cx,
                        app.clone(),
                        crate::components::add_resources_dialog::ResourceMode::Packs,
                    )
                }
            }),
    );
    page_shell(
        "Resource packs",
        format!("Visual packs installed for {}.", instance.name),
        content,
        cx,
    )
    .into_any_element()
}
