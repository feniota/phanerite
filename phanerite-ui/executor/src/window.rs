use crate::app::Context;
use std::sync::Arc;
use tao::window::{Window as TaoWindow, WindowBuilder as TaoWindowBuilder};
use wgpu::{
    Adapter, Device, DeviceDescriptor, Queue, RequestAdapterOptions, RequestAdapterOptionsBase,
    Surface, SurfaceConfiguration,
};

pub struct WindowBuilder {
    window: TaoWindowBuilder,
    adapter: RequestAdapterOptionsBase<()>,
    device: DeviceDescriptor<'static>,
    surface: SurfaceConfigBase,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            window: TaoWindowBuilder::default().with_title("PhaneriteUI"),
            adapter: RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: None,
                apply_limit_buckets: false,
            },
            device: DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                label: Some("PhaneriteUI GPU Device"),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            },
            surface: Default::default(),
        }
    }
}

pub struct SurfaceConfigBase {
    usage: wgpu::TextureUsages,
    color_space: wgpu::SurfaceColorSpace,
    present_mode: wgpu::PresentMode,
    view_formats: Vec<wgpu::TextureFormat>,
    desired_maximum_frame_latency: u32,
}

impl Default for SurfaceConfigBase {
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

pub struct Window {
    window: Arc<TaoWindow>,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    config_base: SurfaceConfigBase,
}

impl WindowBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn config_window<F>(mut self, op: F) -> Self
    where
        F: FnOnce(TaoWindowBuilder) -> TaoWindowBuilder,
    {
        self.window = op(self.window);
        self
    }
    pub fn config_adapter(mut self, config: RequestAdapterOptionsBase<()>) -> Self {
        self.adapter = config;
        self
    }
    pub fn config_device(mut self, config: DeviceDescriptor<'static>) -> Self {
        self.device = config;
        self
    }
    pub async fn build(self, cx: &Context) -> Result<Window, taao::event::SubmitError> {
        // Create window
        let window = cx
            .event
            .submit(|event, _| self.window.build(event))
            .await?
            .expect("OS Error: failed to create window");
        let window = Arc::new(window);

        // Create Surface
        let surface = cx
            .instance
            .create_surface(window.clone())
            .expect("GPU Error: failed to create surface");

        // Create Adapter
        let adapter = cx
            .instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: self.adapter.power_preference,
                force_fallback_adapter: self.adapter.force_fallback_adapter,
                compatible_surface: Some(&surface),
                apply_limit_buckets: self.adapter.apply_limit_buckets,
            })
            .await
            .expect("GPU Error: failed to create adapter");

        // Create device and queue
        let (device, queue) = adapter
            .request_device(&self.device)
            .await
            .expect("GPU Error: failed to create device and queue");

        // Default Config
        let caps = surface.get_capabilities(&adapter);
        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: self.surface.usage,
            format: caps.formats[0],
            color_space: self.surface.color_space,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: self.surface.present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: self.surface.view_formats.clone(),
            desired_maximum_frame_latency: self.surface.desired_maximum_frame_latency,
        };
        surface.configure(&device, &config);

        Ok(Window {
            window,
            surface,
            adapter,
            device,
            queue,
            config,
            config_base: self.surface,
        })
    }
}

impl Window {
    pub fn builder() -> WindowBuilder {
        Default::default()
    }
    pub fn configure(&mut self, config: SurfaceConfigBase) {
        self.config_base = config;

        let caps = self.surface.get_capabilities(&self.adapter);
        let size = self.window.inner_size();
        let config = SurfaceConfiguration {
            usage: self.config.usage,
            format: caps.formats[0],
            color_space: self.config.color_space,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: self.config.present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: self.config.view_formats.clone(),
            desired_maximum_frame_latency: self.config.desired_maximum_frame_latency,
        };
        self.surface.configure(&self.device, &config);
    }
}
