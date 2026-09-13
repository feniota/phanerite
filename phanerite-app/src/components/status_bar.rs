//! Bottom status bar for low-frequency application status and actions.

use gpui_kit::component::{separator::Separator, status_bar::StatusBar};
use gpui_kit::{App, Entity, IntoElement, ParentElement as _, Styled as _, Window, div};

use crate::state::AppState;

/// This deliberately observes only low/medium-frequency phanerite-app state. It never
/// receives LaunchStore or LiveLogStore handles.
pub fn render(app: Entity<AppState>, window: &Window, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let count = state.instances.read(cx).len();
    let running = state.sessions.read(cx).running_count();
    let radii = crate::window::content_radii(window);
    let bar = StatusBar::new()
        .rounded_bl(radii.bottom_left)
        .rounded_br(radii.bottom_right)
        .left("Phanerite")
        .left(
            div()
                .font_family(crate::theme::MONO_FONT_FAMILY)
                .child("0.1.0-pre"),
        )
        .left(Separator::vertical())
        .left(format!("{count} instances"));
    if running > 0 {
        bar.left(Separator::vertical())
            .left(super::running_instances_popover::render(app, cx))
    } else {
        bar
    }
}
