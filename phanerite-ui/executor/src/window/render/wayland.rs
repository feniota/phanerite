//! Give Vulkan its own Wayland subsurface, separate from GTK's CSD buffers.

use std::{ptr::NonNull, sync::Arc};

use gtk::{gdk, glib, prelude::*};
use tao::{platform::unix::WindowExtUnix, window::Window};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle, RawWindowHandle, WaylandWindowHandle};

use super::WindowRenderError;

/// All GTK access and destruction stays on the application's event-loop thread.
/// The renderer must drop its wgpu surface before dropping this owner.
pub(super) struct WaylandTarget {
    child: gdk::Window,
    content: gtk::Widget,
    allocation: Option<glib::SignalHandlerId>,
    window: Arc<Window>,
}

impl WaylandTarget {
    pub fn new(window: &Arc<Window>) -> Result<Option<Self>, WindowRenderError> {
        if !matches!(
            window.window_handle()?.as_raw(),
            RawWindowHandle::Wayland(_)
        ) {
            return Ok(None);
        }
        let toplevel = window.gtk_window();
        let parent = toplevel
            .window()
            .ok_or(WindowRenderError::NativeTargetUnavailable)?;
        let content: gtk::Widget = window
            .default_vbox()
            .map(|widget| widget.clone().upcast())
            .or_else(|| toplevel.child())
            .ok_or(WindowRenderError::NativeTargetUnavailable)?;
        let (x, y) = content
            .translate_coordinates(toplevel, 0, 0)
            .unwrap_or((0, 0));
        let width = content.allocated_width().max(1);
        let height = content.allocated_height().max(1);
        // A Wayland subsurface uses transient_for as its native parent. A GDK
        // child parent would also add ancestor offsets a second time.
        let child = gdk::Window::new(
            None,
            &gdk::WindowAttr {
                window_type: gdk::WindowType::Subsurface,
                x: Some(x),
                y: Some(y),
                width: width.saturating_add(1),
                height,
                ..Default::default()
            },
        );
        // No GTK painting is associated with this GdkWindow. Only wgpu may
        // attach buffers; GTK still paints the parent and native decorations.
        child.set_transient_for(&parent);
        child.set_pass_through(true);
        child.freeze_updates();
        child.show();
        // Resize after realization so GDK sends the initial buffer scale even
        // when no GTK paint ever occurs on this GPU-owned surface.
        child.move_resize(x, y, width, height);
        let resized_child = child.clone();
        let native = window.clone();
        let allocation = content.connect_size_allocate(move |widget, allocation| {
            let (x, y) = widget
                .translate_coordinates(native.gtk_window(), 0, 0)
                .unwrap_or((0, 0));
            resized_child.move_resize(x, y, allocation.width().max(1), allocation.height().max(1));
            native.request_redraw();
        });
        Ok(Some(Self {
            child,
            content,
            allocation: Some(allocation),
            window: window.clone(),
        }))
    }

    pub fn physical_size(&self) -> tao::dpi::PhysicalSize<u32> {
        tao::dpi::LogicalSize::new(
            self.content.allocated_width().max(0) as u32,
            self.content.allocated_height().max(0) as u32,
        )
        .to_physical(self.window.scale_factor())
    }

    pub fn raw_target(&self) -> Result<wgpu::SurfaceTargetUnsafe, WindowRenderError> {
        // SAFETY: child is a live Wayland GdkWindow, created on this thread.
        let surface = unsafe {
            gdk_wayland_sys::gdk_wayland_window_get_wl_surface(self.child.as_ptr().cast())
        };
        let surface = NonNull::new(surface).ok_or(WindowRenderError::NativeTargetUnavailable)?;
        Ok(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(self.window.display_handle()?.as_raw()),
            raw_window_handle: RawWindowHandle::Wayland(WaylandWindowHandle::new(surface)),
        })
    }
}

impl Drop for WaylandTarget {
    fn drop(&mut self) {
        // Disconnect releases the closure's child/window references.
        // This runs after the GPU surface has been destroyed.
        if let Some(allocation) = self.allocation.take() {
            self.content.disconnect(allocation);
        }
        self.child.destroy();
    }
}
