//! Event registration and renderer lifetime management.

use super::WindowRender;
use crate::{
    Context,
    event::{EventHandle, ObserveError, On},
    tao_event::{lifecycle::RedrawRequested, window::CloseRequested},
};
use tao::window::WindowId;

impl WindowRender {
    /// Registers drawing for this window and removes both listeners on its close
    /// request. Registration consumes the renderer; it does not request a redraw.
    pub fn register(mut self, cx: &Context) -> Result<(), ObserveError> {
        register_redraw(&cx.events, self.window_id, move || self.render_frame())
    }
}

fn register_redraw(
    events: &EventHandle,
    window_id: WindowId,
    mut render: impl FnMut() + 'static,
) -> Result<(), ObserveError> {
    let redraw = events.observe(move |event: On<RedrawRequested>| {
        if event.window_id == window_id {
            render();
        }
    })?;
    let cleanup = events.clone();
    if let Err(error) = events.observe(move |event: On<CloseRequested>| {
        if event.window_id == window_id {
            // Changes are applied between event dispatches. Dispatcher teardown
            // already releases the listeners if it has stopped accepting commands.
            let _ = cleanup.remove_observer(redraw);
            let _ = cleanup.remove_observer(event.observer());
        }
    }) {
        let _ = events.remove_observer(redraw);
        return Err(error);
    }
    Ok(())
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
            register_redraw(&events, window_id, move || {
                let _ = &render_resources;
                render_draws.fetch_add(1, Ordering::Relaxed);
            })
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
    fn registration_reports_dispatcher_teardown() {
        // SAFETY: the ID is only used as event data.
        let window_id = unsafe { WindowId::dummy() };
        let (dispatcher, _receiver, events) = EventDispatcher::channel(1);
        drop(dispatcher);
        assert_eq!(
            register_redraw(&events, window_id, || {}),
            Err(ObserveError::DispatcherDropped)
        );
    }
}
