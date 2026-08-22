//! World management page for a selected instance.

use super::{back_button, instance_exists, missing_resource, page_shell, page_title};
use crate::{route::InstanceRef, state::AppState};
use gpui::{App, Entity, IntoElement as _, ParentElement as _, Styled as _, Window, div};
use gpui_component::{ActiveTheme as _, StyledExt as _, h_flex, v_flex};
pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    _: &mut Window,
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
    let content = v_flex()
        .gap_4()
        .child(back_button(app))
        .child(
            v_flex()
                .gap_2()
                .children(instance.worlds.iter().map(|item| {
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
                                            "{} · last played {}",
                                            item.version, item.last_played
                                        )),
                                ),
                        )
                        .child(div().text_sm().child(format!("{} players", item.players)))
                })),
        );
    page_shell(
        Some(page_title(
            "Worlds",
            format!("Saved worlds for {}.", instance.name),
            cx,
        )),
        content,
        cx,
    )
    .into_any_element()
}
