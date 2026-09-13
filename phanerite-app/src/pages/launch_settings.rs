//! Per-instance overrides and shared launch setting controls.

use super::{missing_resource, page_shell, widgets::*};
use crate::{
    assets::PhaIcon,
    components::form::{self, Choice},
    route::InstanceRef,
    state::{
        ADVANCED_FLAG_FIELDS, ADVANCED_TEXT_FIELDS, AppState, LaunchField, LaunchSettings,
        LaunchValue, MemoryMode, QuickPlayMode, WindowMode,
    },
};
use gpui_kit::component::{
    Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    switch::Switch,
    v_flex,
};
use gpui_kit::{
    App, Context, Div, Entity, IntoElement as _, ParentElement as _, Styled as _, Window, div,
};

pub(crate) fn effective(
    app: &AppState,
    reference: Option<&InstanceRef>,
    cx: &App,
) -> LaunchSettings {
    let global = app.settings.read(cx).launch();
    reference
        .and_then(|reference| app.instances.read(cx).find(reference))
        .map(|instance| instance.launch_overrides.resolve(global))
        .unwrap_or_else(|| global.clone())
}

fn overridden(
    app: &AppState,
    reference: Option<&InstanceRef>,
    field: LaunchField,
    cx: &App,
) -> bool {
    reference.is_none_or(|reference| {
        app.instances
            .read(cx)
            .find(reference)
            .is_some_and(|instance| instance.launch_overrides.is_overridden(field))
    })
}

pub(crate) fn set_value(
    app: &mut AppState,
    reference: Option<&InstanceRef>,
    field: LaunchField,
    value: LaunchValue,
    cx: &mut Context<AppState>,
) {
    if let Some(reference) = reference {
        app.instances.update(cx, |store, cx| {
            if store.set_launch_override(reference, field, value) {
                cx.notify();
            }
        });
    } else {
        app.settings.update(cx, |store, cx| {
            if store.update_launch(|settings| settings.set(field, value)) {
                cx.notify();
            }
        });
    }
}

fn key(reference: Option<&InstanceRef>, field: LaunchField) -> String {
    reference
        .map(|reference| instance_key(reference, field.key()))
        .unwrap_or_else(|| format!("global-launch-{}", field.key()))
}

fn override_fields(
    reference: &InstanceRef,
    fields: &[LaunchField],
    enabled: bool,
    app: &Entity<AppState>,
    cx: &mut App,
) {
    let current = effective(app.read(cx), Some(reference), cx);
    app.read(cx).instances.clone().update(cx, |store, cx| {
        let mut changed = false;
        for field in fields {
            changed |= if enabled {
                store.set_launch_override(reference, *field, current.get(*field))
            } else {
                store.clear_launch_override(reference, *field)
            };
        }
        if changed {
            cx.notify();
        }
    });
}

fn label_line(
    label: &str,
    reference: Option<&InstanceRef>,
    fields: Vec<LaunchField>,
    app: Entity<AppState>,
    cx: &App,
) -> Div {
    let mut line = h_flex()
        .justify_between()
        .gap_3()
        .child(div().text_sm().font_medium().child(label.to_owned()));
    if let Some(reference) = reference {
        let enabled = overridden(app.read(cx), Some(reference), fields[0], cx);
        let target = reference.clone();
        let control = Switch::new(format!("{}-override", key(Some(reference), fields[0])))
            .checked(enabled)
            .accessibility_label(format!("Override {label}"))
            .on_click(move |enabled, _, cx| override_fields(&target, &fields, *enabled, &app, cx));
        line = line.child(
            h_flex()
                .gap_2()
                .child(muted(if enabled { "Override" } else { "Global" }, cx).text_xs())
                .child(control),
        );
    }
    line
}

pub(crate) fn text_field(
    label: &str,
    reference: Option<&InstanceRef>,
    field: LaunchField,
    help: &str,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let enabled = overridden(app.read(cx), reference, field, cx);
    let read_target = reference.cloned();
    let write_target = reference.cloned();
    let control = form::input(
        key(reference, field),
        app.clone(),
        move |app, cx| match effective(app, read_target.as_ref(), cx).get(field) {
            LaunchValue::Text(value) => value,
            LaunchValue::Number(value) => value.to_string(),
            _ => String::new(),
        },
        move |app, value, cx| {
            let value = if matches!(field, LaunchField::WindowWidth | LaunchField::WindowHeight) {
                let Ok(number) = value.parse::<u32>() else {
                    return;
                };
                if !(1..=16384).contains(&number) {
                    return;
                }
                LaunchValue::Number(number)
            } else {
                LaunchValue::Text(value)
            };
            set_value(app, write_target.as_ref(), field, value, cx);
        },
        window,
        cx,
    )
    .w_full()
    .disabled(!enabled);
    let mut group = v_flex()
        .flex_1()
        .min_w_0()
        .gap_2()
        .child(label_line(label, reference, vec![field], app, cx))
        .child(control);
    if !help.is_empty() {
        group = group.child(muted(help.to_owned(), cx).text_xs());
    }
    group
}

fn choice_value(value: LaunchValue) -> String {
    match value {
        LaunchValue::Memory(mode) => mode.label().into(),
        LaunchValue::Window(mode) => mode.label().into(),
        LaunchValue::QuickPlay(mode) => mode.label().into(),
        _ => String::new(),
    }
}

pub(crate) fn selector(
    label: &str,
    reference: Option<&InstanceRef>,
    field: LaunchField,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let choices = match field {
        LaunchField::MemoryMode => MemoryMode::ALL
            .iter()
            .map(|value| Choice::new(value.label(), value.label()))
            .collect(),
        LaunchField::WindowMode => WindowMode::ALL
            .iter()
            .map(|value| Choice::new(value.label(), value.label()))
            .collect(),
        LaunchField::QuickPlayMode => QuickPlayMode::ALL
            .iter()
            .map(|value| Choice::new(value.label(), value.label()))
            .collect(),
        _ => Vec::new(),
    };
    let enabled = overridden(app.read(cx), reference, field, cx);
    let read_target = reference.cloned();
    let write_target = reference.cloned();
    let control = form::select(
        key(reference, field),
        app.clone(),
        choices,
        move |app, cx| choice_value(effective(app, read_target.as_ref(), cx).get(field)),
        move |app, value, cx| {
            let value = match field {
                LaunchField::MemoryMode => MemoryMode::ALL
                    .into_iter()
                    .find(|mode| mode.label() == value)
                    .map(LaunchValue::Memory),
                LaunchField::WindowMode => WindowMode::ALL
                    .into_iter()
                    .find(|mode| mode.label() == value)
                    .map(LaunchValue::Window),
                LaunchField::QuickPlayMode => QuickPlayMode::ALL
                    .into_iter()
                    .find(|mode| mode.label() == value)
                    .map(LaunchValue::QuickPlay),
                _ => None,
            };
            if let Some(value) = value {
                set_value(app, write_target.as_ref(), field, value, cx);
            }
        },
        window,
        cx,
    )
    .w_full()
    .disabled(!enabled)
    .accessibility_label(label.to_owned());
    let fields = if field == LaunchField::WindowMode {
        vec![field, LaunchField::WindowWidth, LaunchField::WindowHeight]
    } else if field == LaunchField::QuickPlayMode {
        vec![field, LaunchField::QuickPlayTarget]
    } else {
        vec![field]
    };
    v_flex()
        .flex_1()
        .min_w_0()
        .gap_2()
        .child(label_line(label, reference, fields, app, cx))
        .child(control_slot(control))
}

pub(crate) fn memory(
    reference: Option<&InstanceRef>,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    let launch = effective(app.read(cx), reference, cx);
    let enabled = overridden(app.read(cx), reference, LaunchField::Memory, cx);
    let read_target = reference.cloned();
    let write_target = reference.cloned();
    let slider = form::slider(
        key(reference, LaunchField::Memory),
        app.clone(),
        move |app, cx| effective(app, read_target.as_ref(), cx).memory,
        move |app, value, cx| {
            set_value(
                app,
                write_target.as_ref(),
                LaunchField::Memory,
                LaunchValue::Number(value),
                cx,
            )
        },
        window,
        cx,
    )
    .disabled(!enabled || launch.memory_mode == MemoryMode::Auto);
    let mut group = v_flex()
        .gap_3()
        .child(label_line(
            &format!("Memory allocation · {} GB", launch.memory),
            reference,
            vec![LaunchField::Memory, LaunchField::MemoryMode],
            app.clone(),
            cx,
        ))
        .child(selector(
            "Allocation mode",
            reference,
            LaunchField::MemoryMode,
            app.clone(),
            window,
            cx,
        ))
        .child(slider)
        .child(
            muted(
                if launch.memory_mode == MemoryMode::Auto {
                    "Phanerite chooses a safe allocation for the machine.".into()
                } else {
                    format!("{} GB maximum heap", launch.memory)
                },
                cx,
            )
            .text_xs(),
        );
    if let Some(reference) = reference.filter(|_| enabled) {
        let target = reference.clone();
        group = group.child(
            div().child(
                Button::new(format!(
                    "{}-reset",
                    key(Some(reference), LaunchField::Memory)
                ))
                .ghost()
                .small()
                .label("Reset to global")
                .on_click(move |_, _, cx| {
                    override_fields(
                        &target,
                        &[LaunchField::Memory, LaunchField::MemoryMode],
                        false,
                        &app,
                        cx,
                    )
                }),
            ),
        );
    }
    group
}

pub(crate) fn flag(
    label: &str,
    reference: Option<&InstanceRef>,
    field: LaunchField,
    app: Entity<AppState>,
    cx: &App,
) -> Div {
    let enabled = matches!(
        effective(app.read(cx), reference, cx).get(field),
        LaunchValue::Flag(true)
    );
    let target = reference.cloned();
    let app_toggle = app.clone();
    let mut controls = h_flex().gap_2();
    if let Some(reference) =
        reference.filter(|reference| overridden(app.read(cx), Some(reference), field, cx))
    {
        let target = reference.clone();
        let app = app.clone();
        controls = controls.child(
            icon_button(
                format!("{}-reset", key(Some(reference), field)),
                PhaIcon::RefreshCw,
                format!("Reset {label} to global"),
            )
            .small()
            .on_click(move |_, _, cx| override_fields(&target, &[field], false, &app, cx)),
        );
    }
    controls = controls.child(
        Switch::new(key(reference, field))
            .checked(enabled)
            .accessibility_label(label.to_owned())
            .on_click(move |value, _, cx| {
                app_toggle.update(cx, |app, cx| {
                    set_value(app, target.as_ref(), field, LaunchValue::Flag(*value), cx)
                })
            }),
    );
    h_flex()
        .min_h_8()
        .justify_between()
        .gap_4()
        .child(div().text_sm().child(label.to_owned()))
        .child(controls)
}

pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    let Some(instance) = app.read(cx).instances.read(cx).find(reference).cloned() else {
        return missing_resource("instance", app).into_any_element();
    };
    let title = instance_header(
        &instance,
        heading(PhaIcon::Settings, "Launch settings", cx),
        "Values inherit global defaults until you override them for this instance.",
        div(),
        app.clone(),
        cx,
    );
    let target = Some(reference);
    let mut basic = v_flex()
        .gap_4()
        .child(memory(target, app.clone(), window, cx))
        .child(selector(
            "Window mode",
            target,
            LaunchField::WindowMode,
            app.clone(),
            window,
            cx,
        ));
    basic = basic.child(
        h_flex()
            .items_start()
            .gap_4()
            .child(text_field(
                "Width",
                target,
                LaunchField::WindowWidth,
                "",
                app.clone(),
                window,
                cx,
            ))
            .child(text_field(
                "Height",
                target,
                LaunchField::WindowHeight,
                "",
                app.clone(),
                window,
                cx,
            )),
    );
    let launch = effective(app.read(cx), target, cx);
    let mut quick_play = v_flex().gap_3().child(selector(
        "Destination",
        target,
        LaunchField::QuickPlayMode,
        app.clone(),
        window,
        cx,
    ));
    if launch.quick_play_mode != QuickPlayMode::None {
        quick_play = quick_play.child(text_field(
            launch.quick_play_mode.target_placeholder(),
            target,
            LaunchField::QuickPlayTarget,
            "",
            app.clone(),
            window,
            cx,
        ));
    }
    let mut advanced = v_flex().gap_4();
    for (label, field, description) in ADVANCED_TEXT_FIELDS {
        advanced = advanced.child(text_field(
            label,
            target,
            field,
            description,
            app.clone(),
            window,
            cx,
        ));
    }
    for (label, field) in ADVANCED_FLAG_FIELDS {
        advanced = advanced.child(flag(label, target, field, app.clone(), cx));
    }
    let content = v_flex()
        .gap_4()
        .child(section(
            "Memory & window",
            "Use the switches to override global values for this instance.",
            basic,
            cx,
        ))
        .child(section(
            "Quick play",
            "Open a destination as soon as the game starts.",
            quick_play,
            cx,
        ))
        .child(section(
            "Advanced",
            "Commands, JVM options, and native library overrides.",
            advanced,
            cx,
        ));
    page_shell(Some(title), content, cx).into_any_element()
}
