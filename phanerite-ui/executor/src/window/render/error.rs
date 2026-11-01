//! Errors returned while creating a window renderer.

use wgpu::{CreateSurfaceError, RequestAdapterError, RequestDeviceError};

/// Failure to initialize the GPU resources for a window.
#[derive(Debug, thiserror::Error)]
pub enum WindowRenderError {
    #[error("native render target is not available")]
    NativeTargetUnavailable,
    #[error("failed to obtain native window handles: {0}")]
    NativeHandle(#[from] wgpu::rwh::HandleError),
    #[error("surface has no supported configuration for this adapter")]
    UnsupportedSurface,
    #[error("failed to create window surface: {0}")]
    CreateSurface(#[from] CreateSurfaceError),
    #[error("failed to request window adapter: {0}")]
    RequestAdapter(#[from] RequestAdapterError),
    #[error("failed to request window device: {0}")]
    RequestDevice(#[from] RequestDeviceError),
}
