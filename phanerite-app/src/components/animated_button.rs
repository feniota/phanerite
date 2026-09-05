//! A `gpui_kit::base::Button` wrapper with an animated background target.

use gpui_kit::base::{
    Button,
    motion::{Transition, transition},
};
use gpui_kit::{
    App, ElementId, Hsla, IntoElement, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, Window,
};
use std::time::Duration;

pub fn render(
    id: impl Into<ElementId>,
    normal: Hsla,
    hover: Hsla,
    child: impl IntoElement,
    on_click: impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) + 'static,
    window: &mut Window,
    cx: &mut App,
) -> Button {
    let id = id.into();
    let hovered = window.use_keyed_state((id.clone(), "hover"), cx, |_, _| false);
    let background = transition(
        (id.clone(), "background"),
        if *hovered.read(cx) { hover } else { normal },
        Transition::new(Duration::from_millis(250)),
        window,
        cx,
    );
    let hovered = hovered.clone();

    Button::new(id)
        .bg(background)
        .cursor_pointer()
        .on_hover(move |is_hovered, _, cx| {
            hovered.update(cx, |hovered, cx| {
                if *hovered != *is_hovered {
                    *hovered = *is_hovered;
                    cx.notify();
                }
            });
        })
        .on_click(on_click)
        .child(child)
}
