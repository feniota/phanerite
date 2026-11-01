//! GPU lifecycle, frame submission, and pluggable window drawing.

mod content;
mod error;
mod observer;
mod options;
#[cfg(target_os = "linux")]
mod wayland;

pub use content::{RenderContent, RenderContentFactory};
pub use error::WindowRenderError;
pub use options::{SurfaceOptions, WindowRenderOptions};
pub use phanerite_render::RenderViewport;

use super::Window;
use crate::Context;
use std::sync::Arc;
use tao::window::{Window as TaoWindow, WindowId};
use wgpu::{
    Adapter, Color, CommandEncoder, CommandEncoderDescriptor, CurrentSurfaceTexture, Device,
    LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, SubmissionIndex, Surface, SurfaceConfiguration, TextureView,
};

/// Owns the window's GPU resources and shares its native window handle.
///
/// Clears the background and invokes custom drawing. `register` installs frame acquisition,
/// drawing, and presentation in the event dispatcher. `render` can also be used
/// directly with a caller-provided texture view.
pub struct WindowRender {
    surface: Surface<'static>,
    // Field order matters: the GPU surface must be destroyed before its target.
    #[cfg(target_os = "linux")]
    wayland_target: Option<wayland::WaylandTarget>,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub surface_config: SurfaceConfiguration,
    pub surface_options: SurfaceOptions,
    window_id: WindowId,
    window: Arc<TaoWindow>,
    clear_color: Color,
    content: Box<dyn RenderContent>,
}

impl WindowRender {
    /// Creates all GPU resources using the context's shared instance.
    /// The renderer retains a native window handle, not a borrow of `Window`.
    pub async fn new(window: &Window, cx: &Context) -> Result<Self, WindowRenderError> {
        Self::new_with_options(window, cx, WindowRenderOptions::default()).await
    }

    pub async fn new_with_options(
        window: &Window,
        cx: &Context,
        options: WindowRenderOptions,
    ) -> Result<Self, WindowRenderError> {
        #[cfg(target_os = "linux")]
        let wayland_target = wayland::WaylandTarget::new(&window.window)?;
        #[cfg(target_os = "linux")]
        let surface = if let Some(target) = &wayland_target {
            // SAFETY: the local renderer retains the native target, keeps the
            // wgpu surface private, and destroys it before the target.
            unsafe { cx.instance.create_surface_unsafe(target.raw_target()?)? }
        } else {
            cx.instance.create_surface(window.window.clone())?
        };
        #[cfg(not(target_os = "linux"))]
        let surface = cx.instance.create_surface(window.window.clone())?;
        let adapter = cx
            .instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: options.adapter.power_preference,
                force_fallback_adapter: options.adapter.force_fallback_adapter,
                compatible_surface: Some(&surface),
                apply_limit_buckets: options.adapter.apply_limit_buckets,
            })
            .await?;
        let (device, queue) = adapter.request_device(&options.device).await?;
        let size = window.window.inner_size();
        #[cfg(target_os = "linux")]
        let size = wayland_target
            .as_ref()
            .map_or(size, |target| target.physical_size());
        let surface_config = Self::configuration(&surface, &adapter, size, &options.surface)?;
        surface.configure(&device, &surface_config);
        let content = (options.content)(&device, &queue, &surface_config);
        Ok(Self {
            surface,
            #[cfg(target_os = "linux")]
            wayland_target,
            adapter,
            device,
            queue,
            surface_config,
            surface_options: options.surface,
            window_id: window.window.id(),
            window: window.window.clone(),
            clear_color: options.clear_color,
            content,
        })
    }

    /// Applies the renderer's surface options and current physical window size.
    pub fn configure(&mut self) -> Result<(), WindowRenderError> {
        let config = Self::configuration(
            &self.surface,
            &self.adapter,
            self.physical_size(),
            &self.surface_options,
        )?;
        self.surface.configure(&self.device, &config);
        self.surface_config = config;
        self.content
            .configure(&self.device, &self.queue, &self.surface_config);
        Ok(())
    }

    /// The GPU presentation surface, whose native target remains renderer-owned.
    pub fn surface(&self) -> &Surface<'_> {
        &self.surface
    }

    fn physical_size(&self) -> tao::dpi::PhysicalSize<u32> {
        #[cfg(target_os = "linux")]
        if let Some(target) = &self.wayland_target {
            return target.physical_size();
        }
        self.window.inner_size()
    }

    fn configuration(
        surface: &Surface<'_>,
        adapter: &Adapter,
        size: tao::dpi::PhysicalSize<u32>,
        options: &SurfaceOptions,
    ) -> Result<SurfaceConfiguration, WindowRenderError> {
        let caps = surface.get_capabilities(adapter);
        Ok(SurfaceConfiguration {
            usage: options.usage,
            format: *caps
                .formats
                .first()
                .ok_or(WindowRenderError::UnsupportedSurface)?,
            color_space: options.color_space,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: options.present_mode,
            alpha_mode: *caps
                .alpha_modes
                .first()
                .ok_or(WindowRenderError::UnsupportedSurface)?,
            view_formats: options.view_formats.clone(),
            desired_maximum_frame_latency: options.desired_maximum_frame_latency,
        })
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Encodes drawing without submitting, allowing other drawing in the same frame.
    ///
    /// The encoder and target must belong to this renderer's device. The target
    /// texture must have `TextureUsages::RENDER_ATTACHMENT` usage.
    /// Its extent must match `surface_config`; use `encode_with_viewport` for
    /// targets with a different size or scale.
    pub fn encode(&mut self, encoder: &mut CommandEncoder, target: &TextureView) {
        if let Some(viewport) = RenderViewport::new(
            self.surface_config.width,
            self.surface_config.height,
            self.window.scale_factor(),
        ) {
            self.encode_with_viewport(encoder, target, viewport);
        }
    }

    /// Encodes a frame using an explicit target extent and scale, e.g. offscreen.
    pub fn encode_with_viewport(
        &mut self,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        viewport: RenderViewport,
    ) {
        self.content.prepare(&self.device, &self.queue, viewport);
        let color_attachments = [Some(RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(self.clear_color),
                store: StoreOp::Store,
            },
        })];

        let pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("PhaneriteUI window render pass"),
            color_attachments: &color_attachments,
            ..Default::default()
        });
        drop(pass);
        self.content
            .render(&self.device, &self.queue, target, encoder);
    }

    /// Encodes and submits one frame without acquiring or presenting a surface.
    ///
    /// The target must belong to this renderer's device and have render attachment
    /// usage. Returns after submission without waiting for GPU completion.
    pub fn render(&mut self, target: &TextureView) -> SubmissionIndex {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("PhaneriteUI window command encoder"),
            });
        self.encode(&mut encoder, target);
        self.queue.submit([encoder.finish()])
    }

    fn render_frame(&mut self) {
        let size = self.physical_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        if (self.surface_config.width != size.width || self.surface_config.height != size.height)
            && let Err(error) = self.configure()
        {
            tracing::error!(%error, "surface reconfiguration failed");
            return;
        }

        let (frame, suboptimal) = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => (frame, false),
            CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => return,
            CurrentSurfaceTexture::Outdated => {
                if let Err(error) = self.configure() {
                    tracing::error!(%error, "surface reconfiguration failed");
                    return;
                }
                self.window.request_redraw();
                return;
            }
            CurrentSurfaceTexture::Lost => {
                tracing::error!(window_id = ?self.window_id, "render surface lost; recreation required");
                return;
            }
            CurrentSurfaceTexture::Validation => {
                tracing::error!(window_id = ?self.window_id, "surface frame acquisition failed validation");
                return;
            }
        };
        let view = frame.texture.create_view(&Default::default());
        self.render(&view);
        drop(view);
        self.queue.present(frame);

        if suboptimal {
            if let Err(error) = self.configure() {
                tracing::error!(%error, "surface reconfiguration failed");
                return;
            }
            self.window.request_redraw();
        }
    }
}
