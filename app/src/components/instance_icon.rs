//! Deterministic crystal artwork used to identify Minecraft instances.

use gpui::{
    App, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _,
};
use gpui_component::{h_flex, v_flex};

use crate::{
    palette,
    state::{InstanceSummary, Loader},
};

fn hash(seed: &str) -> u32 {
    seed.bytes().fold(2_166_136_261u32, |hash, byte| {
        (hash ^ byte as u32).wrapping_mul(16_777_619)
    })
}

fn ramp(loader: Loader) -> [u32; 4] {
    match loader {
        Loader::Vanilla => palette::ramp::VANILLA,
        Loader::Fabric => palette::ramp::FABRIC,
        Loader::Forge => palette::ramp::FORGE,
        Loader::NeoForge => palette::ramp::NEOFORGE,
    }
}

/// Deterministic 9×9 crystal lattice. The seed and loader fully determine the
/// silhouette, so gallery renders never change between frames.
pub fn render(instance: &InstanceSummary, _: &App) -> impl IntoElement {
    let mut state = hash(&instance.icon_seed);
    let colors = ramp(instance.loader);
    let mut rows = v_flex().gap_0();
    for row in 0..9 {
        let mut cells = h_flex().gap_0();
        for column in 0..9 {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let active = (column as i32 - 4).abs() <= 2
                && (row as i32 - 4).abs() <= 3
                && ((state >> 28) & 3) != 0;
            let tone = ((state >> 24) & 3) as usize;
            cells = cells.child(
                div()
                    .w(gpui::px(3.))
                    .h(gpui::px(3.))
                    .when(active, |cell| cell.bg(palette::color(colors[tone]))),
            );
        }
        rows = rows.child(cells);
    }
    rows.id(format!("instance-icon-{}", instance.id))
        .w(gpui::px(36.))
        .h(gpui::px(36.))
        .items_center()
        .justify_center()
}
