//! Player portrait, with built-in pixel artwork while remote skins are unavailable.
use crate::{
    palette::{self, skin},
    state::PlayerProfileSummary,
};
use gpui_kit::{
    App, Bounds, IntoElement, ParentElement as _, Styled as _, StyledImage as _, canvas, div, fill,
    img, point, size,
};

pub(crate) fn render(profile: &PlayerProfileSummary, _: &App) -> impl IntoElement {
    let slim = profile.is_slim || profile.name.to_ascii_lowercase().contains("alex");
    div().h_80().w_full().p_4().child(
        img(format!(
            "https://mc-heads.net/body/{}/256",
            super::minecraft_avatar::skin_name(profile)
        ))
        .size_full()
        .object_fit(gpui_kit::ObjectFit::Contain)
        .with_loading(move || fallback(slim, false).into_any_element())
        .with_fallback(move || fallback(slim, false).into_any_element()),
    )
}

pub(crate) fn fallback(slim: bool, head_only: bool) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let width = if head_only { 8. } else { 16. };
            let height = if head_only { 8. } else { 32. };
            let unit = (bounds.size.width / width)
                .min(bounds.size.height / height)
                .floor();
            let origin = bounds.origin
                + point(
                    (bounds.size.width - unit * width) / 2.,
                    (bounds.size.height - unit * height) / 2.,
                );
            let mut block = |x: f32, y: f32, w: f32, h: f32, color: u32| {
                window.paint_quad(fill(
                    Bounds::new(origin + point(unit * x, unit * y), size(unit * w, unit * h)),
                    palette::color(color),
                ));
            };
            let head_x = if head_only { 0. } else { 4. };
            let hair = if slim { skin::ALEX_HAIR } else { skin::HAIR };
            let shirt = if slim { skin::ALEX_SHIRT } else { skin::SHIRT };
            if !head_only {
                block(4., 8., 8., 12., shirt);
                block(4., 18., 8., 2., skin::SHIRT_SHADE);
                block(0., 8., 4., 4., shirt);
                block(12., 8., 4., 4., shirt);
                block(0., 12., 4., 8., skin::SKIN);
                block(12., 12., 4., 8., skin::SKIN_SHADE);
                block(4., 20., 4., 10., skin::TROUSERS);
                block(8., 20., 4., 10., skin::TROUSERS);
                block(7., 21., 1., 10., skin::SHOES);
                block(4., 30., 8., 2., skin::SHOES);
                block(7., 8., 2., 1., skin::SKIN);
            }
            block(head_x, 0., 8., 8., skin::SKIN);
            block(head_x, 0., 8., 2., hair);
            block(head_x, 2., 1., 2., hair);
            block(head_x + 7., 2., 1., 2., hair);
            block(head_x + 1., 4., 2., 1., skin::EYES);
            block(head_x + 5., 4., 2., 1., skin::EYES);
            block(head_x + 2., 4., 1., 1., skin::TROUSERS);
            block(head_x + 5., 4., 1., 1., skin::TROUSERS);
            block(head_x + 3., 5., 2., 1., skin::SKIN_SHADE);
            block(head_x + 2., 6., 4., 2., hair);
            block(head_x + 3., 6., 2., 1., skin::SKIN);
        },
    )
    .size_full()
}
