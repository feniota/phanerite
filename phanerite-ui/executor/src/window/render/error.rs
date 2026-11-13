//! Errors returned while creating a window renderer.

use wgpu::{CreateSurfaceError, RequestAdapterError, RequestDeviceError};

/// Failure to initialize the GPU resources for a window.
#[derive(Debug, thiserror::Error)]
pub enum WindowRenderError {
    #[error("failed to create window surface: {0}")]
    CreateSurface(#[from] CreateSurfaceError),
    #[error("failed to request window adapter: {0}")]
    RequestAdapter(#[from] RequestAdapterError),
    #[error("failed to request window device: {0}")]
    RequestDevice(#[from] RequestDeviceError),
}
