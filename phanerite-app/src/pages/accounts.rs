//! Accounts page for managing authenticated and offline player profiles.

use super::{page_shell, page_title};
use crate::state::AppState;
use gpui_kit::component::{
    ActiveTheme as _, IconName, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};
pub fn render(app: Entity<AppState>, _window: &mut Window, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let accounts_entity = state.accounts.clone();
    let accounts = accounts_entity.read(cx);
    let active = accounts.active_id();
    let subtitle = "Microsoft, Aphanite, and custom Yggdrasil accounts support online play. Offline accounts skip login entirely.";
    let content = v_flex()
        .gap_4()
        .child(
            div()
                .p_4()
                .rounded(cx.theme().radius)
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().accordion)
                .child(subtitle),
        )
        .child(
            v_flex()
                .gap_2()
                .children(accounts.all().iter().map(|account| {
                    let id = account.id.clone();
                    let accounts_entity = accounts_entity.clone();
                    h_flex()
                        .items_center()
                        .justify_between()
                        .p_3()
                        .rounded(cx.theme().radius)
                        .border_1()
                        .border_color(if active.as_deref() == Some(&id) {
                            cx.theme().primary
                        } else {
                            cx.theme().border
                        })
                        .bg(cx.theme().accordion)
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().font_medium().child(account.username.clone()))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!(
                                            "{} · {}",
                                            account.account_type.label(),
                                            account.detail()
                                        )),
                                ),
                        )
                        .child(
                            Button::new(format!("account-{}", id))
                                .label(if active.as_deref() == Some(&id) {
                                    "Active"
                                } else {
                                    "Use account"
                                })
                                .on_click(move |_, _, cx| {
                                    accounts_entity.update(cx, |store, cx| {
                                        if store.set_active(&id) {
                                            cx.notify()
                                        }
                                    });
                                }),
                        )
                })),
        )
        .child(
            Button::new("account-add")
                .primary()
                .icon(IconName::Plus)
                .label("Add account")
                .on_click({
                    let app = app.clone();
                    move |_, window, cx| {
                        crate::components::account_add_dialog::open(window, cx, app.clone())
                    }
                }),
        );
    page_shell(
        Some(page_title(
            "Accounts",
            "One active account per launch; offline accounts work for single-player only.",
            cx,
        )),
        content,
        cx,
    )
}
