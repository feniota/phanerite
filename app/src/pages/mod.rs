//! Route-driven page renderers for the main application content area.

pub mod accounts;
pub mod aphanite;
pub mod crash_report;
pub mod instance_detail;
pub mod instances;
pub mod launch_settings;
pub mod logs;
pub mod mods;
pub mod packs;
pub mod play;
pub mod settings;
pub mod setup;
pub mod shaders;
pub mod worlds;

use gpui::{
    App, Entity, InteractiveElement as _, IntoElement, ParentElement as _, Styled as _, Window, div,
};

use crate::{
    route::{CrashRef, InstanceRef, Route},
    state::AppState,
};

pub fn render(
    route: &Route,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    div()
        .id("route-content")
        .size_full()
        .min_w_0()
        .min_h_0()
        .child(match route {
            Route::Setup => setup::render(app, window, cx).into_any_element(),
            Route::Play => play::render(app, window, cx).into_any_element(),
            Route::Instances => instances::render(app, window, cx).into_any_element(),
            Route::Aphanite => aphanite::render(app, window, cx).into_any_element(),
            Route::InstanceDetail(reference) => {
                instance_detail::render(reference, app, window, cx).into_any_element()
            }
            Route::Mods(reference) => mods::render(reference, app, window, cx).into_any_element(),
            Route::Packs(reference) => packs::render(reference, app, window, cx).into_any_element(),
            Route::Shaders(reference) => {
                shaders::render(reference, app, window, cx).into_any_element()
            }
            Route::Worlds(reference) => {
                worlds::render(reference, app, window, cx).into_any_element()
            }
            Route::Logs(reference) => logs::render(reference, app, window, cx).into_any_element(),
            Route::LaunchSettings(reference) => {
                launch_settings::render(reference, app, window, cx).into_any_element()
            }
            Route::Crash(reference) => {
                crash_report::render(reference, app, window, cx).into_any_element()
            }
            Route::Accounts => accounts::render(app, window, cx).into_any_element(),
            Route::Settings => settings::render(app, window, cx).into_any_element(),
        })
}

pub(crate) fn page_shell(
    title: impl Into<gpui::SharedString>,
    description: impl Into<gpui::SharedString>,
    content: impl IntoElement,
    cx: &App,
) -> impl IntoElement {
    use gpui_component::{ActiveTheme as _, Sizable as _, StyledExt, v_flex};
    v_flex()
        .size_full()
        .min_h_0()
        .bg(cx.theme().background)
        .child(
            v_flex()
                .gap_1()
                .px_6()
                .pt_6()
                .pb_4()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(div().text_lg().font_semibold().child(title.into()))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(description.into()),
                ),
        )
        .child(div().flex_1().min_h_0().p_6().child(content))
}

pub(crate) fn route_button(
    id: impl Into<gpui::ElementId>,
    label: impl Into<gpui::SharedString>,
    route: Route,
    app: Entity<AppState>,
) -> gpui_component::button::Button {
    use gpui_component::button::{Button, ButtonVariants as _};
    Button::new(id).label(label).on_click(move |_, _, cx| {
        app.update(cx, |state, cx| state.push(route.clone(), cx));
    })
}

pub(crate) fn back_button(app: Entity<AppState>) -> gpui_component::button::Button {
    use gpui_component::{
        IconName, Sizable as _, StyledExt,
        button::{Button, ButtonVariants as _},
    };
    Button::new("page-back")
        .ghost()
        .xsmall()
        .icon(IconName::ArrowLeft)
        .label("Back")
        .on_click(move |_, _, cx| app.update(cx, |state, cx| state.back(cx)))
}

pub(crate) fn missing_resource(label: &str, app: Entity<AppState>) -> impl IntoElement {
    use gpui_component::{Sizable as _, StyledExt, button::Button, v_flex};
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_3()
        .child(div().text_lg().font_semibold().child("Not found"))
        .child(
            div()
                .text_sm()
                .child(format!("The {label} is no longer available.")),
        )
        .child(
            Button::new("missing-resource-back")
                .label("Back")
                .on_click(move |_, _, cx| app.update(cx, |state, cx| state.back(cx))),
        )
}

pub(crate) fn instance_exists(reference: &InstanceRef, app: &Entity<AppState>, cx: &App) -> bool {
    app.read(cx).instances.read(cx).find(reference).is_some()
}

pub(crate) fn crash_exists(reference: &CrashRef, app: &Entity<AppState>, cx: &App) -> bool {
    app.read(cx).crashes.read(cx).find(reference).is_some()
}
