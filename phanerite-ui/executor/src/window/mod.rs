//! Window state and its independently owned GPU resources.

mod builder;
mod render;
mod surface;

pub use builder::WindowBuilder;
pub use surface::SurfaceOptions;

use std::sync::Arc;
use tao::window::Window as TaoWindow;
use wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};

pub struct Window {
    window: Arc<TaoWindow>,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,
    surface_options: SurfaceOptions,
}

impl Window {
    pub fn builder() -> WindowBuilder {
        Default::default()
    }
    pub fn configure(&mut self, config: SurfaceOptions) {
        self.surface_options = config;

        let caps = self.surface.get_capabilities(&self.adapter);
        let size = self.window.inner_size();
        let config = SurfaceConfiguration {
            usage: self.surface_config.usage,
            format: caps.formats[0],
            color_space: self.surface_config.color_space,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: self.surface_config.present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: self.surface_config.view_formats.clone(),
            desired_maximum_frame_latency: self.surface_config.desired_maximum_frame_latency,
        };
        self.surface.configure(&self.device, &config);
    }
}
