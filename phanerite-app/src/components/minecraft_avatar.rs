//! Cropped Minecraft head with a pixel-art fallback.
use crate::state::PlayerProfileSummary;
use gpui_kit::{
    App, IntoElement as _, ParentElement as _, Styled as _, StyledImage as _, div, img,
};

pub fn render(profile: Option<&PlayerProfileSummary>, _: &App) -> gpui_kit::Div {
    let name = profile.map(skin_name).unwrap_or("Steve");
    let slim = profile.is_some_and(|profile| profile.is_slim)
        || name.to_ascii_lowercase().contains("alex");
    div().size_9().flex_shrink_0().overflow_hidden().child(
        img(format!("https://mc-heads.net/avatar/{name}/64"))
            .size_full()
            .with_loading(move || super::skin_preview::fallback(slim, true).into_any_element())
            .with_fallback(move || super::skin_preview::fallback(slim, true).into_any_element()),
    )
}

pub(crate) fn skin_name(profile: &PlayerProfileSummary) -> &str {
    profile
        .skin_url
        .strip_prefix("https://mc-heads.net/skin/")
        .filter(|name| !name.is_empty())
        .unwrap_or(&profile.name)
}
