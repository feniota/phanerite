//! Bottom status bar for low-frequency application status and actions.

use gpui::{App, Entity, IntoElement, ParentElement as _, Styled as _, div};
use gpui_component::{separator::Separator, status_bar::StatusBar};

use crate::state::AppState;

/// This deliberately observes only low/medium-frequency phanerite-app state. It never
/// receives LaunchStore or LiveLogStore handles.
pub fn render(app: Entity<AppState>, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let count = state.instances.read(cx).len();
    let running = state.sessions.read(cx).running_count();
    let _account = state
        .accounts
        .read(cx)
        .active()
        .map(|item| item.username.clone());
    let bar = StatusBar::new()
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
            .left(format!("{running} running"))
    } else {
        bar
    }
}
