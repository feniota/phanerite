use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

/// Persistent drawing resources, created once after GPU initialization.
///
/// The renderer clears the target before calling `render`. Additional render
/// passes should use `LoadOp::Load` to preserve that background. Submission and
/// presentation are handled by `WindowRender`.
pub trait RenderContent: Send {
    /// Called after surface reconfiguration; the factory receives initial config.
    fn configure(&mut self, _device: &Device, _queue: &Queue, _config: &SurfaceConfiguration) {}

    fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        target: &TextureView,
        encoder: &mut CommandEncoder,
    );
}

pub type RenderContentFactory = Box<
    dyn FnOnce(&Device, &Queue, &SurfaceConfiguration) -> Box<dyn RenderContent> + Send + 'static,
>;

pub(super) struct EmptyContent;

impl RenderContent for EmptyContent {
    fn render(&mut self, _: &Device, _: &Queue, _: &TextureView, _: &mut CommandEncoder) {}
}
