//! Surface options independent of the current window size.

pub struct SurfaceOptions {
    pub(super) usage: wgpu::TextureUsages,
    pub(super) color_space: wgpu::SurfaceColorSpace,
    pub(super) present_mode: wgpu::PresentMode,
    pub(super) view_formats: Vec<wgpu::TextureFormat>,
    pub(super) desired_maximum_frame_latency: u32,
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
