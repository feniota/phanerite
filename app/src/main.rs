mod assets;
mod components;

use assets::{Assets, PhaIcon};
use gpui::prelude::*;
use gpui::*;
use gpui_component::status_bar::StatusBar;
use gpui_component::{combobox::*, sidebar::*, *};

/// Entry point of the whole Phanerite app
pub struct Phanerite {
    appearance_subscription: Option<Subscription>,
}

impl Phanerite {
    fn new() -> Self {
        Self {
            appearance_subscription: None,
        }
    }
}

impl Render for Phanerite {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if self.appearance_subscription.is_none() {
            // Follow system dark mode changes
            let subscription = window.observe_window_appearance(|window, cx| {
                Theme::sync_system_appearance(Some(window), cx);
            });
            self.appearance_subscription = Some(subscription);
        }

        v_flex()
            .size_full()
            .child(TitleBar::new().child("Phanerite"))
            .child(
                h_flex()
                    .size_full()
                    .child(
                        Sidebar::new("main-sidebar")
                            .child(
                                SidebarGroup::new("LIBRARY").child(
                                    SidebarMenu::new()
                                        .child(
                                            SidebarMenuItem::new("Play")
                                                .icon(IconName::Play)
                                                .on_click(|_, _, _| println!("Play clicked")),
                                        )
                                        .child(
                                            SidebarMenuItem::new("Instances")
                                                .icon(PhaIcon::Layers)
                                                .on_click(|_, _, _| println!("Instances clicked")),
                                        ),
                                ),
                            )
                            .footer(SidebarFooter::new()),
                    )
                    .child(div()),
            )
            .child(
                StatusBar::new()
                    .left("Ready")
                    .child("README.md")
                    .right("UTF-8"),
            )
    }
}

fn main() {
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        // Follow the system light/dark mode
        Theme::sync_system_appearance(None, cx);
        cx.refresh_windows();

        cx.spawn(async move |cx| {
            let window_options = cx.update(|cx| WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(
                    gpui::Bounds::<Pixels>::centered(
                        None,
                        gpui::Size::new(Pixels::from(1200.0), Pixels::from(800.0)),
                        cx,
                    ),
                )),
                ..TitleBar::window_options()
            });
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|_| Phanerite::new());
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
