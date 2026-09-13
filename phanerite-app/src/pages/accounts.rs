//! Account selection and player profile preview.
use super::{page_shell, widgets::*};
use crate::{
    assets::PhaIcon,
    components::{account_add_dialog, minecraft_avatar},
    state::{AccountType, AppState},
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::DialogButtonProps,
    h_flex, v_flex,
};
use gpui_kit::{
    App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _,
};

pub fn render(app: Entity<AppState>, window: &mut Window, cx: &App) -> impl IntoElement {
    let compact = window.viewport_size().width < window.rem_size() * 70.;
    let accounts = app.read(cx).accounts.read(cx).all();
    let active = app.read(cx).accounts.read(cx).active();
    let active_id = active.as_ref().map(|account| account.id.as_str());
    let add_app = app.clone();
    let title = header(
        heading(PhaIcon::Users, "Accounts", cx),
        "One active account per launch; offline accounts work for single-player only.",
        Button::new("account-add")
            .icon(PhaIcon::Plus)
            .label("Add account…")
            .on_click(move |_, window, cx| account_add_dialog::open(window, cx, add_app.clone())),
        cx,
    );
    let mut list = v_flex().flex_1().min_w_0().gap_4()
        .child(panel(cx).p_3().gap_1().child(h_flex().gap_2().child(Icon::new(PhaIcon::Info).size_4()).child(div().text_sm().font_medium().child("Account types")))
            .child(muted("Microsoft, Aphanite, and custom Yggdrasil accounts support online play. Offline accounts skip login entirely.", cx).text_xs()));
    if accounts.is_empty() {
        list = list.child(empty(
            PhaIcon::Users,
            "No accounts yet",
            "Add an account to choose a player profile.",
            cx,
        ));
    }
    for account in &accounts {
        let selected = active_id == Some(account.id.as_str());
        let id = account.id.clone();
        let app_select = app.clone();
        let provider_icon = match account.account_type {
            AccountType::Microsoft => PhaIcon::KeyRound,
            AccountType::Aphanite => PhaIcon::Flame,
            AccountType::Yggdrasil => PhaIcon::Server,
            AccountType::Offline => PhaIcon::Monitor,
        };
        let label = h_flex()
            .flex_wrap()
            .gap_2()
            .min_w_0()
            .child(
                div()
                    .text_sm()
                    .font_medium()
                    .truncate()
                    .child(account.username.clone()),
            )
            .child(
                h_flex()
                    .gap_1()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(Icon::new(provider_icon).size_3())
                    .child(account.account_type.label()),
            )
            .children(selected.then(|| badge("Active")));
        let mut row = h_flex()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(if selected {
                cx.theme().primary.opacity(0.4)
            } else {
                cx.theme().border
            })
            .bg(if selected {
                cx.theme().accent.opacity(0.4)
            } else {
                cx.theme().group_box
            })
            .child(
                Button::new(format!("account-select-{id}"))
                    .ghost()
                    .h_auto()
                    .p_3()
                    .min_w_0()
                    .flex_1()
                    .accessibility_label(format!("Use account {}", account.username))
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .child(minecraft_avatar::render(account.active_profile(), cx))
                            .child(
                                v_flex()
                                    .min_w_0()
                                    .flex_1()
                                    .gap_1()
                                    .text_left()
                                    .child(label)
                                    .child(muted(account.detail(), cx).text_xs().truncate()),
                            ),
                    )
                    .on_click(move |_, _, cx| {
                        app_select
                            .read(cx)
                            .accounts
                            .clone()
                            .update(cx, |store, cx| {
                                if store.set_active(&id) {
                                    cx.notify();
                                }
                            });
                    }),
            );
        if accounts.len() > 1 {
            let id = account.id.clone();
            let name = account.username.clone();
            let app = app.clone();
            row = row.child(
                div().pr_2().child(
                    icon_button(
                        format!("account-remove-{id}"),
                        PhaIcon::Trash2,
                        format!("Remove {}", account.username),
                    )
                    .on_click(move |_, window, cx| {
                        let app = app.clone();
                        let id = id.clone();
                        let name = name.clone();
                        window.open_alert_dialog(cx, move |dialog, _, _| {
                            let app = app.clone();
                            let id = id.clone();
                            dialog
                                .confirm()
                                .title(format!("Remove “{name}”?"))
                                .description(
                                    "The stored account is removed. You can add it again later.",
                                )
                                .button_props(
                                    DialogButtonProps::default()
                                        .show_cancel(true)
                                        .cancel_text("Keep account")
                                        .ok_text("Remove")
                                        .ok_variant(
                                            gpui_kit::component::button::ButtonVariant::Danger,
                                        ),
                                )
                                .on_ok(move |_, _, cx| {
                                    app.read(cx).accounts.clone().update(cx, |store, cx| {
                                        if store.remove(&id) {
                                            cx.notify();
                                        }
                                    });
                                    true
                                })
                        });
                    }),
                ),
            );
        }
        list = list.child(row);
    }
    let mut content = if compact {
        v_flex()
    } else {
        h_flex().items_start()
    };
    content = content
        .gap_6()
        .child(list.when(compact, |list| list.w_full()));
    if let Some(account) = active.as_ref().or_else(|| accounts.first())
        && let Some(profile) = account.active_profile()
    {
        let mut preview = panel(cx)
            .w_64()
            .when(compact, |preview| preview.w_full())
            .flex_shrink_0()
            .p_4()
            .gap_4()
            .child(
                h_flex()
                    .gap_3()
                    .child(minecraft_avatar::render(Some(profile), cx))
                    .child(div().text_base().font_medium().child(profile.name.clone())),
            )
            .child(
                muted(
                    format!("{} player profile preview", account.account_type.label()),
                    cx,
                )
                .text_xs(),
            )
            .child(crate::components::skin_preview::render(profile, cx));
        if account.profiles.len() > 1 {
            preview = preview.child(v_flex().gap_2().children(account.profiles.iter().map(
                |profile| {
                    let id = account.id.clone();
                    let profile_id = profile.id.clone();
                    let app = app.clone();
                    Button::new(format!("account-profile-{}", profile.id))
                        .label(profile.name.clone())
                        .when(profile.id == account.active_profile_id, |button| {
                            button.bg(cx.theme().accent)
                        })
                        .on_click(move |_, _, cx| {
                            app.read(cx).accounts.clone().update(cx, |store, cx| {
                                if store.set_active_profile(&id, profile_id.clone()) {
                                    cx.notify();
                                }
                            });
                        })
                },
            )));
        }
        content = content.child(preview);
    }
    page_shell(Some(title), content, cx)
}
