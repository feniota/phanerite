#![recursion_limit = "256"]

use phanerite_executor::event::On;
use phanerite_executor::tao_event::window::CloseRequested;
use phanerite_executor::window::{RenderContent, Window, WindowRender, WindowRenderOptions};
use phanerite_executor::{Application, Shutdown};

struct Triangle {
    pipeline: wgpu::RenderPipeline,
    format: wgpu::TextureFormat,
}

impl Triangle {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(config.format.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            format: config.format,
        }
    }
}

impl RenderContent for Triangle {
    fn configure(
        &mut self,
        device: &wgpu::Device,
        _: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if self.format != config.format {
            *self = Self::new(device, config);
        }
    }

    fn render(
        &mut self,
        _: &wgpu::Device,
        _: &wgpu::Queue,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Triangle pass"),
            color_attachments: &attachments,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}

fn main() {
    Application::new()
        .run(async move |cx, exit| {
            let window = Window::builder().build(&cx).await.unwrap();
            let options = WindowRenderOptions::default()
                .with_content(|device, _queue, config| Triangle::new(device, config));
            let render = WindowRender::new_with_options(&window, &cx, options)
                .await
                .unwrap();
            render.register(&cx).unwrap();
            let window_id = window.window.id();
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
