use futures_lite::future::block_on;
use phanerite_layout::{LayoutNodeKind, LayoutSnapshot, LayoutTree, taffy::prelude::*};
use phanerite_render::{LayoutDebugRenderer, LayoutRenderError, RenderContent, RenderViewport};

// Explicitly opt in on machines with a native GPU or software Vulkan adapter.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn gpu_outlines_are_one_physical_pixel_at_multiple_scales() {
    block_on(async {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&Default::default())
            .await
            .expect("GPU adapter required for this test");
        eprintln!("Testing outlines on {:?}", adapter.get_info());
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8Unorm,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: 128,
            height: 128,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        let mut renderer = LayoutDebugRenderer::new(&device, &config);
        let mut tree: LayoutTree = LayoutTree::new(128.0, 128.0).unwrap();
        let parent = tree
            .create_node(
                LayoutNodeKind::Box,
                Style {
                    position: Position::Absolute,
                    inset: Rect {
                        left: length(3.2),
                        top: length(4.3),
                        ..Rect::auto()
                    },
                    size: Size {
                        width: length(26.1),
                        height: length(28.4),
                    },
                    ..Style::default()
                },
                Some(tree.root_node()),
            )
            .unwrap();
        tree.create_node(
            LayoutNodeKind::Box,
            Style {
                position: Position::Absolute,
                inset: Rect {
                    left: length(2.0),
                    top: length(1.4),
                    ..Rect::auto()
                },
                size: Size {
                    width: length(11.7),
                    height: length(10.4),
                },
                ..Style::default()
            },
            Some(parent),
        )
        .unwrap();
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let viewport = RenderViewport::new(128, 128, scale).unwrap();
            let [width, height] = viewport.logical_size();
            tree.resize(width, height).unwrap();
            let snapshot = tree.compute_layout().unwrap();
            renderer
                .update(&device, &queue, viewport, &snapshot)
                .unwrap();
            let pixels = read_frame(&device, &queue, &mut renderer);
            // Check every physical pixel: no fill, no doubled thickness, no
            // intermediate rounding of the child's parent-relative position.
            let rects = [
                [0.0, 0.0, width, height],
                [3.2, 4.3, 26.1, 28.4],
                [5.2, 5.7, 11.7, 10.4],
            ];
            for y in 0..128 {
                for x in 0..128 {
                    let expected = rects.iter().any(|rect| {
                        let low_x = (rect[0] * scale as f32).round_ties_even() as u32;
                        let low_y = (rect[1] * scale as f32).round_ties_even() as u32;
                        let high_x = ((rect[0] + rect[2]) * scale as f32).round_ties_even() as u32;
                        let high_y = ((rect[1] + rect[3]) * scale as f32).round_ties_even() as u32;
                        x >= low_x
                            && x < high_x
                            && y >= low_y
                            && y < high_y
                            && (x == low_x || x + 1 == high_x || y == low_y || y + 1 == high_y)
                    });
                    let offset = ((y * 128 + x) * 4) as usize;
                    let painted = pixels[offset..offset + 4] != [255, 255, 255, 255];
                    assert_eq!(painted, expected, "scale {scale}, pixel ({x}, {y})");
                }
            }
            let empty = LayoutSnapshot {
                viewport: snapshot.viewport,
                nodes: vec![],
            };
            renderer.update(&device, &queue, viewport, &empty).unwrap();
            assert!(
                read_frame(&device, &queue, &mut renderer)
                    .iter()
                    .all(|byte| *byte == 255)
            );
            let stale_viewport = RenderViewport::new(256, 128, scale).unwrap();
            assert!(matches!(
                renderer.update(&device, &queue, stale_viewport, &snapshot),
                Err(LayoutRenderError::ViewportMismatch { .. })
            ));
        }
    });
}

fn read_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut LayoutDebugRenderer,
) -> Vec<u8> {
    let extent = wgpu::Extent3d {
        width: 128,
        height: 128,
        depth_or_array_layers: 1,
    };
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Outline test target"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Outline test readback"),
        size: 128 * 128 * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                store: wgpu::StoreOp::Store,
            },
        })];
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &attachments,
            ..Default::default()
        });
    }
    renderer.render(device, queue, &view, &mut encoder);
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(512),
                rows_per_image: Some(128),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback.map_async(wgpu::MapMode::Read, .., move |result| {
        sender.send(result).unwrap()
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(30)),
        })
        .unwrap();
    receiver
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap()
        .unwrap();
    let pixels = readback.get_mapped_range(..).unwrap().to_vec();
    readback.unmap();
    pixels
}
