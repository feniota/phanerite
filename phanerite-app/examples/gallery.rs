//! Standalone gallery window for previewing the application UI with seeded data.

use gpui_kit::component::{Root, TitleBar};
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
}

fn main() {
    if std::env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }
    let route = route_from_args();
    let app = gpui_kit::application().with_assets(Assets);
    app.run(move |cx| {
        gpui_kit::init(cx);
        phanerite::theme::install("emerald", None, cx);
        cx.spawn(async move |cx| {
            let options = cx.update(|cx| WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    Size::new(Pixels::from(1200.0), Pixels::from(760.0)),
                    cx,
                ))),
                ..TitleBar::window_options()
            });
            cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| Phanerite::new_with_route(cx, Some(route.clone())));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open gallery window");
        })
        .detach();
    });
}
