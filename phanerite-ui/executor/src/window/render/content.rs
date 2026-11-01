use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};

pub use phanerite_render::RenderContent;

pub type RenderContentFactory =
    Box<dyn FnOnce(&Device, &Queue, &SurfaceConfiguration) -> Box<dyn RenderContent> + 'static>;

pub(super) struct EmptyContent;

impl RenderContent for EmptyContent {
    fn render(&mut self, _: &Device, _: &Queue, _: &TextureView, _: &mut CommandEncoder) {}
}
