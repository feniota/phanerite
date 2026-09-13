//! Global preferences, runtime management and appearance.

use super::{launch_settings as launch, page_shell, route_button, widgets::*};
use crate::{
    assets::PhaIcon,
    components::form::{self, Choice},
    route::Route,
    state::{
        AppState, FONT_SIZES, LANGUAGES, LaunchField, PreferenceFlag, Preferences, ProcessPriority,
    },
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, Selectable as _, StyledExt as _, Theme, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    notification::Notification,
    switch::Switch,
    v_flex,
};
use gpui_kit::{App, Div, Entity, IntoElement, ParentElement as _, Styled as _, Window, div, px};

fn preference_flag(label: &str, flag: PreferenceFlag, app: Entity<AppState>, cx: &App) -> Div {
    let checked = app.read(cx).settings.read(cx).preferences().flag(flag);
    h_flex()
        .min_h_8()
        .gap_4()
        .justify_between()
        .child(div().text_sm().child(label.to_owned()))
        .child(
            Switch::new(flag.key())
                .checked(checked)
                .accessibility_label(label.to_owned())
                .on_click(move |checked, _, cx| {
                    app.read(cx).settings.clone().update(cx, |store, cx| {
                        if store.set_flag(flag, *checked) {
                            cx.notify();
                        }
                    });
                }),
        )
}

fn preference_input(
    id: &'static str,
    label: &str,
    read: impl Fn(&Preferences) -> String + 'static,
    write: impl Fn(&mut Preferences, String) + 'static,
    app: Entity<AppState>,
    window: &mut Window,
    cx: &mut App,
) -> Div {
    field(
        label.to_owned(),
        form::input(
            id,
            app,
            move |app, cx| read(app.settings.read(cx).preferences()),
            move |app, value, cx| {
                app.settings.update(cx, |store, cx| {
                    if store.update_preferences(|prefs| write(prefs, value)) {
                        cx.notify();
                    }
                });
            },
            window,
            cx,
        )
        .w_full(),
    )
}

pub fn render(app: Entity<AppState>, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let runtimes = app.read(cx).settings.read(cx).runtimes().to_vec();
    let settings = app.read(cx).settings.read(cx).settings().clone();
    let title = header(
        heading(PhaIcon::Settings, "Settings", cx),
        "Global preferences. Instance-specific options live in each instance’s detail panel.",
        div(),
        cx,
    );
    let mut java = v_flex().gap_4().child(
        h_flex()
            .justify_between()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_medium()
                    .child("Available Java runtimes"),
            )
            .child(
                Button::new("runtime-add")
                    .icon(PhaIcon::Plus)
                    .label("Add custom…")
                    .on_click({
                        let app = app.clone();
                        move |_, window, cx| {
                            crate::components::runtime_dialog::open(window, cx, app.clone())
                        }
                    }),
            ),
    );
    java = java.child(v_flex().gap_2().children(runtimes.iter().map(|runtime| {
        h_flex()
            .gap_3()
            .p_3()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted.opacity(0.2))
            .child(
                Icon::new(PhaIcon::Cpu)
                    .size_5()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                v_flex()
                    .min_w_0()
                    .flex_1()
                    .gap_1()
                    .child(div().text_sm().font_medium().child(runtime.name.clone()))
                    .child(
                        muted(runtime.path.to_string_lossy().into_owned(), cx)
                            .text_xs()
                            .font_family(crate::theme::MONO_FONT_FAMILY)
                            .truncate(),
                    ),
            )
            .child(badge(format!("Java {}", runtime.version)))
            .children(runtime.managed.then(|| badge("Managed")))
    })));
    if runtimes.is_empty() {
        java = java.child(muted(
            "No Java runtimes configured. Add a Java executable to get started.",
            cx,
        ));
    }
    java = java
        .child(launch::memory(None, app.clone(), window, cx))
        .child(launch::text_field(
            "JVM arguments",
            None,
            LaunchField::JavaArgs,
            "",
            app.clone(),
            window,
            cx,
        ));
    let priority = form::select(
        "settings-priority",
        app.clone(),
        ProcessPriority::ALL
            .iter()
            .map(|priority| Choice::new(priority.label(), priority.label()))
            .collect(),
        |app, cx| {
            app.settings
                .read(cx)
                .preferences()
                .process_priority
                .label()
                .into()
        },
        |app, value, cx| {
            if let Some(priority) = ProcessPriority::ALL
                .into_iter()
                .find(|priority| priority.label() == value)
            {
                app.settings.update(cx, |store, cx| {
                    if store.update_preferences(|prefs| prefs.process_priority = priority) {
                        cx.notify();
                    }
                });
            }
        },
        window,
        cx,
    )
    .w_full()
    .accessibility_label("Process priority");
    let mut defaults = v_flex()
        .gap_4()
        .child(
            h_flex()
                .items_start()
                .gap_4()
                .child(launch::selector(
                    "Window mode",
                    None,
                    LaunchField::WindowMode,
                    app.clone(),
                    window,
                    cx,
                ))
                .child(field("Process priority", control_slot(priority))),
        )
        .child(
            h_flex()
                .items_start()
                .gap_4()
                .child(launch::text_field(
                    "Window width",
                    None,
                    LaunchField::WindowWidth,
                    "",
                    app.clone(),
                    window,
                    cx,
                ))
                .child(launch::text_field(
                    "Window height",
                    None,
                    LaunchField::WindowHeight,
                    "",
                    app.clone(),
                    window,
                    cx,
                )),
        );
    for (label, flag) in [
        ("Show game logs", PreferenceFlag::ShowGameLogs),
        ("Write debug logs", PreferenceFlag::DebugLogs),
        ("Generate game options", PreferenceFlag::GenerateGameOptions),
    ] {
        defaults = defaults.child(preference_flag(label, flag, app.clone(), cx));
    }
    for (label, flag) in [
        ("Use default JVM arguments", LaunchField::UseDefaultJvmArgs),
        (
            "Use optimized JVM arguments",
            LaunchField::UseOptimizedJvmArgs,
        ),
    ] {
        defaults = defaults.child(launch::flag(label, None, flag, app.clone(), cx));
    }
    defaults = defaults
        .child(launch::text_field(
            "Environment variables",
            None,
            LaunchField::EnvironmentVariables,
            "KEY=value pairs, separated by spaces.",
            app.clone(),
            window,
            cx,
        ))
        .child(
            h_flex()
                .items_start()
                .gap_3()
                .child(launch::text_field(
                    "Pre-launch command",
                    None,
                    LaunchField::PreLaunchCommand,
                    "",
                    app.clone(),
                    window,
                    cx,
                ))
                .child(launch::text_field(
                    "Command wrapper",
                    None,
                    LaunchField::CommandWrapper,
                    "",
                    app.clone(),
                    window,
                    cx,
                ))
                .child(launch::text_field(
                    "Post-exit command",
                    None,
                    LaunchField::PostExitCommand,
                    "",
                    app.clone(),
                    window,
                    cx,
                )),
        );
    let mut graphics = v_flex()
        .gap_4()
        .child(
            h_flex()
                .items_start()
                .gap_4()
                .child(preference_input(
                    "opengl-renderer",
                    "OpenGL renderer",
                    |prefs| prefs.opengl_renderer.clone(),
                    |prefs, value| prefs.opengl_renderer = value,
                    app.clone(),
                    window,
                    cx,
                ))
                .child(preference_input(
                    "vulkan-renderer",
                    "Vulkan renderer",
                    |prefs| prefs.vulkan_renderer.clone(),
                    |prefs, value| prefs.vulkan_renderer = value,
                    app.clone(),
                    window,
                    cx,
                )),
        )
        .child(preference_flag(
            "Prefer high-performance GPU",
            PreferenceFlag::PreferHighPerformanceGpu,
            app.clone(),
            cx,
        ))
        .child(launch::flag(
            "Use custom natives",
            None,
            LaunchField::UseCustomNatives,
            app.clone(),
            cx,
        ))
        .child(launch::flag(
            "Skip native patching",
            None,
            LaunchField::SkipNativePatching,
            app.clone(),
            cx,
        ));
    for (label, field) in [
        ("Native library path", LaunchField::NativesPath),
        ("GLFW path", LaunchField::GlfwPath),
        ("OpenAL path", LaunchField::OpenalPath),
    ] {
        graphics = graphics.child(launch::text_field(
            label,
            None,
            field,
            "",
            app.clone(),
            window,
            cx,
        ));
    }
    let launcher = v_flex().gap_2().children(
        [
            (
                "Close launcher after game starts",
                PreferenceFlag::CloseAfterLaunch,
            ),
            (
                "Minimize to tray while playing",
                PreferenceFlag::HideOnLaunch,
            ),
            (
                "Allow multiple instances at once",
                PreferenceFlag::MultiInstance,
            ),
            ("Check for updates on startup", PreferenceFlag::CheckUpdates),
        ]
        .into_iter()
        .map(|(label, flag)| preference_flag(label, flag, app.clone(), cx)),
    );
    let accents = h_flex()
        .gap_2()
        .children(crate::palette::ACCENTS.iter().map(|(name, ..)| {
            let name = *name;
            let app = app.clone();
            Button::new(format!("accent-{name}"))
                .ghost()
                .selected(settings.accent == name)
                .tooltip(format!("{name} accent"))
                .accessibility_label(format!("{name} accent"))
                .child(
                    div()
                        .size_5()
                        .rounded_full()
                        .bg(crate::theme::accent_swatch(name)),
                )
                .on_click(move |_, window, cx| {
                    let changed = app.read(cx).settings.clone().update(cx, |store, cx| {
                        let changed = store.set_accent(name);
                        if changed {
                            cx.notify();
                        }
                        changed
                    });
                    if changed {
                        let size = Theme::global(cx).font_size;
                        crate::theme::install(name, Some(window), cx);
                        Theme::global_mut(cx).font_size = size;
                        crate::theme::sync_tokens(cx);
                        window.refresh();
                    }
                })
        }));
    let language = form::select(
        "settings-language",
        app.clone(),
        LANGUAGES
            .iter()
            .map(|(key, label)| Choice::new(*key, *label))
            .collect(),
        |app, cx| app.settings.read(cx).preferences().language.clone(),
        |app, value, cx| {
            app.settings.update(cx, |store, cx| {
                if store.update_preferences(|prefs| prefs.language = value) {
                    cx.notify();
                }
            });
        },
        window,
        cx,
    )
    .w_full()
    .accessibility_label("Language");
    let font = form::select(
        "settings-font",
        app.clone(),
        FONT_SIZES
            .iter()
            .map(|(key, label)| Choice::new(*key, *label))
            .collect(),
        |app, cx| app.settings.read(cx).preferences().font_size.clone(),
        |app, value, cx| {
            let size = match value.as_str() {
                "sm" => 14.,
                "lg" => 18.,
                _ => 16.,
            };
            app.settings.update(cx, |store, cx| {
                if store.update_preferences(|prefs| prefs.font_size = value) {
                    cx.notify();
                }
            });
            // This pixel value anchors the rem scale for the entire window.
            Theme::global_mut(cx).font_size = px(size);
            crate::theme::sync_tokens(cx);
            cx.refresh_windows();
        },
        window,
        cx,
    )
    .w_full()
    .accessibility_label("UI font size");
    let appearance = v_flex()
        .gap_4()
        .child(field("Accent color", accents))
        .child(muted("Switches the primary color across the whole UI.", cx).text_xs())
        .child(
            h_flex()
                .items_start()
                .gap_4()
                .child(field("Language", control_slot(language)))
                .child(field("UI font size", control_slot(font))),
        );
    let downloads = form::slider(
        "settings-downloads",
        app.clone(),
        |app, cx| app.settings.read(cx).preferences().download_threads,
        |app, value, cx| {
            app.settings.update(cx, |store, cx| {
                if store.update_preferences(|prefs| prefs.download_threads = value) {
                    cx.notify();
                }
            });
        },
        window,
        cx,
    );
    let network = v_flex()
        .gap_4()
        .child(field(
            format!(
                "Concurrent downloads · {}",
                settings.preferences.download_threads
            ),
            downloads,
        ))
        .child(preference_input(
            "connection-timeout",
            "Connection timeout (seconds)",
            |prefs| prefs.connection_timeout.to_string(),
            |prefs, value| {
                if let Ok(value @ 1..=600) = value.parse() {
                    prefs.connection_timeout = value;
                }
            },
            app.clone(),
            window,
            cx,
        ))
        .child(muted("Seconds before a mirror connection is retried.", cx).text_xs());
    let configurations = app.read(cx).instances.read(cx).aphanite().count();
    let installed = app
        .read(cx)
        .instances
        .read(cx)
        .aphanite()
        .filter(|instance| instance.last_played.is_some())
        .count();
    let aphanite = v_flex()
        .gap_3()
        .child(preference_input(
            "aphanite-server",
            "Server URL",
            |prefs| prefs.aphanite_server.clone(),
            |prefs, value| prefs.aphanite_server = value,
            app.clone(),
            window,
            cx,
        ))
        .child(
            h_flex()
                .gap_3()
                .justify_between()
                .child(
                    muted(
                        format!("Contains {configurations} configurations, {installed} installed."),
                        cx,
                    )
                    .text_xs(),
                )
                .child(
                    route_button(
                        "settings-configurations",
                        "View configurations",
                        Route::Aphanite,
                        app,
                    )
                    .ghost(),
                ),
        );
    let about = h_flex()
        .gap_4()
        .justify_between()
        .child(
            h_flex()
                .gap_6()
                .child(
                    v_flex()
                        .gap_1()
                        .child(muted("Version", cx).text_xs())
                        .child(div().text_sm().child(env!("CARGO_PKG_VERSION"))),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(muted("License", cx).text_xs())
                        .child(div().text_sm().child("GPL-3.0-or-later")),
                )
                .child(badge("Rust · GPUI Kit")),
        )
        .child(
            Button::new("settings-updates")
                .icon(PhaIcon::RefreshCw)
                .label("Check for updates")
                .on_click(|_, window, cx| {
                    window.push_notification(
                        Notification::info("You are using a development build of Phanerite."),
                        cx,
                    )
                }),
        );
    let content = v_flex()
        .gap_4()
        .child(section(
            "Java & runtime",
            "Phanerite manages Java runtimes for your instances.",
            java,
            cx,
        ))
        .child(section(
            "Game launch defaults",
            "Default values inherited by every instance.",
            defaults,
            cx,
        ))
        .child(section("Graphics & native libraries", "", graphics, cx))
        .child(section(
            "Launcher",
            "Behaviour around launching.",
            launcher,
            cx,
        ))
        .child(section(
            "Appearance",
            "Dark surfaces with a configurable accent and text scale.",
            appearance,
            cx,
        ))
        .child(section(
            "Network",
            "Download behaviour for assets and libraries.",
            network,
            cx,
        ))
        .child(section(
            "Connected Aphanite server",
            "Fetch server-managed modpack configurations.",
            aphanite,
            cx,
        ))
        .child(section("About", "", about, cx));
    page_shell(Some(title), content, cx)
}
