use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

use crate::RenderViewport;

/// Persistent drawing resources, created once after GPU initialization.
///
/// The host clears the target before calling `render`. Additional render passes
/// should use `LoadOp::Load` to preserve that background. Command submission and
/// presentation are the host's responsibility. Offscreen hosts need no window.
///
/// Content need not be `Send`, so a local host can own a layout tree. Transfer
/// owned snapshots when introducing a separate render thread.
pub trait RenderContent {
    /// Called after target reconfiguration; the host provides initial config
    /// when constructing the content.
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
