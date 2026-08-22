//! Native application entry point that initializes GPUI and opens the main window.

use gpui::*;
use gpui_component::{Root, TitleBar};
use phanerite::Phanerite;
use phanerite::assets::Assets;

fn main() {
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        let load_font = |path| {
            cx.asset_source()
                .load(path)
                .expect("failed to load bundled font")
                .expect("bundled font is missing")
        };
        cx.text_system()
            .add_fonts(vec![
                load_font("fonts/SarasaAdwaitaUiSC-Regular.ttf.zst"),
                load_font("fonts/AdwaitaMono-Regular.ttf.zst"),
            ])
            .expect("failed to register bundled fonts");

        gpui_component::init(cx);
        phanerite::theme::install("emerald", None, cx);
        cx.refresh_windows();

        cx.spawn(async move |cx| {
            let window_options = cx.update(|cx| WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(
                    gpui::Bounds::<Pixels>::centered(
                        None,
                        gpui::Size::new(Pixels::from(1200.0), Pixels::from(760.0)),
                        cx,
                    ),
                )),
                ..TitleBar::window_options()
            });
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| Phanerite::new(cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
