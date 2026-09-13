//! A focused form for registering a local Java executable.
use crate::{
    pages::widgets::{field, muted},
    state::AppState,
};
use gpui_kit::component::{
    Disableable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    Subscription, Window, div,
};

struct RuntimeDialog {
    app: Entity<AppState>,
    name: Entity<InputState>,
    version: Entity<InputState>,
    path: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}
pub(crate) fn open(window: &mut Window, cx: &mut App, app: Entity<AppState>) {
    let view = cx.new(|cx| {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. Oracle JDK 21"));
        let version = cx.new(|cx| InputState::new(window, cx).default_value("21"));
        let path = cx.new(|cx| InputState::new(window, cx).placeholder("/path/to/bin/java"));
        let subscriptions = vec![
            cx.observe(&name, |_: &mut RuntimeDialog, _, cx| cx.notify()),
            cx.observe(&version, |_: &mut RuntimeDialog, _, cx| cx.notify()),
            cx.observe(&path, |_: &mut RuntimeDialog, _, cx| cx.notify()),
        ];
        RuntimeDialog {
            app,
            name,
            version,
            path,
            _subscriptions: subscriptions,
        }
    });
    window.open_dialog(cx, move |dialog, window, _| {
        let view = view.clone();
        dialog
            .title("Add Java runtime")
            .width(window.rem_size() * 30.)
            .content(move |content, _, _| content.child(view.clone()))
    });
}
impl RuntimeDialog {
    fn valid(&self, cx: &App) -> bool {
        !self.name.read(cx).value().trim().is_empty()
            && !self.path.read(cx).value().trim().is_empty()
            && self
                .version
                .read(cx)
                .value()
                .parse::<u32>()
                .is_ok_and(|version| (8..=99).contains(&version))
    }
}
impl Render for RuntimeDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_4()
            .child(muted(
                "Add a Java executable installed outside Phanerite’s managed runtimes.",
                cx,
            ))
            .child(field("Name", Input::new(&self.name)))
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        div()
                            .w_24()
                            .child(field("Version", Input::new(&self.version))),
                    )
                    .child(field("Java executable", Input::new(&self.path))),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("runtime-cancel")
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("runtime-submit")
                            .primary()
                            .label("Add runtime")
                            .disabled(!self.valid(cx))
                            .on_click(cx.listener(|this, _, window, cx| {
                                if !this.valid(cx) {
                                    return;
                                }
                                let name = this.name.read(cx).value().trim().to_owned();
                                let path = this.path.read(cx).value().trim().to_owned();
                                let version = this.version.read(cx).value().parse().unwrap_or(21);
                                let added =
                                    this.app.read(cx).settings.clone().update(cx, |store, cx| {
                                        let added = store.add_runtime(name, version, path);
                                        if added {
                                            cx.notify();
                                        }
                                        added
                                    });
                                if added {
                                    window.close_dialog(cx);
                                }
                            })),
                    ),
            )
    }
}
