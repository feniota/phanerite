//! Window identity and chrome shared by the application and gallery.

use gpui_kit::component::{ActiveTheme as _, Root};
use gpui_kit::{
    AnyElement, AnyView, App, Bounds, Context, Corners, CursorStyle, Decorations, Edges,
    InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Pixels, ResizeEdge,
    Size, Styled as _, Tiling, TitlebarOptions, Window, WindowOptions, div, point,
    prelude::FluentBuilder as _, px, size, transparent_black,
};

use crate::{components::titlebar::TitleBar, theme};

// These are platform geometry, independent of the application's text zoom. Keep
// the inset stable across maximize/restore, as GPUI uses it to restore the size.
const SHADOW_INSET: Pixels = px(20.);
const BORDER_WIDTH: Pixels = px(1.);
const RESIZE_HIT_SIZE: Pixels = px(4.);

/// Window options for Phanerite's title bar and client-side frame.
pub fn options() -> WindowOptions {
    WindowOptions {
        app_id: Some(crate::APP_ID.to_owned()),
        titlebar: Some(TitlebarOptions {
            title: Some(crate::APP_WINDOW_TITLE.into()),
            ..TitleBar::title_bar_options()
        }),
        // GPUI sends this bitmap to X11. Wayland resolves the icon through the
        // desktop entry whose filename matches APP_ID.
        #[cfg(target_os = "linux")]
        icon: Some(crate::assets::window_icon()),
        #[cfg(target_os = "linux")]
        window_background: gpui_kit::WindowBackgroundAppearance::Transparent,
        #[cfg(target_os = "linux")]
        window_decorations: Some(gpui_kit::WindowDecorations::Client),
        ..TitleBar::window_options()
    }
}

/// Keeps the component root's focus and overlay services around our own frame.
pub fn root(view: impl Into<AnyView>, window: &mut Window, cx: &mut Context<Root>) -> Root {
    // GPUI Kit 0.6's generic border and root background are rectangular. Both
    // must give way to the application frame for its corners to stay transparent.
    Root::new(view, window, cx)
        .bordered(false)
        .bg(transparent_black())
}

fn client_tiling(window: &Window) -> Option<Tiling> {
    match window.window_decorations() {
        Decorations::Server => None,
        Decorations::Client { tiling } => {
            Some(if window.is_fullscreen() || window.is_maximized() {
                Tiling::tiled()
            } else {
                tiling
            })
        }
    }
}

fn edge_sizes(tiling: Tiling, width: Pixels) -> Edges<Pixels> {
    Edges {
        top: if tiling.top { px(0.) } else { width },
        right: if tiling.right { px(0.) } else { width },
        bottom: if tiling.bottom { px(0.) } else { width },
        left: if tiling.left { px(0.) } else { width },
    }
}

fn corner_radii(tiling: Tiling, radius: Pixels) -> Corners<Pixels> {
    Corners {
        top_left: if tiling.top || tiling.left {
            px(0.)
        } else {
            radius
        },
        top_right: if tiling.top || tiling.right {
            px(0.)
        } else {
            radius
        },
        bottom_left: if tiling.bottom || tiling.left {
            px(0.)
        } else {
            radius
        },
        bottom_right: if tiling.bottom || tiling.right {
            px(0.)
        } else {
            radius
        },
    }
}

pub(crate) fn content_radii(window: &Window) -> Corners<Pixels> {
    client_tiling(window)
        .map(|tiling| corner_radii(tiling, theme::WINDOW_RADIUS - BORDER_WIDTH))
        .unwrap_or_default()
}

pub(crate) fn round_corners<T: gpui_kit::Styled>(element: T, radii: Corners<Pixels>) -> T {
    element
        .rounded_tl(radii.top_left)
        .rounded_tr(radii.top_right)
        .rounded_bl(radii.bottom_left)
        .rounded_br(radii.bottom_right)
}

pub(crate) fn frame(content: impl IntoElement, window: &mut Window, cx: &App) -> AnyElement {
    let Some(tiling) = client_tiling(window) else {
        return content.into_any_element();
    };
    window.set_client_inset(SHADOW_INSET);
    let insets = edge_sizes(tiling, SHADOW_INSET);
    let borders = edge_sizes(tiling, BORDER_WIDTH);
    let window_size = window.viewport_size();

    div()
        .id("window-frame")
        .relative()
        .flex()
        .flex_col()
        .size_full()
        .pt(insets.top)
        .pr(insets.right)
        .pb(insets.bottom)
        .pl(insets.left)
        .child(
            div()
                .relative()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .bg(cx.theme().background)
                .border_color(cx.theme().window_border)
                .border_t(borders.top)
                .border_r(borders.right)
                .border_b(borders.bottom)
                .border_l(borders.left)
                .map(|frame| round_corners(frame, corner_radii(tiling, theme::WINDOW_RADIUS)))
                .when(!tiling.is_tiled(), |frame| {
                    frame.shadow(theme::window_shadows(window.is_window_active()))
                })
                .child(content),
        )
        .when(window.is_resizable(), |frame| {
            frame.child(
                div().absolute().top_0().left_0().size_full().children(
                    resize_regions(window_size, tiling)
                        .into_iter()
                        .map(|(edge, bounds)| {
                            div()
                                .absolute()
                                .left(bounds.origin.x)
                                .top(bounds.origin.y)
                                .w(bounds.size.width)
                                .h(bounds.size.height)
                                .cursor(resize_cursor(edge))
                                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                    window.start_window_resize(edge);
                                })
                        }),
                ),
            )
        })
        .into_any_element()
}

fn resize_cursor(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}

fn resize_regions(window_size: Size<Pixels>, tiling: Tiling) -> Vec<(ResizeEdge, Bounds<Pixels>)> {
    let insets = edge_sizes(tiling, SHADOW_INSET);
    let left = insets.left - RESIZE_HIT_SIZE;
    let top = insets.top - RESIZE_HIT_SIZE;
    let right = window_size.width - insets.right - RESIZE_HIT_SIZE;
    let bottom = window_size.height - insets.bottom - RESIZE_HIT_SIZE;
    let band = RESIZE_HIT_SIZE * 2.;
    let width = right - left + band;
    let height = bottom - top + band;
    let mut regions = Vec::new();
    let mut add = |edge, x, y, width, height| {
        regions.push((edge, Bounds::new(point(x, y), size(width, height))));
    };

    if !tiling.top {
        add(ResizeEdge::Top, left, top, width, band);
    }
    if !tiling.bottom {
        add(ResizeEdge::Bottom, left, bottom, width, band);
    }
    if !tiling.left {
        add(ResizeEdge::Left, left, top, band, height);
    }
    if !tiling.right {
        add(ResizeEdge::Right, right, top, band, height);
    }
    // Corners paint last so diagonal resizing wins over the adjacent strips.
    if !tiling.top && !tiling.left {
        add(ResizeEdge::TopLeft, left, top, band, band);
    }
    if !tiling.top && !tiling.right {
        add(ResizeEdge::TopRight, right, top, band, band);
    }
    if !tiling.bottom && !tiling.left {
        add(ResizeEdge::BottomLeft, left, bottom, band, band);
    }
    if !tiling.bottom && !tiling.right {
        add(ResizeEdge::BottomRight, right, bottom, band, band);
    }
    regions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiled_edges_remove_only_their_adjacent_corners() {
        let radius = theme::WINDOW_RADIUS;
        assert_eq!(
            corner_radii(Tiling::default(), radius),
            Corners::all(radius)
        );
        assert_eq!(corner_radii(Tiling::tiled(), radius), Corners::default());
        for tiling in [
            Tiling {
                left: true,
                ..Tiling::default()
            },
            Tiling {
                top: true,
                ..Tiling::default()
            },
        ] {
            let corners = corner_radii(tiling, radius);
            assert_eq!(corners.top_left, px(0.));
            assert_eq!(corners.bottom_right, radius);
            assert_eq!(corners.top_right, if tiling.top { px(0.) } else { radius });
            assert_eq!(
                corners.bottom_left,
                if tiling.left { px(0.) } else { radius }
            );
        }
    }

    #[test]
    fn resize_hits_follow_the_visible_frame_and_leave_the_shadow_alone() {
        let regions = resize_regions(size(px(1200.), px(760.)), Tiling::default());
        let hit = |x, y| {
            regions
                .iter()
                .rev()
                .find_map(|(edge, bounds)| bounds.contains(&point(px(x), px(y))).then_some(*edge))
        };
        assert_eq!(hit(20., 20.), Some(ResizeEdge::TopLeft));
        assert_eq!(hit(1180., 20.), Some(ResizeEdge::TopRight));
        assert_eq!(hit(20., 740.), Some(ResizeEdge::BottomLeft));
        assert_eq!(hit(1180., 740.), Some(ResizeEdge::BottomRight));
        assert_eq!(hit(600., 20.), Some(ResizeEdge::Top));
        assert_eq!(hit(1180., 380.), Some(ResizeEdge::Right));
        assert_eq!(hit(5., 380.), None);
        assert_eq!(hit(600., 50.), None);
    }

    #[test]
    fn maximized_frames_have_no_insets_or_resize_regions() {
        let tiling = Tiling::tiled();
        assert_eq!(edge_sizes(tiling, SHADOW_INSET), Edges::all(px(0.)));
        assert!(resize_regions(size(px(1200.), px(760.)), tiling).is_empty());
        let regions = resize_regions(
            size(px(900.), px(760.)),
            Tiling {
                left: true,
                ..Tiling::default()
            },
        );
        assert!(regions.iter().all(|(edge, _)| !matches!(
            edge,
            ResizeEdge::Left | ResizeEdge::TopLeft | ResizeEdge::BottomLeft
        )));
    }
}
