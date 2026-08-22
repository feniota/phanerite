//! Dialog for choosing and adding an account provider.

use gpui::{App, AppContext as _, Entity, ParentElement as _, Styled as _, Window};
use gpui_component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    input::{Input, InputState},
    v_flex,
};

use crate::state::{AccountType, AppState};

pub fn open(window: &mut Window, cx: &mut App, app: Entity<AppState>) {
    window.open_dialog(cx, move |dialog, window, cx| {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. Steve"));
        let submit_name = name.clone();
        let app = app.clone();
        dialog
            .title("Add account")
            .content(move |content, _, _| {
                content.child(
                    v_flex()
                        .gap_3()
                        .child(gpui::div().text_sm().child("Offline account"))
                        .child(Input::new(&name))
                        .child(
                            gpui::div()
                                .text_sm()
                                .child("Offline accounts work for single-player only."),
                        ),
                )
            })
            .footer(
                gpui_component::h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("account-dialog-cancel")
                            .outline()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("account-dialog-submit")
                            .primary()
                            .label("Add account")
                            .on_click(move |_, window, cx| {
                                let name = submit_name.read(cx).value();
                                if !name.trim().is_empty() {
                                    app.update(cx, |state, cx| {
                                        state.accounts.update(cx, |accounts, cx| {
                                            accounts.add(
                                                name,
                                                AccountType::Offline,
                                                None,
                                                vec![],
                                                None,
                                            );
                                            cx.notify();
                                        });
                                    });
                                    window.close_dialog(cx);
                                }
                            }),
                    ),
            )
    });
}
