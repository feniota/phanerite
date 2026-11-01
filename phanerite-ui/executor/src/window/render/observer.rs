//! Event registration and renderer lifetime management.

use super::WindowRender;
use crate::{
    Context,
    event::{EventHandle, ObserveError, On},
    tao_event::{
        lifecycle::RedrawRequested,
        window::{CloseRequested, Resized, ScaleFactorChanged},
    },
};
use tao::window::WindowId;

impl WindowRender {
    /// Registers drawing and native resize/DPI invalidation for this window.
    /// Removes all listeners on its close
    /// request. Registration consumes the renderer; it does not request a redraw.
    pub fn register(mut self, cx: &Context) -> Result<(), ObserveError> {
        let window = self.window.clone();
        register_redraw(
            &cx.events,
            self.window_id,
            move || self.render_frame(),
            move || window.request_redraw(),
        )
    }
}

fn register_redraw(
    events: &EventHandle,
    window_id: WindowId,
    mut render: impl FnMut() + 'static,
    request_redraw: impl Fn() + Clone + 'static,
) -> Result<(), ObserveError> {
    let mut registered = Vec::new();
    let result = (|| {
        registered.push(events.observe(move |event: On<RedrawRequested>| {
            if event.window_id == window_id {
                render();
            }
        })?);
        let resized = request_redraw.clone();
        registered.push(events.observe(move |event: On<Resized>| {
            if event.window_id == window_id {
                resized();
            }
        })?);
        registered.push(events.observe(move |event: On<ScaleFactorChanged>| {
            if event.window_id == window_id {
                request_redraw();
            }
        })?);
        let cleanup = events.clone();
        let observers = registered.clone();
        events.observe(move |event: On<CloseRequested>| {
            if event.window_id == window_id {
                for &observer in &observers {
                    let _ = cleanup.remove_observer(observer);
                }
                let _ = cleanup.remove_observer(event.observer());
            }
        })?;
        Ok(())
    })();
    if result.is_err() {
        for observer in registered {
            let _ = events.remove_observer(observer);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use futures_lite::future::{block_on, poll_once};

    use super::*;
    use crate::event::EventDispatcher;

    #[test]
    fn close_removes_render_listener_and_releases_resources_before_pump_ends() {
        block_on(async {
            // SAFETY: the ID is only used as event data, never in native operations.
            let window_id = unsafe { WindowId::dummy() };
            let (dispatcher, receiver, events) = EventDispatcher::channel(4);
            let draws = Arc::new(AtomicUsize::new(0));
            let resources = Arc::new(());
            let render_resources = resources.clone();
            let render_draws = draws.clone();
            register_redraw(
                &events,
                window_id,
                move || {
                    let _ = &render_resources;
                    render_draws.fetch_add(1, Ordering::Relaxed);
                },
                || {},
            )
            .unwrap();

            events.try_emit(RedrawRequested { window_id }).unwrap();
            events.try_emit(CloseRequested { window_id }).unwrap();
            events.try_emit(RedrawRequested { window_id }).unwrap();
            let mut pump = Box::pin(dispatcher.pump(receiver));
            assert!(poll_once(pump.as_mut()).await.is_none());
            assert_eq!(draws.load(Ordering::Relaxed), 1);
            assert_eq!(Arc::strong_count(&resources), 1);
            events.close();
            pump.await;
        });
    }

    #[test]
    fn resize_and_dpi_changes_request_redraw_only_until_close() {
        block_on(async {
            // SAFETY: the ID is only used as event data.
            let window_id = unsafe { WindowId::dummy() };
            let (dispatcher, receiver, events) = EventDispatcher::channel(8);
            let requests = Arc::new(AtomicUsize::new(0));
            let captured = requests.clone();
            register_redraw(
                &events,
                window_id,
                || {},
                move || {
                    captured.fetch_add(1, Ordering::Relaxed);
                },
            )
            .unwrap();
            let resized = Resized {
                window_id,
                size: tao::dpi::PhysicalSize::new(800, 600),
            };
            let scaled = ScaleFactorChanged {
                window_id,
                scale_factor: 1.25,
                suggested_inner_size: tao::dpi::PhysicalSize::new(1000, 750),
            };
            events.try_emit(resized.clone()).unwrap();
            events.try_emit(scaled.clone()).unwrap();
            events.try_emit(CloseRequested { window_id }).unwrap();
            events.try_emit(resized).unwrap();
            events.try_emit(scaled).unwrap();
            let mut pump = Box::pin(dispatcher.pump(receiver));
            assert!(poll_once(pump.as_mut()).await.is_none());
            assert_eq!(requests.load(Ordering::Relaxed), 2);
            assert_eq!(Arc::strong_count(&requests), 1);
            events.close();
            pump.await;
        });
    }

    #[test]
    fn registration_reports_dispatcher_teardown() {
        // SAFETY: the ID is only used as event data.
        let window_id = unsafe { WindowId::dummy() };
        let (dispatcher, _receiver, events) = EventDispatcher::channel(1);
        drop(dispatcher);
        assert_eq!(
            register_redraw(&events, window_id, || {}, || {}),
            Err(ObserveError::DispatcherDropped)
        );
    }
}
