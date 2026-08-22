//! Bottom status bar for low-frequency application status and actions.

use gpui::{App, Entity, IntoElement, ParentElement as _};
use gpui_component::status_bar::StatusBar;

use crate::state::AppState;

/// This deliberately observes only low/medium-frequency phanerite-app state. It never
/// receives LaunchStore or LiveLogStore handles.
pub fn render(app: Entity<AppState>, cx: &App) -> impl IntoElement {
    let state = app.read(cx);
    let count = state.instances.read(cx).len();
    let running = state.sessions.read(cx).running_count();
    let account = state
        .accounts
        .read(cx)
        .active()
        .map(|item| item.username.clone());
    let bar = StatusBar::new()
        .left("Phanerite")
        .left("0.1.0-pre")
        .child(format!("{count} instances"));
    let bar = if running > 0 {
        bar.child(format!("{running} running"))
    } else {
        bar
    };
    if let Some(account) = account {
        bar.right(format!("Signed in as {account}"))
    } else {
        bar
    }
}
