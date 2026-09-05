//! Shared confirmation dialog for destructive actions.

use gpui_kit::App;
use gpui_kit::Window;
use gpui_kit::component::{Icon, IconName, WindowExt as _};
use std::rc::Rc;

/// Opens the shared destructive confirmation pattern. The callback performs
/// the actual deletion only after the user confirms.
pub fn open(
    window: &mut Window,
    cx: &mut App,
    title: impl Into<String>,
    consequence: impl Into<String>,
    on_confirm: impl Fn(&gpui_kit::ClickEvent, &mut Window, &mut App) -> bool + 'static,
) {
    let title = title.into();
    let consequence = consequence.into();
    let on_confirm = Rc::new(on_confirm);
    window.open_alert_dialog(cx, move |alert, _, _| {
        let on_confirm = on_confirm.clone();
        alert
            .confirm()
            .icon(Icon::new(IconName::TriangleAlert))
            .title(title.clone())
            .description(format!("{} This cannot be undone.", consequence))
            .on_ok(move |event, window, cx| on_confirm(event, window, cx))
    });
}
