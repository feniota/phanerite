//! Retained controls for configuring a new, isolated game instance.

use crate::{
    pages::widgets::{control_slot, field, muted},
    route::Route,
    state::{AppState, Loader, NewInstance},
};
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IndexPath, Selectable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState, Textarea, TextareaState},
    notification::Notification,
    select::{Select, SelectState},
    slider::{Slider, SliderState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Styled as _, Subscription, Window, div, rems,
};

const MC_VERSIONS: [&str; 5] = ["1.21.4", "1.21.1", "1.20.1", "1.19.4", "1.12.2"];

struct InstanceDialog {
    app: Entity<AppState>,
    name: Entity<InputState>,
    description: Entity<TextareaState>,
    version: Entity<SelectState<Vec<&'static str>>>,
    loader: Loader,
    loader_version: Entity<InputState>,
    memory: Entity<SliderState>,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

pub fn open(window: &mut Window, cx: &mut App, app: Entity<AppState>) {
    let view = cx.new(|cx| {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. Modded Survival"));
        let description = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("What is this instance for?")
                .auto_grow(2, 4)
        });
        let version = cx
            .new(|cx| SelectState::new(MC_VERSIONS.to_vec(), Some(IndexPath::new(0)), window, cx));
        let loader_version = cx.new(|cx| InputState::new(window, cx).placeholder("No loader"));
        let memory = cx.new(|_| {
            SliderState::new()
                .min(2.)
                .max(16.)
                .step(1.)
                .default_value(4.)
        });
        let subscriptions = vec![
            cx.observe(&name, |_: &mut InstanceDialog, _, cx| cx.notify()),
            cx.observe(&memory, |_: &mut InstanceDialog, _, cx| cx.notify()),
        ];
        InstanceDialog {
            app,
            name,
            description,
            version,
            loader: Loader::Vanilla,
            loader_version,
            memory,
            error: None,
            _subscriptions: subscriptions,
        }
    });
    window.open_dialog(cx, move |dialog, window, _| {
        let view = view.clone();
        dialog
            .title("Create instance")
            .width(window.rem_size() * 36.)
            .content(move |content, _, _| content.child(view.clone()))
    });
}

impl InstanceDialog {
    fn valid(&self, cx: &App) -> bool {
        let name = self.name.read(cx).value();
        !name.trim().is_empty() && name.trim().chars().count() <= 64
    }

    fn select_loader(&mut self, loader: Loader, window: &mut Window, cx: &mut Context<Self>) {
        self.loader = loader;
        self.loader_version.update(cx, |input, cx| {
            input.set_placeholder(
                if loader == Loader::Vanilla {
                    "No loader"
                } else {
                    "Latest compatible version"
                },
                window,
                cx,
            );
        });
        cx.notify();
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.valid(cx) {
            return;
        }
        let Some(storage) = self.app.read(cx).storage() else {
            self.error = Some("Choose a game directory before creating an instance.".into());
            cx.notify();
            return;
        };
        let name = self.name.read(cx).value().trim().to_owned();
        let loader_version = self.loader_version.read(cx).value().trim().to_owned();
        let input = NewInstance {
            name: name.clone(),
            description: self.description.read(cx).value().trim().to_owned(),
            mc_version: self
                .version
                .read(cx)
                .selected_value()
                .copied()
                .unwrap_or(MC_VERSIONS[0])
                .into(),
            loader: self.loader,
            loader_version: if self.loader == Loader::Vanilla {
                "—".into()
            } else if loader_version.is_empty() {
                "Latest".into()
            } else {
                loader_version
            },
            memory: self.memory.read(cx).value().start().round() as u32,
        };
        let created = self.app.read(cx).instances.clone().update(cx, |store, cx| {
            let created = store.create(storage, input);
            if created.is_some() {
                cx.notify();
            }
            created
        });
        if let Some(reference) = created {
            self.app
                .update(cx, |app, cx| app.push(Route::InstanceDetail(reference), cx));
            window.close_dialog(cx);
            window.push_notification(
                Notification::success(format!("Instance “{name}” created")),
                cx,
            );
        } else {
            self.error = Some("The selected game directory is no longer available.".into());
            cx.notify();
        }
    }
}

impl Render for InstanceDialog {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let memory = self.memory.read(cx).value().start().round() as u32;
        let mut form = v_flex()
            .gap_4()
            .child(muted(
                "Set up a fresh game installation. Everything stays editable later.",
                cx,
            ))
            .child(
                field("Name", Input::new(&self.name))
                    .debug_selector(|| "create-instance-name".into()),
            )
            .child(
                h_flex()
                    .items_start()
                    .gap_4()
                    .child(
                        div().w(rems(9.)).child(field(
                            "Game version",
                            control_slot(
                                Select::new(&self.version)
                                    .w_full()
                                    .accessibility_label("Game version"),
                            ),
                        )),
                    )
                    .child(field(
                        "Mod loader",
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .children(Loader::ALL.into_iter().map(|loader| {
                                Button::new(format!("create-loader-{}", loader.key()))
                                    .label(loader.label())
                                    .selected(self.loader == loader)
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.select_loader(loader, window, cx)
                                    }))
                            })),
                    )),
            )
            .child(field(
                "Loader version",
                Input::new(&self.loader_version).disabled(self.loader == Loader::Vanilla),
            ))
            .child(muted("Leave empty to use the latest compatible loader build.", cx).text_xs())
            .child(field(
                format!("Maximum memory — {memory} GB"),
                Slider::new(&self.memory),
            ))
            .child(
                h_flex()
                    .justify_between()
                    .child(muted("2 GB", cx).text_xs())
                    .child(muted("16 GB", cx).text_xs()),
            )
            .child(field("Description", Textarea::new(&self.description)));
        if let Some(error) = &self.error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(error.clone()),
            );
        }
        if self.name.read(cx).value().trim().chars().count() > 64 {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child("Keep the name within 64 characters."),
            );
        }
        form.child(
            h_flex()
                .debug_selector(|| "create-instance-actions".into())
                .justify_end()
                .gap_2()
                .child(
                    Button::new("create-cancel")
                        .label("Cancel")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .child(
                    Button::new("create-submit")
                        .primary()
                        .label("Create instance")
                        .disabled(!self.valid(cx))
                        .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                ),
        )
    }
}
