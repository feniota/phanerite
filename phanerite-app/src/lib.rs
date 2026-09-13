//! Phanerite launcher application shell and root presentation entity.

#![warn(clippy::unused_trait_names)]

mod app_metadata;
pub mod assets;
pub mod components;
pub mod db;
pub mod pages;
pub mod palette;
pub mod route;
#[cfg(feature = "seed")]
pub mod seed;
pub mod state;
pub mod theme;
pub mod utils;
pub mod window;

#[cfg(all(test, feature = "seed"))]
mod ui_tests;

pub use app_metadata::{APP_ID, APP_NAME, APP_WINDOW_TITLE};
pub(crate) use gpui_kit::block_on;

use gpui_kit::base::{h_resizable, resizable_panel};
use gpui_kit::component::{ActiveTheme as _, Root, v_flex};
use gpui_kit::{
    AppContext as _, Context, Entity, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use phanerite_core::storage::{Storage, StorageIdent, multi::MultiStorage};

use crate::components::titlebar::TitleBar;
use crate::state::{
    AccountStore, AppState, CrashStore, InstanceStore, SessionStore, Settings, SettingsStore,
};

/// Root presentation entity. Launch and live-log entities remain outside this
/// view so the sidebar and status bar cannot observe high-frequency updates.
pub struct Phanerite {
    app: Entity<AppState>,
    _subscription: gpui_kit::Subscription,
}

impl Phanerite {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self::new_with_route(cx, None)
    }

    pub fn new_with_route(cx: &mut Context<Self>, initial_route: Option<route::Route>) -> Self {
        #[cfg(feature = "seed")]
        let (instances, accounts, settings, crashes) = {
            let storage = seed::storage_ident(0);
            (
                seed::seed_instances(storage.clone()),
                seed::seed_account_store(),
                SettingsStore::new(Settings::default(), seed::seed_runtimes()),
                seed::seed_crash_reports(storage.clone()),
            )
        };
        #[cfg(not(feature = "seed"))]
        let (instances, accounts, settings, crashes) = (
            Vec::new(),
            AccountStore::default(),
            SettingsStore::new(Settings::default(), Vec::new()),
            Vec::new(),
        );

        let storage: Option<(StorageIdent, Storage)> = {
            #[cfg(feature = "seed")]
            {
                // FIXME: Move startup storage construction into the real
                // asynchronous application initialization flow.
                let root = dirs::data_dir()
                    .unwrap_or_else(std::env::temp_dir)
                    .join("phanerite");
                let storage =
                    block_on(Storage::new(&root)).expect("failed to initialize Phanerite storage");
                Some((StorageIdent::from(&storage), storage))
            }
            #[cfg(not(feature = "seed"))]
            {
                None
            }
        };
        let (storage_key, storages) = match storage {
            Some((key, storage)) => {
                let storages = MultiStorage::new();
                block_on(storages.insert(key.clone(), storage))
                    .expect("failed to register Phanerite storage");
                (Some(key), storages)
            }
            None => (None, MultiStorage::new()),
        };
        // Seeded resources must share the selected storage identity: store
        // mutations deliberately reject results from a different storage.
        #[cfg(feature = "seed")]
        let (instances, crashes, initial_route) = {
            let mut instances = instances;
            let mut crashes = crashes;
            let mut initial_route = initial_route;
            if let Some(storage) = storage_key.as_ref() {
                for instance in &mut instances {
                    instance.storage = storage.clone();
                }
                for report in &mut crashes {
                    report.storage = storage.clone();
                }
                match initial_route.as_mut() {
                    Some(
                        route::Route::InstanceDetail(reference)
                        | route::Route::Mods(reference)
                        | route::Route::Packs(reference)
                        | route::Route::Shaders(reference)
                        | route::Route::Worlds(reference)
                        | route::Route::Logs(reference)
                        | route::Route::LaunchSettings(reference),
                    ) => reference.storage = storage.clone(),
                    Some(route::Route::Crash(reference)) => reference.storage = storage.clone(),
                    _ => {}
                }
            }
            (instances, crashes, initial_route)
        };
        let instances = cx.new(|_| InstanceStore::new(instances));
        instances.update(cx, |store, _| {
            if let Some(storage) = storage_key.clone() {
                store.set_storage_context(storage);
            }
        });
        let accounts: Entity<AccountStore> = cx.new(|_| accounts);
        #[cfg(feature = "seed")]
        accounts.update(cx, |store, _| {
            let mut accounts = store.all();
            accounts.sort_by(|left, right| left.username.cmp(&right.username));
            if let Some(account) = accounts.first() {
                store.set_active(&account.id);
            }
        });
        let settings = cx.new(|_| settings);
        let crashes = cx.new(|_| CrashStore::new(crashes));
        let sessions = cx.new(|_| SessionStore::default());
        let app = cx.new(|cx| {
            AppState::new_with_route(
                storages,
                storage_key,
                instances,
                accounts,
                settings,
                crashes,
                sessions,
                initial_route,
                cx,
            )
        });
        let _subscription = cx.observe(&app, |_, _, cx| cx.notify());
        Self { app, _subscription }
    }
}

impl Render for Phanerite {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let route = self.app.read(cx).route().clone();
        let radii = crate::window::content_radii(window);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let has_overlay = dialog_layer.is_some() || sheet_layer.is_some();
        let content = v_flex()
            .id("phanerite-root")
            .relative()
            .size_full()
            .bg(cx.theme().background)
            .map(|content| crate::window::round_corners(content, radii))
            .child(
                TitleBar::new().child(
                    gpui_kit::component::h_flex()
                        .items_center()
                        .gap_2()
                        .child(assets::phanerite_logo().size(px(18.)))
                        .child(div().text_sm().child(APP_WINDOW_TITLE)),
                ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    h_resizable("window-root-resizable")
                        .child(
                            resizable_panel()
                                // ResizablePanel accepts resolved pixels only.
                                .size(window.rem_size() * 15.625)
                                .size_range(window.rem_size() * 14. ..window.rem_size() * 25.)
                                .child(components::nav_sidebar::render(self.app.clone(), cx)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .min_h_0()
                                .overflow_hidden()
                                .child(pages::render(&route, self.app.clone(), window, cx))
                                .into_any_element(),
                        ),
                ),
            )
            .child(components::status_bar::render(self.app.clone(), window, cx))
            .when(has_overlay, |content| {
                content.child(crate::window::round_corners(
                    div().absolute().inset_0().bg(theme::window_overlay()),
                    radii,
                ))
            })
            .children(dialog_layer)
            .children(sheet_layer)
            .children(Root::render_notification_layer(window, cx));

        crate::window::frame(content, window, cx)
    }
}
