//! Dialog for collecting the fields needed to create a Minecraft instance.

use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Entity, IntoElement, ParentElement as _, Styled as _, Window,
};

use crate::state::{AppState, Loader, NewInstance};

/// Opens the create-instance workflow with owned InputState entities retained
/// by the modal's element tree.
pub fn open(window: &mut Window, cx: &mut App, app: Entity<AppState>) {
    window.open_dialog(cx, move |dialog, window, cx| {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. Modded Survival"));
        let description =
            cx.new(|cx| InputState::new(window, cx).placeholder("What is this instance for?"));
        let name_for_submit = name.clone();
        let description_for_submit = description.clone();
        let app_for_submit = app.clone();
        dialog
            .title("Create instance")
            .content(move |content, _, _| {
                content.child(
                    v_flex()
                        .gap_4()
                        .child(labelled("Name", Input::new(&name)))
                        .child(
                            h_flex()
                                .gap_3()
                                .child(labelled("Game version", "1.21.4"))
                                .child(labelled("Mod loader", "Vanilla")),
                        )
                        .child(labelled("Description", Input::new(&description))),
                )
            })
            .footer(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("create-cancel")
                            .outline()
                            .label("Cancel")
                            .on_click(|_, window, cx| {
                                window.close_dialog(cx);
                            }),
                    )
                    .child(
                        Button::new("create-submit")
                            .primary()
                            .label("Create")
                            .on_click(move |_, window, cx| {
                                let name = name_for_submit.read(cx).value();
                                if name.trim().is_empty() {
                                    return;
                                }
                                let description = description_for_submit.read(cx).value();
                                app_for_submit.update(cx, |state, cx| {
                                    let Some(storage) = state.storage() else {
                                        return;
                                    };
                                    let created = state.instances.update(cx, |store, _| {
                                        store.create(
                                            storage,
                                            NewInstance {
                                                name: name.to_string(),
                                                description: description.to_string(),
                                                mc_version: "1.21.4".into(),
                                                loader: Loader::Vanilla,
                                                loader_version: "—".into(),
                                                memory: 4,
                                            },
                                        )
                                    });
                                    if let Some(reference) = created {
                                        state.push(
                                            crate::route::Route::InstanceDetail(reference),
                                            cx,
                                        );
                                    }
                                });
                                window.close_dialog(cx);
                            }),
                    ),
            )
    });
}

fn labelled(label: &'static str, child: impl IntoElement) -> impl IntoElement {
    v_flex()
        .gap_1()
        .child(gpui_kit::div().text_sm().child(label))
        .child(child)
}
