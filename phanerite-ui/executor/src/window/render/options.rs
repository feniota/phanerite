//! GPU initialization, surface, and custom drawing options.

use super::content::{EmptyContent, RenderContent, RenderContentFactory};
use wgpu::{
    Color, Device, DeviceDescriptor, Queue, RequestAdapterOptionsBase, SurfaceConfiguration,
};

/// GPU initialization options, separate from native window construction.
pub struct WindowRenderOptions {
    pub adapter: RequestAdapterOptionsBase<()>,
    pub device: DeviceDescriptor<'static>,
    pub surface: SurfaceOptions,
    pub clear_color: Color,
    pub content: RenderContentFactory,
}

impl WindowRenderOptions {
    /// Creates persistent drawing resources after the device and format are known.
    pub fn with_content<F, C>(mut self, factory: F) -> Self
    where
        F: FnOnce(&Device, &Queue, &SurfaceConfiguration) -> C + 'static,
        C: RenderContent + 'static,
    {
        self.content =
            Box::new(move |device, queue, config| Box::new(factory(device, queue, config)));
        self
    }
}

impl Default for WindowRenderOptions {
    fn default() -> Self {
        Self {
            adapter: RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: None,
                apply_limit_buckets: false,
            },
            device: DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                #[cfg(target_arch = "wasm32")]
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                #[cfg(not(target_arch = "wasm32"))]
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                label: Some("PhaneriteUI GPU Device"),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            },
            surface: SurfaceOptions::default(),
            clear_color: Color::WHITE,
            content: Box::new(|_, _, _| Box::new(EmptyContent)),
        }
    }
}

pub struct SurfaceOptions {
    pub usage: wgpu::TextureUsages,
    pub color_space: wgpu::SurfaceColorSpace,
    pub present_mode: wgpu::PresentMode,
    pub view_formats: Vec<wgpu::TextureFormat>,
    pub desired_maximum_frame_latency: u32,
}

impl Default for SurfaceOptions {
    fn default() -> Self {
        Self {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            color_space: wgpu::SurfaceColorSpace::Auto,
            present_mode: wgpu::PresentMode::AutoVsync,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }
}
