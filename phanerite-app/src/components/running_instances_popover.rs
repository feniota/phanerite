//! Popover that summarizes currently running Minecraft instances.

use gpui_kit::component::StyledExt as _;
use gpui_kit::component::v_flex;
use gpui_kit::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div,
};

use crate::state::AppState;

/// Content used by the status-bar running-instance affordance. The popover
/// wiring is intentionally local to this component when launch controls land.
pub fn content(app: Entity<AppState>, cx: &App) -> impl IntoElement {
    let sessions = app.read(cx).sessions.read(cx);
    v_flex()
        .id("running-instances-popover")
        .w_56()
        .gap_2()
        .p_3()
        .child(div().font_semibold().child("Running instances"))
        .children(
            sessions
                .running()
                .map(|session| div().text_sm().child(session.instance.instance_id.clone())),
        )
}
