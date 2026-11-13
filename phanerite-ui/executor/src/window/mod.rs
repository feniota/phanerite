//! Native windows and their independently owned renderers.

mod builder;
pub mod render;

pub use builder::WindowBuilder;
pub use render::{
    LayoutDebugRenderer, LayoutRenderError, RenderContent, RenderContentFactory, RenderViewport,
    SurfaceOptions, WindowRender, WindowRenderError, WindowRenderOptions,
};

use std::sync::Arc;
use tao::window::Window as TaoWindow;

pub struct Window {
    pub window: Arc<TaoWindow>,
}

impl Window {
    pub fn builder() -> WindowBuilder {
        Default::default()
    }
}
