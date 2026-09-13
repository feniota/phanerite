//! Instance resource collection.
use crate::{route::InstanceRef, state::AppState};
use gpui_kit::{App, Entity, Window};
pub fn render(
    reference: &InstanceRef,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> gpui_kit::AnyElement {
    super::resources::render(
        reference,
        super::resources::ResourceKind::Packs,
        app,
        window,
        cx,
    )
}
