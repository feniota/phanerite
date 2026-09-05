//! Dialog for importing resource packs and shader packs into an instance.

use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    v_flex,
};
use gpui_kit::{App, Entity, ParentElement as _, Styled as _, Window};

use crate::state::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceMode {
    Mods,
    Packs,
    Shaders,
}

impl ResourceMode {
    fn title(self) -> &'static str {
        match self {
            Self::Mods => "Add mods",
            Self::Packs => "Import resource packs",
            Self::Shaders => "Import shader packs",
        }
    }
}

pub fn open(window: &mut Window, cx: &mut App, _: Entity<AppState>, mode: ResourceMode) {
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title(mode.title())
            .content(|content, _, _| {
                content.child(
                    v_flex()
                        .items_center()
                        .gap_2()
                        .p_6()
                        .child(gpui_kit::div().text_lg().child("Drop files here"))
                        .child(
                            gpui_kit::div()
                                .text_sm()
                                .child("or choose files from your computer."),
                        ),
                )
            })
            .footer(
                gpui_kit::component::h_flex()
                    .justify_end()
                    .gap_2()
                    .child(
                        Button::new("resources-cancel")
                            .outline()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("resources-browse")
                            .primary()
                            .label("Browse…")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    ),
            )
    });
}
