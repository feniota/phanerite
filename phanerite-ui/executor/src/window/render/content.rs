use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

use super::RenderViewport;

/// Persistent drawing resources, created once after GPU initialization.
///
/// The renderer clears the target before calling `render`. Additional render
/// passes should use `LoadOp::Load` to preserve that background. Submission and
/// presentation are handled by `WindowRender`.
/// Like event observers, content lives on the local executor and need not be
/// `Send`. Transfer owned snapshots when introducing a separate render thread.
pub trait RenderContent {
    /// Called after surface reconfiguration; the factory receives initial config.
    fn configure(&mut self, _device: &Device, _queue: &Queue, _config: &SurfaceConfiguration) {}

    /// Called before each frame, including when only the window scale changes.
    /// CPU layout can publish a new snapshot here before GPU encoding begins.
    fn prepare(&mut self, _device: &Device, _queue: &Queue, _viewport: RenderViewport) {}

    fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        target: &TextureView,
        encoder: &mut CommandEncoder,
    );
}

pub type RenderContentFactory =
    Box<dyn FnOnce(&Device, &Queue, &SurfaceConfiguration) -> Box<dyn RenderContent> + 'static>;

pub(super) struct EmptyContent;

impl RenderContent for EmptyContent {
    fn render(&mut self, _: &Device, _: &Queue, _: &TextureView, _: &mut CommandEncoder) {}
}
