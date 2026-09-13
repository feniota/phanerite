//! Retained GPUI controls bound to the launcher's existing stores.
//!
//! Keyed controllers own subscriptions. Model synchronization never feeds an
//! unchanged value back through the user-edit path, and incomplete text edits
//! remain in the input until the model actually changes.

use crate::state::AppState;
use gpui_kit::component::{
    IndexPath,
    input::{Input, InputEvent, InputState},
    searchable_list::SearchableListItem,
    select::{Select, SelectEvent, SelectState},
    slider::{Slider, SliderEvent, SliderState},
};
use gpui_kit::{App, AppContext as _, Context, Entity, SharedString, Subscription, Window};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub(crate) struct Choice {
    value: String,
    label: SharedString,
}

impl Choice {
    pub(crate) fn new(value: impl Into<String>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

impl SearchableListItem for Choice {
    type Value = String;
    fn value(&self) -> &String {
        &self.value
    }
    fn title(&self) -> SharedString {
        self.label.clone()
    }
}

struct InputBinding {
    state: Entity<InputState>,
    value: String,
    _subscriptions: Vec<Subscription>,
}
struct SelectBinding {
    state: Entity<SelectState<Vec<Choice>>>,
    value: String,
    _subscriptions: Vec<Subscription>,
}
struct SliderBinding {
    state: Entity<SliderState>,
    value: u32,
    _subscriptions: Vec<Subscription>,
}

pub(crate) fn input(
    id: impl Into<gpui_kit::ElementId>,
    app: Entity<AppState>,
    read: impl Fn(&AppState, &App) -> String + 'static,
    write: impl Fn(&mut AppState, String, &mut Context<AppState>) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> Input {
    let binding = window.use_keyed_state(id, cx, move |window, cx| {
        let read = Rc::new(read);
        let value = read(app.read(cx), cx);
        let state = cx.new(|cx| InputState::new(window, cx).default_value(value.clone()));
        let read_edit = read.clone();
        let app_edit = app.clone();
        let edit = cx.subscribe_in(
            &state,
            window,
            move |_: &mut InputBinding, input, event, _, cx| {
                if matches!(event, InputEvent::Change) {
                    let next = input.read(cx).value().to_string();
                    if next != read_edit(app_edit.read(cx), cx) {
                        app_edit.update(cx, |app, cx| write(app, next, cx));
                    }
                }
            },
        );
        let sync = cx.observe_in(
            &app,
            window,
            move |this: &mut InputBinding, app, window, cx| {
                let value = read(app.read(cx), cx);
                if this.value != value {
                    this.value = value.clone();
                    if this.state.read(cx).value().as_ref() != value {
                        this.state
                            .update(cx, |input, cx| input.set_value(value, window, cx));
                    }
                }
            },
        );
        InputBinding {
            state,
            value,
            _subscriptions: vec![edit, sync],
        }
    });
    Input::new(&binding.read(cx).state)
}

pub(crate) fn select(
    id: impl Into<gpui_kit::ElementId>,
    app: Entity<AppState>,
    choices: Vec<Choice>,
    read: impl Fn(&AppState, &App) -> String + 'static,
    write: impl Fn(&mut AppState, String, &mut Context<AppState>) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> Select<Vec<Choice>> {
    let binding = window.use_keyed_state(id, cx, move |window, cx| {
        let read = Rc::new(read);
        let value = read(app.read(cx), cx);
        let selected = choices
            .iter()
            .position(|item| item.value == value)
            .map(IndexPath::new);
        let state = cx.new(|cx| SelectState::new(choices, selected, window, cx));
        let app_edit = app.clone();
        let read_edit = read.clone();
        let edit = cx.subscribe(&state, move |_: &mut SelectBinding, _, event, cx| {
            if let SelectEvent::Confirm(Some(next)) = event
                && *next != read_edit(app_edit.read(cx), cx)
            {
                app_edit.update(cx, |app, cx| write(app, next.clone(), cx));
            }
        });
        let sync = cx.observe_in(
            &app,
            window,
            move |this: &mut SelectBinding, app, window, cx| {
                let value = read(app.read(cx), cx);
                if this.value != value {
                    this.value = value.clone();
                    this.state
                        .update(cx, |state, cx| state.set_selected_value(&value, window, cx));
                }
            },
        );
        SelectBinding {
            state,
            value,
            _subscriptions: vec![edit, sync],
        }
    });
    Select::new(&binding.read(cx).state)
}

pub(crate) fn slider(
    id: impl Into<gpui_kit::ElementId>,
    app: Entity<AppState>,
    read: impl Fn(&AppState, &App) -> u32 + 'static,
    write: impl Fn(&mut AppState, u32, &mut Context<AppState>) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> Slider {
    let binding = window.use_keyed_state(id, cx, move |window, cx| {
        let read = Rc::new(read);
        let value = read(app.read(cx), cx);
        let state = cx.new(|_| {
            SliderState::new()
                .min(1.)
                .max(16.)
                .step(1.)
                .default_value(value as f32)
        });
        let app_edit = app.clone();
        let read_edit = read.clone();
        let edit = cx.subscribe(&state, move |_: &mut SliderBinding, _, event, cx| {
            let SliderEvent::Change(value) = event else {
                return;
            };
            let next = value.start().round() as u32;
            if next != read_edit(app_edit.read(cx), cx) {
                app_edit.update(cx, |app, cx| write(app, next, cx));
            }
        });
        let sync = cx.observe_in(
            &app,
            window,
            move |this: &mut SliderBinding, app, window, cx| {
                let value = read(app.read(cx), cx);
                if this.value != value {
                    this.value = value;
                    this.state
                        .update(cx, |state, cx| state.set_value(value as f32, window, cx));
                }
            },
        );
        SliderBinding {
            state,
            value,
            _subscriptions: vec![edit, sync],
        }
    });
    Slider::new(&binding.read(cx).state)
}
