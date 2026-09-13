//! Shared page geometry and typography, following the hand-tuned instance list.

use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, StyledExt as _,
    button::{Button, ButtonCustomVariant, ButtonVariants as _},
    h_flex,
    tag::Tag,
    v_flex,
};
use gpui_kit::{
    App, Div, Entity, IntoElement, ParentElement as _, SharedString, Styled as _, div,
    prelude::FluentBuilder as _,
};

use crate::{
    assets::PhaIcon,
    route::{InstanceRef, Route},
    state::AppState,
};

pub(crate) fn heading(icon: PhaIcon, title: impl Into<SharedString>, cx: &App) -> Div {
    h_flex()
        .gap_2()
        .min_w_0()
        .child(Icon::new(icon).size_5().text_color(cx.theme().primary))
        .child(div().text_lg().font_semibold().child(title.into()))
}

pub(crate) fn muted(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

pub(crate) fn badge(text: impl Into<SharedString>) -> Tag {
    Tag::secondary()
        .small()
        .rounded_full()
        .border_0()
        .child(text.into())
}

pub(crate) fn header(
    title: impl IntoElement,
    description: impl Into<SharedString>,
    actions: impl IntoElement,
    cx: &App,
) -> Div {
    h_flex()
        .flex_shrink_0()
        .gap_4()
        .px_6()
        .pt_6()
        .pb_4()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            v_flex()
                .min_w_0()
                .flex_1()
                .gap_1()
                .child(title)
                .child(muted(description, cx)),
        )
        .child(actions)
}

pub(crate) fn instance_header(
    instance: &crate::state::InstanceSummary,
    title: impl IntoElement,
    description: impl Into<SharedString>,
    actions: impl IntoElement,
    app: Entity<AppState>,
    cx: &App,
) -> Div {
    v_flex()
        .flex_shrink_0()
        .gap_3()
        .px_6()
        .pt_4()
        .pb_4()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(
            h_flex().child(
                super::route_button(
                    "instance-back",
                    instance.name.clone(),
                    Route::InstanceDetail(instance.reference()),
                    app,
                )
                .ghost()
                .small()
                .icon(PhaIcon::ArrowLeft),
            ),
        )
        .child(
            h_flex()
                .gap_4()
                .child(
                    v_flex()
                        .min_w_0()
                        .flex_1()
                        .gap_1()
                        .child(title)
                        .child(muted(description, cx)),
                )
                .child(actions),
        )
}

pub(crate) fn panel(cx: &App) -> Div {
    v_flex()
        .min_w_0()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().group_box)
}

pub(crate) fn section(
    title: impl Into<SharedString>,
    description: impl Into<SharedString>,
    body: impl IntoElement,
    cx: &App,
) -> Div {
    let description = description.into();
    let mut heading = v_flex()
        .gap_1()
        .child(div().text_sm().font_semibold().child(title.into()));
    if !description.is_empty() {
        heading = heading.child(muted(description, cx));
    }
    panel(cx).p_4().gap_4().child(heading).child(body)
}

pub(crate) fn field(label: impl Into<SharedString>, control: impl IntoElement) -> Div {
    v_flex()
        .min_w_0()
        .flex_1()
        .gap_2()
        .child(div().text_sm().font_medium().child(label.into()))
        .child(control)
}

/// SelectState renders a full-size outer view. Give it a definite height so
/// it cannot stretch a wrapping toolbar or a vertical form field.
pub(crate) fn control_slot(control: impl IntoElement) -> Div {
    div()
        .w_full()
        .h_8()
        .min_w_0()
        .flex_shrink_0()
        .child(control)
}

pub(crate) fn icon_tile(icon: PhaIcon, cx: &App) -> Div {
    h_flex()
        .size_10()
        .flex_shrink_0()
        .justify_center()
        .rounded(cx.theme().radius)
        .bg(cx.theme().secondary)
        .child(
            Icon::new(icon)
                .size_5()
                .text_color(cx.theme().muted_foreground),
        )
}

pub(crate) fn empty(icon: PhaIcon, title: &str, description: &str, cx: &App) -> Div {
    v_flex()
        .min_h_64()
        .items_center()
        .justify_center()
        .gap_3()
        .p_6()
        .child(
            Icon::new(icon)
                .size_8()
                .text_color(cx.theme().muted_foreground),
        )
        .child(div().text_base().font_semibold().child(title.to_owned()))
        .child(muted(description.to_owned(), cx).text_center())
}

pub(crate) fn icon_button(
    id: impl Into<gpui_kit::ElementId>,
    icon: PhaIcon,
    label: impl Into<SharedString>,
) -> Button {
    let label = label.into();
    Button::new(id)
        .ghost()
        .icon(icon)
        .tooltip(label.clone())
        .accessibility_label(label)
}

pub(crate) fn favorite_button(
    id: impl Into<gpui_kit::ElementId>,
    favorite: bool,
    cx: &App,
) -> Button {
    icon_button(
        id,
        if favorite {
            PhaIcon::StarFilled
        } else {
            PhaIcon::Star
        },
        if favorite {
            "Remove from favorites"
        } else {
            "Add to favorites"
        },
    )
    .toggled(favorite)
    .text_color(cx.theme().foreground)
    .when(favorite, |button| {
        let background = cx.theme().primary.opacity(0.15);
        button
            .custom(
                ButtonCustomVariant::new(cx)
                    .color(background)
                    .foreground(cx.theme().foreground)
                    .hover(cx.theme().primary.opacity(0.2))
                    .active(cx.theme().primary.opacity(0.25)),
            )
            // The custom variant mixes its normal color with transparency.
            .bg(background)
    })
}

pub(crate) fn instance_key(reference: &InstanceRef, suffix: &str) -> String {
    format!(
        "{}:{}:{suffix}",
        reference.storage.root_dir.display(),
        reference.instance_id
    )
}
