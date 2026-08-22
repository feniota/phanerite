//! Phanerite launcher application shell and root presentation entity.

pub mod assets;
pub mod components;
pub mod pages;
pub mod palette;
pub mod route;
#[cfg(feature = "seed")]
pub mod seed;
pub mod state;
pub mod theme;

use gpui::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, Styled as _, Window, div, px,
};
use gpui_base::{h_resizable, resizable_panel};
use gpui_component::{ActiveTheme as _, Root, scroll::ScrollableElement, v_flex};

use crate::components::titlebar::TitleBar;
use crate::state::{
    AccountStore, AppState, CrashStore, InstanceStore, SessionStore, Settings, SettingsStore,
    StorageRegistry,
};

/// Root presentation entity. Launch and live-log entities remain outside this
/// view so the sidebar and status bar cannot observe high-frequency updates.
pub struct Phanerite {
    app: Entity<AppState>,
}

impl Phanerite {
    pub fn new(cx: &mut App) -> Self {
        Self::new_with_route(cx, None)
    }

    pub fn new_with_route(cx: &mut App, initial_route: Option<route::Route>) -> Self {
        #[cfg(feature = "seed")]
        let (instances, accounts, settings, crashes) = {
            let storage_id = route::StorageId::for_test(0);
            (
                seed::seed_instances(storage_id),
                seed::seed_accounts(),
                SettingsStore::new(Settings::default(), seed::seed_runtimes()),
                seed::seed_crash_reports(storage_id),
            )
        };
        #[cfg(not(feature = "seed"))]
        let (instances, accounts, settings, crashes) = (
            Vec::new(),
            Vec::new(),
            SettingsStore::new(Settings::default(), Vec::new()),
            Vec::new(),
        );

        let instances = cx.new(|_| InstanceStore::new(instances));
        instances.update(cx, |store, _| {
            store.set_storage_context(route::StorageId::for_test(0));
        });
        let accounts = cx.new(|_| AccountStore::new(accounts));
        let settings = cx.new(|_| settings);
        let crashes = cx.new(|_| CrashStore::new(crashes));
        let sessions = cx.new(|_| SessionStore::default());
        let app = cx.new(|cx| {
            AppState::new_with_route(
                StorageRegistry::new(),
                instances,
                accounts,
                settings,
                crashes,
                sessions,
                initial_route,
                cx,
            )
        });
        Self { app }
    }
}

impl Render for Phanerite {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let route = self.app.read(cx).route().clone();
        v_flex()
            .id("phanerite-root")
            .size_full()
            .bg(cx.theme().background)
            .child(
                TitleBar::new().child(
                    gpui_component::h_flex()
                        .items_center()
                        .gap_2()
                        .child(components::phanerite_app_icon::render())
                        .child(div().text_sm().child("Phanerite"))
                        .child(
                            div()
                                .font_family(crate::theme::MONO_FONT_FAMILY)
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("v0.1.0-pre"),
                        ),
                ),
            )
            .child(
                h_resizable("window-root-resizable")
                    .child(
                        resizable_panel()
                            .size(px(250.))
                            .size_range(px(240.)..px(400.))
                            .child(components::nav_sidebar::render(self.app.clone(), cx)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .overflow_y_scrollbar()
                            .child(pages::render(&route, self.app.clone(), window, cx))
                            .into_any_element(),
                    ),
            )
            .child(components::status_bar::render(self.app.clone(), cx))
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
