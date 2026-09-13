//! Native render and input regressions, without opening desktop windows.

use crate::{
    Phanerite,
    components::instance_create_dialog,
    route::{InstanceRef, Route},
    seed,
    state::*,
};
use gpui_kit as gpui;
use gpui_kit::{
    App, AppContext as _, Entity, KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, TestAppContext,
    VisualTestContext, point, px, size,
};

fn fixture(cx: &mut App) -> Entity<AppState> {
    let storage = seed::storage_ident(0);
    let instances = cx.new(|_| {
        let mut store = InstanceStore::new(seed::seed_instances(storage.clone()));
        store.set_storage_context(storage.clone());
        store
    });
    let accounts = cx.new(|_| seed::seed_account_store());
    let settings = cx.new(|_| SettingsStore::new(Settings::default(), seed::seed_runtimes()));
    let crashes = cx.new(|_| CrashStore::new(seed::seed_crash_reports(storage.clone())));
    let sessions = cx.new(|_| SessionStore::default());
    cx.new(|cx| {
        AppState::new(
            phanerite_core::storage::multi::MultiStorage::new(),
            Some(storage),
            instances,
            accounts,
            settings,
            crashes,
            sessions,
            cx,
        )
    })
}

fn open_window(cx: &mut TestAppContext) -> (Entity<AppState>, &mut VisualTestContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::theme::install("emerald", None, cx);
    });
    let app = cx.update(fixture);
    let app_view = app.clone();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let view = cx.new(|cx| {
            let subscription = cx.observe(&app_view, |_, _, cx| cx.notify());
            Phanerite {
                app: app_view,
                _subscription: subscription,
            }
        });
        crate::window::root(view, window, cx)
    });
    cx.simulate_resize(size(px(1200.), px(760.)));
    (app, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn click(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx.debug_bounds(selector).expect("control must be visible");
    cx.simulate_click(bounds.center(), Modifiers::default());
    draw(cx);
}

fn press(cx: &mut VisualTestContext, key: &str) {
    let keystroke = Keystroke::parse(key).unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
    draw(cx);
}

#[gpui::test]
fn sidebar_navigation_and_disclosure_are_independent(cx: &mut TestAppContext) {
    let (app, cx) = open_window(cx);
    draw(cx);
    for (navigation, toggle, leaf, route) in [
        (
            "sidebar-instances",
            "sidebar-instances-toggle",
            "sidebar-instance-inst-vanilla",
            Route::Instances,
        ),
        (
            "sidebar-aphanite",
            "sidebar-aphanite-toggle",
            "sidebar-instance-inst-neo",
            Route::Aphanite,
        ),
    ] {
        assert!(cx.debug_bounds(leaf).is_some(), "groups start expanded");
        click(cx, navigation);
        app.read_with(cx, |app, _| assert_eq!(app.route(), &route));
        assert!(
            cx.debug_bounds(leaf).is_some(),
            "navigation must not collapse its group"
        );

        click(cx, toggle);
        app.read_with(cx, |app, _| assert_eq!(app.route(), &route));
        assert!(cx.debug_bounds(leaf).is_none());
        assert!(cx.debug_bounds("sidebar-instance-inst-fog").is_some());

        app.update(cx, |app, cx| app.replace(Route::Play, cx));
        draw(cx);
        click(cx, navigation);
        app.read_with(cx, |app, _| assert_eq!(app.route(), &route));
        assert!(
            cx.debug_bounds(leaf).is_none(),
            "navigation must preserve a collapsed group"
        );

        click(cx, toggle);
        assert!(cx.debug_bounds(leaf).is_some());
        // Native buttons preserve pointer focus. Reach the disclosure through
        // the sidebar's tab order before testing keyboard activation.
        cx.update(|window, cx| {
            window.blur(cx);
            let stops = if route == Route::Instances { 5 } else { 8 };
            for _ in 0..stops {
                window.focus_next(cx);
            }
        });
        draw(cx);
        press(cx, "space");
        assert!(
            cx.debug_bounds(leaf).is_none(),
            "Space toggles the focused disclosure"
        );
        press(cx, "enter");
        assert!(
            cx.debug_bounds(leaf).is_some(),
            "Enter toggles the focused disclosure"
        );
        app.read_with(cx, |app, _| assert_eq!(app.route(), &route));
    }
}

#[gpui::test]
fn sidebar_rows_footer_and_play_avatar_keep_their_geometry(cx: &mut TestAppContext) {
    let (app, cx) = open_window(cx);
    for (width, font_size) in [(1200., 16.), (900., 18.)] {
        cx.simulate_resize(size(px(width), px(760.)));
        cx.update(|_, cx| {
            gpui_kit::component::Theme::global_mut(cx).font_size = px(font_size);
            crate::theme::sync_tokens(cx);
        });
        draw(cx);
        let play = cx.debug_bounds("sidebar-play").unwrap();
        for selector in [
            "sidebar-instance-inst-fog",
            "sidebar-instance-inst-vanilla",
            "sidebar-instance-inst-neo",
            "sidebar-account",
            "sidebar-settings",
        ] {
            let row = cx.debug_bounds(selector).unwrap();
            assert_eq!(row.left(), play.left(), "{selector} has an extra inset");
            assert_eq!(
                row.right(),
                play.right(),
                "{selector} has a different trailing edge"
            );
        }
        let account = cx.debug_bounds("sidebar-account").unwrap();
        let settings = cx.debug_bounds("sidebar-settings").unwrap();
        assert!(
            account.bottom() < settings.top(),
            "footer buttons must remain separate"
        );
        let avatar = cx
            .debug_bounds("play-account-avatar")
            .expect("active profile needs an avatar");
        let account_button = cx.debug_bounds("play-account").unwrap();
        assert_eq!(avatar.size.width, avatar.size.height);
        assert!(avatar.size.width > px(0.));
        assert!(avatar.top() >= account_button.top() && avatar.bottom() <= account_button.bottom());
    }
    click(cx, "sidebar-account");
    app.read_with(cx, |app, _| assert_eq!(app.route(), &Route::Play));
    cx.simulate_keystrokes("escape");
    draw(cx);
    click(cx, "sidebar-settings");
    app.read_with(cx, |app, _| assert_eq!(app.route(), &Route::Settings));
}

#[gpui::test]
fn routes_render_at_regular_and_large_text_sizes(cx: &mut TestAppContext) {
    let (app, cx) = open_window(cx);
    let reference = InstanceRef::new(seed::storage_ident(0), "inst-fog");
    let mut routes = vec![
        Route::Play,
        Route::Instances,
        Route::Aphanite,
        Route::Accounts,
        Route::Settings,
        Route::InstanceDetail(reference.clone()),
        Route::Mods(reference.clone()),
        Route::Packs(reference.clone()),
        Route::Shaders(reference.clone()),
        Route::Worlds(reference.clone()),
        Route::Logs(reference.clone()),
        Route::LaunchSettings(reference),
    ];
    routes.extend(
        seed::seed_crash_reports(seed::storage_ident(0))
            .into_iter()
            .map(|report| Route::Crash(report.reference())),
    );
    for (width, font_size) in [(1200., 16.), (900., 18.)] {
        cx.simulate_resize(size(px(width), px(760.)));
        cx.update(|_, cx| {
            gpui_kit::component::Theme::global_mut(cx).font_size = px(font_size);
            crate::theme::sync_tokens(cx);
        });
        for route in &routes {
            app.update(cx, |app, cx| app.replace(route.clone(), cx));
            draw(cx);
            if matches!(route, Route::Logs(_)) {
                let filters = cx
                    .debug_bounds("log-filters")
                    .expect("log filters should be rendered");
                let terminal = cx
                    .debug_bounds("log-terminal")
                    .expect("log terminal should be rendered");
                assert!(
                    filters.bottom() <= terminal.top(),
                    "log controls overlap output at width {width}"
                );
                assert!(
                    terminal.size.height > px(200.),
                    "filters must leave room for output"
                );
            }
        }
    }
}

#[gpui::test]
fn create_dialog_preserves_typed_name_across_renders_and_updates_the_instance_store(
    cx: &mut TestAppContext,
) {
    let (app, cx) = open_window(cx);
    draw(cx);
    cx.update(|window, cx| instance_create_dialog::open(window, cx, app.clone()));
    draw(cx);
    let name = cx
        .debug_bounds("create-instance-name")
        .expect("name field should be visible");
    cx.simulate_click(
        point(name.center().x, name.bottom() - px(12.)),
        Modifiers::default(),
    );
    cx.simulate_input("Survival");
    draw(cx);
    cx.simulate_input("2026");
    draw(cx);
    let actions = cx
        .debug_bounds("create-instance-actions")
        .expect("dialog actions should be visible");
    cx.simulate_click(
        point(actions.right() - px(40.), actions.center().y),
        Modifiers::default(),
    );
    draw(cx);
    app.read_with(cx, |app, cx| {
        let Route::InstanceDetail(reference) = app.route() else {
            panic!("creation should open the new instance");
        };
        let instance = app
            .instances
            .read(cx)
            .find(reference)
            .expect("created instance must exist");
        assert_eq!(instance.name, "Survival2026");
        assert_eq!(instance.mc_version, "1.21.4");
        assert_eq!(app.instances.read(cx).len(), 6);
    });
}
