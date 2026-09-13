//! Native application entry point that initializes GPUI and opens the main window.

use gpui_kit::*;
use phanerite::Phanerite;
use phanerite::assets::Assets;

fn main() {
    let app = gpui_kit::application().with_assets(Assets);

    app.run(move |cx| {
        cx.set_app_identity(phanerite::APP_ID, phanerite::APP_NAME);
        phanerite::assets::load_fonts(cx).expect("failed to register bundled fonts");

        gpui_kit::init(cx);
        phanerite::theme::install("emerald", None, cx);
        cx.refresh_windows();

        cx.spawn(async move |cx| {
            let window_options = cx.update(|cx| WindowOptions {
                window_min_size: Some(size(px(900.), px(600.))),
                window_bounds: Some(gpui_kit::WindowBounds::Windowed(
                    gpui_kit::Bounds::<Pixels>::centered(
                        None,
                        gpui_kit::Size::new(Pixels::from(1200.0), Pixels::from(760.0)),
                        cx,
                    ),
                )),
                ..phanerite::window::options()
            });
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(Phanerite::new);
                cx.new(|cx| phanerite::window::root(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
