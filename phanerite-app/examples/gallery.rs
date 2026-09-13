//! Standalone gallery window for previewing the application UI with seeded data.

use gpui_kit::*;
use phanerite::{
    Phanerite,
    assets::Assets,
    route::{CrashRef, InstanceRef, Route},
};

fn argument(name: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(value) = args.next() {
        if value == name {
            return args.next();
        }
    }
    None
}

fn route_from_args() -> Route {
    let page = argument("--page").unwrap_or_else(|| "play".into());
    let storage = phanerite::seed::storage_ident(0);
    let instance = argument("--instance").unwrap_or_else(|| "inst-fog".into());
    let instance_ref = || InstanceRef::new(storage.clone(), instance.clone());
    match page.as_str() {
        "setup" => Route::Setup,
        "play" => Route::Play,
        "instances" => Route::Instances,
        "aphanite" => Route::Aphanite,
        "instance-detail" => Route::InstanceDetail(instance_ref()),
        "mods" => Route::Mods(instance_ref()),
        "packs" => Route::Packs(instance_ref()),
        "shaders" => Route::Shaders(instance_ref()),
        "worlds" => Route::Worlds(instance_ref()),
        "logs" => Route::Logs(instance_ref()),
        "launch-settings" => Route::LaunchSettings(instance_ref()),
        "crash" => Route::Crash(CrashRef::new(
            storage,
            argument("--report").unwrap_or_else(|| "crash-sodium-optifine".into()),
        )),
        "accounts" => Route::Accounts,
        "settings" => Route::Settings,
        other => {
            eprintln!("Unknown page '{other}'. Use --help for the supported page names.");
            Route::Play
        }
    }
}

fn print_help() {
    println!("Phanerite gallery");
    println!(
        "  --page <name>       setup, play, instances, aphanite, instance-detail, mods, packs,"
    );
    println!(
        "                      shaders, worlds, logs, launch-settings, crash, accounts, settings"
    );
    println!("  --instance <id>     instance id for instance-scoped pages (default: inst-fog)");
    println!("  --report <id>       crash report id for --page crash");
    println!("  --width <px>        window width (default: 1200)");
    println!("  --height <px>       window height (default: 760)");
    println!("  --font-size <px>    base UI text size (default: 16)");
    println!("  --interact <steps>  preview input, e.g. 'click:1100:80;key:tab;type:Survival'");
}

fn main() {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }
    let route = route_from_args();
    let interactions = argument("--interact");
    let width = argument("--width")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(1200.)
        .max(900.);
    let height = argument("--height")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(760.)
        .max(600.);
    let font_size = argument("--font-size")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(16.)
        .clamp(14., 18.);
    let app = gpui_kit::application().with_assets(Assets);
    app.run(move |cx| {
        cx.set_app_identity(phanerite::APP_ID, phanerite::APP_NAME);
        phanerite::assets::load_fonts(cx).expect("failed to register bundled fonts");
        gpui_kit::init(cx);
        phanerite::theme::install("emerald", None, cx);
        gpui_kit::component::Theme::global_mut(cx).font_size = px(font_size);
        phanerite::theme::sync_tokens(cx);
        cx.spawn(async move |cx| {
            let options = cx.update(|cx| WindowOptions {
                window_min_size: Some(size(px(900.), px(600.))),
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    Size::new(px(width), px(height)),
                    cx,
                ))),
                ..phanerite::window::options()
            });
            cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| Phanerite::new_with_route(cx, Some(route.clone())));
                if let Some(steps) = interactions {
                    // Dispatch only inside this gallery window. This makes
                    // modal and input states reproducible in native reviews.
                    window
                        .spawn(cx, async move |cx| {
                            for step in steps.split(';').map(str::trim) {
                                cx.background_executor()
                                    .timer(std::time::Duration::from_millis(600))
                                    .await;
                                if cx
                                    .update(|window, cx| {
                                        if let Some(coordinates) = step.strip_prefix("click:") {
                                            let mut coordinates = coordinates
                                                .split(':')
                                                .filter_map(|value| value.parse::<f32>().ok());
                                            if let (Some(x), Some(y)) =
                                                (coordinates.next(), coordinates.next())
                                            {
                                                let position = point(px(x), px(y));
                                                window.dispatch_event(
                                                    PlatformInput::MouseDown(MouseDownEvent {
                                                        position,
                                                        button: MouseButton::Left,
                                                        click_count: 1,
                                                        modifiers: Modifiers::default(),
                                                        first_mouse: false,
                                                    }),
                                                    cx,
                                                );
                                                window.dispatch_event(
                                                    PlatformInput::MouseUp(MouseUpEvent {
                                                        position,
                                                        button: MouseButton::Left,
                                                        click_count: 1,
                                                        modifiers: Modifiers::default(),
                                                    }),
                                                    cx,
                                                );
                                            }
                                        } else if let Some(key) = step.strip_prefix("key:") {
                                            if let Ok(key) = Keystroke::parse(key) {
                                                window.dispatch_keystroke(key, cx);
                                            }
                                        } else if let Some(text) = step.strip_prefix("type:") {
                                            for character in text.chars() {
                                                let key = if character == ' ' {
                                                    "space".into()
                                                } else {
                                                    character.to_string()
                                                };
                                                if let Ok(key) = Keystroke::parse(&key) {
                                                    window.dispatch_keystroke(key, cx);
                                                }
                                            }
                                        }
                                    })
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        })
                        .detach();
                }
                cx.new(|cx| phanerite::window::root(view, window, cx))
            })
            .expect("Failed to open gallery window");
        })
        .detach();
    });
}
