#![recursion_limit = "256"]

use std::{cell::Cell, rc::Rc};

use phanerite_executor::event::On;
use phanerite_executor::tao_event::lifecycle::RedrawRequested;
use phanerite_executor::tao_event::window::CloseRequested;
use phanerite_executor::window::{Window, WindowRender, WindowRenderOptions};
use phanerite_executor::{Application, Shutdown};
use phanerite_layout::{LayoutNodeKind, LayoutTree, taffy::prelude::*};
use phanerite_render::{LayoutDebugRenderer, RenderContent, RenderViewport};

struct LayoutDemo {
    tree: LayoutTree,
    renderer: LayoutDebugRenderer,
    viewport: Option<RenderViewport>,
    prepared: Rc<Cell<Option<RenderViewport>>>,
}

impl RenderContent for LayoutDemo {
    fn configure(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        self.renderer.configure(device, queue, config);
    }

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, viewport: RenderViewport) {
        let [width, height] = viewport.logical_size();
        self.tree.resize(width, height).unwrap();
        if self.viewport != Some(viewport) || self.tree.needs_layout().unwrap() {
            let snapshot = self.tree.compute_layout().unwrap();
            self.renderer
                .update(device, queue, viewport, &snapshot)
                .unwrap();
            self.viewport = Some(viewport);
            self.prepared.set(Some(viewport));
        }
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.renderer.render(device, queue, target, encoder);
    }
}

fn demo_tree() -> LayoutTree {
    let mut tree: LayoutTree = LayoutTree::new(1000.0, 700.0).unwrap();
    let root = tree.root_node();
    tree.set_style(
        root,
        Style {
            flex_direction: FlexDirection::Column,
            padding: Rect::length(24.25),
            gap: Size::length(18.5),
            ..Style::default()
        },
    )
    .unwrap();
    let header = tree
        .create_node(
            LayoutNodeKind::Box,
            Style {
                size: Size {
                    width: percent(1.0),
                    height: length(80.5),
                },
                flex_shrink: 0.0,
                padding: Rect::length(12.25),
                gap: Size::length(10.5),
                ..Style::default()
            },
            Some(root),
        )
        .unwrap();
    for width in [0.2, 0.35, 0.15] {
        tree.create_node(
            LayoutNodeKind::Box,
            Style {
                size: Size {
                    width: percent(width),
                    height: percent(1.0),
                },
                ..Style::default()
            },
            Some(header),
        )
        .unwrap();
    }
    let body = tree
        .create_node(
            LayoutNodeKind::Box,
            Style {
                flex_grow: 1.0,
                min_size: Size::length(0.0),
                gap: Size::length(18.5),
                ..Style::default()
            },
            Some(root),
        )
        .unwrap();
    let sidebar = tree
        .create_node(
            LayoutNodeKind::Box,
            Style {
                size: Size {
                    width: percent(0.25),
                    height: percent(1.0),
                },
                flex_direction: FlexDirection::Column,
                padding: Rect::length(12.25),
                gap: Size::length(10.5),
                ..Style::default()
            },
            Some(body),
        )
        .unwrap();
    for height in [45.25, 70.5, 100.75] {
        tree.create_node(
            LayoutNodeKind::Box,
            Style {
                size: Size {
                    width: percent(1.0),
                    height: length(height),
                },
                ..Style::default()
            },
            Some(sidebar),
        )
        .unwrap();
    }
    let grid = tree
        .create_node(
            LayoutNodeKind::Box,
            Style {
                display: Display::Grid,
                flex_grow: 1.0,
                min_size: Size::length(0.0),
                padding: Rect::length(12.25),
                gap: Size::length(12.5),
                grid_template_columns: vec![fr(1.0), fr(2.0)],
                grid_template_rows: vec![fr(1.0), fr(1.0)],
                ..Style::default()
            },
            Some(body),
        )
        .unwrap();
    for _ in 0..4 {
        let cell = tree
            .create_node(
                LayoutNodeKind::Box,
                Style {
                    padding: Rect::length(10.25),
                    align_items: AlignItems::START,
                    ..Style::default()
                },
                Some(grid),
            )
            .unwrap();
        tree.create_node(
            LayoutNodeKind::Box,
            Style {
                size: Size {
                    width: percent(0.65),
                    height: percent(0.5),
                },
                ..Style::default()
            },
            Some(cell),
        )
        .unwrap();
    }
    tree
}

fn main() {
    let smoke_test = std::env::args().any(|argument| argument == "--smoke-test");
    let prepared = Rc::new(Cell::new(None));
    let observed = prepared.clone();
    Application::new()
        .run(async move |cx, exit| {
            let window = Window::builder()
                .with_window_builder(|builder| {
                    builder
                        .with_title("PhaUI — Taffy layout")
                        .with_inner_size(tao::dpi::LogicalSize::new(1000.0, 700.0))
                })
                .build(&cx)
                .await
                .unwrap();
            let tree = demo_tree();
            let options =
                WindowRenderOptions::default().with_content(move |device, _queue, config| {
                    LayoutDemo {
                        tree,
                        renderer: LayoutDebugRenderer::new(device, config),
                        viewport: None,
                        prepared: observed.clone(),
                    }
                });
            WindowRender::new_with_options(&window, &cx, options)
                .await
                .unwrap()
                .register(&cx)
                .unwrap();
            let window_id = window.window.id();
            if smoke_test {
                let native = window.window.clone();
                let events = cx.events.clone();
                let prepared = prepared.clone();
                let mut stage = 0;
                let sizes = [[1000.0, 700.0], [720.0, 480.0], [1133.0, 777.0]];
                cx.events
                    .observe(move |event: On<RedrawRequested>| {
                        if event.window_id != window_id {
                            return;
                        }
                        let Some(viewport) = prepared.get() else {
                            return;
                        };
                        let Some(expected) = sizes.get(stage) else {
                            return;
                        };
                        if viewport.logical_size() != *expected {
                            return;
                        }
                        eprintln!("Layout frame verified: {viewport:?}");
                        stage += 1;
                        if stage < sizes.len() {
                            native.set_inner_size(tao::dpi::LogicalSize::new(
                                sizes[stage][0],
                                sizes[stage][1],
                            ));
                        } else {
                            events.try_emit(CloseRequested { window_id }).unwrap();
                        }
                    })
                    .unwrap();
            }
            let events = cx.events.clone();
            cx.events
                .observe(move |event: On<CloseRequested>| {
                    if event.window_id == window_id {
                        events
                            .try_emit(Shutdown)
                            .expect("Failed to enqueue shutdown");
                    }
                })
                .unwrap();
            window.window.request_redraw();
            exit.await;
        })
        .unwrap();
}
