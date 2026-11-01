//! Border-only layout inspection. No component state or layout-tree access.

use phanerite_layout::LayoutSnapshot;
use wgpu::util::DeviceExt;

use super::{RenderContent, RenderViewport};

// Two Float32x4 instance attributes: logical rect and linear RGBA color.
type RectInstance = [f32; 8];
const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
    wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
const COLORS: [[f32; 4]; 6] = [
    [0.16, 0.21, 0.29, 1.0],
    [0.02, 0.24, 0.60, 1.0],
    [0.02, 0.38, 0.25, 1.0],
    [0.55, 0.16, 0.04, 1.0],
    [0.32, 0.09, 0.57, 1.0],
    [0.01, 0.34, 0.42, 1.0],
];

#[derive(Debug, thiserror::Error)]
pub enum LayoutRenderError {
    #[error("layout snapshot viewport {snapshot:?} does not match render viewport {render:?}")]
    ViewportMismatch {
        snapshot: [f32; 2],
        render: [f32; 2],
    },
    #[error("layout snapshot exceeds the GPU instance buffer limit")]
    TooManyNodes,
}

/// Draws owned layout snapshots as one-physical-pixel inset outlines.
///
/// Logical rectangles remain fractional in the buffer. The shader scales and
/// snaps absolute edges, then produces triangles for four one-pixel strips.
/// Only the viewport clips drawing; overflow clipping and CSS stacking are not
/// implemented. Text/replaced nodes draw boxes only, without intrinsic content.
pub struct LayoutDebugRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instances: wgpu::Buffer,
    capacity: u64,
    instance_count: u32,
    format: wgpu::TextureFormat,
}

impl LayoutDebugRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let pipeline = pipeline(device, config.format);
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Layout viewport uniform"),
            contents: bytemuck::cast_slice(&[1.0f32, 1.0, 1.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = bind_group(device, &pipeline, &uniform);
        let capacity = size_of::<RectInstance>() as u64;
        Self {
            pipeline,
            uniform,
            bind_group,
            instances: instance_buffer(device, capacity),
            capacity,
            instance_count: 0,
            format: config.format,
        }
    }

    /// Replaces the current GPU batch. Call only when layout or viewport changes.
    /// The snapshot must have been computed for this viewport's logical size.
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: RenderViewport,
        snapshot: &LayoutSnapshot,
    ) -> Result<(), LayoutRenderError> {
        let size = [snapshot.viewport.width, snapshot.viewport.height];
        if size != viewport.logical_size() {
            return Err(LayoutRenderError::ViewportMismatch {
                snapshot: size,
                render: viewport.logical_size(),
            });
        }
        let data = instances(snapshot);
        let count = u32::try_from(data.len()).map_err(|_| LayoutRenderError::TooManyNodes)?;
        let bytes = bytemuck::cast_slice::<RectInstance, u8>(&data);
        let required = bytes.len() as u64;
        let limit = device.limits().max_buffer_size;
        if required > limit {
            return Err(LayoutRenderError::TooManyNodes);
        }
        if required > self.capacity {
            self.capacity = required.next_power_of_two().min(limit);
            self.instances = instance_buffer(device, self.capacity);
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.instances, 0, bytes);
        }
        let [width, height] = viewport.physical_size();
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::cast_slice(&[
                width as f32,
                height as f32,
                viewport.scale_factor() as f32,
                0.0,
            ]),
        );
        self.instance_count = count;
        Ok(())
    }
}

impl RenderContent for LayoutDebugRenderer {
    fn configure(
        &mut self,
        device: &wgpu::Device,
        _: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if self.format != config.format {
            self.pipeline = pipeline(device, config.format);
            self.bind_group = bind_group(device, &self.pipeline, &self.uniform);
            self.format = config.format;
        }
    }

    fn render(
        &mut self,
        _: &wgpu::Device,
        _: &wgpu::Queue,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.instance_count == 0 {
            return;
        }
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
            label: Some("Layout outline pass"),
            color_attachments: &attachments,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..24, 0..self.instance_count);
    }
}

fn instances(snapshot: &LayoutSnapshot) -> Vec<RectInstance> {
    snapshot
        .nodes
        .iter()
        .filter_map(|node| {
            let rect = [
                node.position.x,
                node.position.y,
                node.size.width,
                node.size.height,
            ];
            if rect.iter().any(|value| !value.is_finite()) || rect[2] <= 0.0 || rect[3] <= 0.0 {
                return None;
            }
            let color = COLORS[node.depth % COLORS.len()];
            Some([
                rect[0], rect[1], rect[2], rect[3], color[0], color[1], color[2], color[3],
            ])
        })
        .collect()
}

fn instance_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Layout rectangle instances"),
        size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn bind_group(
    device: &wgpu::Device,
    pipeline: &wgpu::RenderPipeline,
    uniform: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Layout viewport bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    })
}

fn pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Layout outline shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("layout.wgsl").into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Layout outline pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<RectInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &ATTRIBUTES,
            })],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        multiview_mask: None,
        cache: None,
    })
}
