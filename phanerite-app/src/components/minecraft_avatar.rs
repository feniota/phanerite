//! Avatar element for Minecraft player profiles and account identities.

use gpui::{App, IntoElement};
use gpui_component::avatar::Avatar;

use crate::state::PlayerProfileSummary;

/// Uses the component Avatar's image path when available and preserves the
/// initial fallback while a remote skin is unavailable.
pub fn render(profile: Option<&PlayerProfileSummary>, _: &App) -> impl IntoElement {
    match profile {
        Some(profile) => Avatar::new()
            .name(profile.name.clone())
            .src(profile.skin_url.clone()),
        None => Avatar::new().name("Offline"),
    }
}
