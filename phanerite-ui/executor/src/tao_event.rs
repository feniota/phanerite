//! Typed routing of Taao/Tao events, without interpreting UI interactions.
//!
//! Observers register normally using `On<window::MouseInput>`,
//! `On<window::Resized>`, or `On<device::MouseMotion>`.
//! Each backend input dispatches its original `taao::event::EventKind` first,
//! followed by its typed event, before processing the next input.
//! Custom `emit` events pass through unchanged and are not routed again.

pub mod device;
pub mod lifecycle;
pub mod window;

use taao::event::EventKind;
use tao::event::{DeviceEvent, Event as TaoEvent, WindowEvent};

use crate::event::ErasedEvent;

pub(crate) fn route(event: &EventKind) -> Option<ErasedEvent> {
    Some(match event {
        EventKind::Tao(event) => match event {
            TaoEvent::WindowEvent {
                window_id, event, ..
            } => return route_window(*window_id, event),
            TaoEvent::DeviceEvent {
                device_id, event, ..
            } => return route_device(*device_id, event),
            TaoEvent::NewEvents(cause) => ErasedEvent::new(lifecycle::NewEvents { cause: *cause }),
            TaoEvent::UserEvent(()) => ErasedEvent::new(lifecycle::UserEvent),
            TaoEvent::MainEventsCleared => ErasedEvent::new(lifecycle::MainEventsCleared),
            TaoEvent::RedrawRequested(window_id) => ErasedEvent::new(lifecycle::RedrawRequested {
                window_id: *window_id,
            }),
            TaoEvent::RedrawEventsCleared => ErasedEvent::new(lifecycle::RedrawEventsCleared),
            TaoEvent::LoopDestroyed => ErasedEvent::new(lifecycle::LoopDestroyed),
            TaoEvent::Opened { urls } => ErasedEvent::new(lifecycle::Opened { urls: urls.clone() }),
            TaoEvent::Reopen {
                has_visible_windows,
                ..
            } => ErasedEvent::new(lifecycle::Reopen {
                has_visible_windows: *has_visible_windows,
            }),
            // Future and platform-specific variants remain observable as EventKind.
            _ => return None,
        },
        EventKind::TaoScaleFactorChanged {
            window_id,
            scale_factor,
            suggested_inner_size,
        } => ErasedEvent::new(window::ScaleFactorChanged {
            window_id: *window_id,
            scale_factor: *scale_factor,
            suggested_inner_size: *suggested_inner_size,
        }),
    })
}

#[allow(deprecated)]
fn route_window(window_id: tao::window::WindowId, event: &WindowEvent<'_>) -> Option<ErasedEvent> {
    use window::*;
    Some(match event {
        WindowEvent::Resized(size) => ErasedEvent::new(Resized {
            window_id,
            size: *size,
        }),
        WindowEvent::Moved(position) => ErasedEvent::new(Moved {
            window_id,
            position: *position,
        }),
        WindowEvent::CloseRequested => ErasedEvent::new(CloseRequested { window_id }),
        WindowEvent::Destroyed => ErasedEvent::new(Destroyed { window_id }),
        WindowEvent::Started => ErasedEvent::new(Started { window_id }),
        WindowEvent::Suspended => ErasedEvent::new(Suspended { window_id }),
        WindowEvent::Resumed => ErasedEvent::new(Resumed { window_id }),
        WindowEvent::Stopped => ErasedEvent::new(Stopped { window_id }),
        WindowEvent::DroppedFile(path) => ErasedEvent::new(DroppedFile {
            window_id,
            path: path.clone(),
        }),
        WindowEvent::HoveredFile(path) => ErasedEvent::new(HoveredFile {
            window_id,
            path: path.clone(),
        }),
        WindowEvent::HoveredFileCancelled => ErasedEvent::new(HoveredFileCancelled { window_id }),
        WindowEvent::ReceivedImeText(text) => ErasedEvent::new(ReceivedImeText {
            window_id,
            text: text.clone(),
        }),
        WindowEvent::Focused(focused) => ErasedEvent::new(Focused {
            window_id,
            focused: *focused,
        }),
        WindowEvent::KeyboardInput {
            device_id,
            event,
            is_synthetic,
            ..
        } => ErasedEvent::new(KeyboardInput {
            window_id,
            device_id: *device_id,
            event: event.clone(),
            is_synthetic: *is_synthetic,
        }),
        WindowEvent::ModifiersChanged(modifiers) => ErasedEvent::new(ModifiersChanged {
            window_id,
            modifiers: *modifiers,
        }),
        WindowEvent::CursorMoved {
            device_id,
            position,
            modifiers,
        } => ErasedEvent::new(CursorMoved {
            window_id,
            device_id: *device_id,
            position: *position,
            modifiers: *modifiers,
        }),
        WindowEvent::CursorEntered { device_id } => ErasedEvent::new(CursorEntered {
            window_id,
            device_id: *device_id,
        }),
        WindowEvent::CursorLeft { device_id } => ErasedEvent::new(CursorLeft {
            window_id,
            device_id: *device_id,
        }),
        WindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
            modifiers,
        } => ErasedEvent::new(MouseWheel {
            window_id,
            device_id: *device_id,
            delta: *delta,
            phase: *phase,
            modifiers: *modifiers,
        }),
        WindowEvent::MouseInput {
            device_id,
            state,
            button,
            modifiers,
        } => ErasedEvent::new(MouseInput {
            window_id,
            device_id: *device_id,
            state: *state,
            button: *button,
            modifiers: *modifiers,
        }),
        WindowEvent::TouchpadPressure {
            device_id,
            pressure,
            stage,
        } => ErasedEvent::new(TouchpadPressure {
            window_id,
            device_id: *device_id,
            pressure: *pressure,
            stage: *stage,
        }),
        WindowEvent::AxisMotion {
            device_id,
            axis,
            value,
        } => ErasedEvent::new(AxisMotion {
            window_id,
            device_id: *device_id,
            axis: *axis,
            value: *value,
        }),
        WindowEvent::Touch(touch) => ErasedEvent::new(Touch {
            window_id,
            touch: *touch,
        }),
        WindowEvent::ScaleFactorChanged {
            scale_factor,
            new_inner_size,
        } => ErasedEvent::new(ScaleFactorChanged {
            window_id,
            scale_factor: *scale_factor,
            suggested_inner_size: **new_inner_size,
        }),
        WindowEvent::ThemeChanged(theme) => ErasedEvent::new(ThemeChanged {
            window_id,
            theme: *theme,
        }),
        WindowEvent::DecorationsClick => ErasedEvent::new(DecorationsClick { window_id }),
        _ => return None,
    })
}

fn route_device(device_id: tao::event::DeviceId, event: &DeviceEvent) -> Option<ErasedEvent> {
    use device::*;
    Some(match event {
        DeviceEvent::Added => ErasedEvent::new(Added { device_id }),
        DeviceEvent::Removed => ErasedEvent::new(Removed { device_id }),
        DeviceEvent::MouseMotion { delta, .. } => ErasedEvent::new(MouseMotion {
            device_id,
            delta: *delta,
        }),
        DeviceEvent::MouseWheel { delta, .. } => ErasedEvent::new(MouseWheel {
            device_id,
            delta: *delta,
        }),
        DeviceEvent::Motion { axis, value, .. } => ErasedEvent::new(Motion {
            device_id,
            axis: *axis,
            value: *value,
        }),
        DeviceEvent::Button { button, state, .. } => ErasedEvent::new(Button {
            device_id,
            button: *button,
            state: *state,
        }),
        DeviceEvent::Key(event) => ErasedEvent::new(Key {
            device_id,
            event: *event,
        }),
        DeviceEvent::Text { codepoint, .. } => ErasedEvent::new(Text {
            device_id,
            codepoint: *codepoint,
        }),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_routes_keep_identity_and_owned_text() {
        // SAFETY: dummy IDs are only carried as test data, never used in Tao operations.
        let window_id = unsafe { tao::window::WindowId::dummy() };
        let input = WindowEvent::ReceivedImeText(String::from("输入文本"));
        let event = route_window(window_id, &input).unwrap();
        drop(input);
        let text = event.downcast_ref::<window::ReceivedImeText>().unwrap();
        assert_eq!(text.window_id, window_id);
        assert_eq!(text.text, "输入文本");
        assert!(event.downcast_ref::<window::Focused>().is_none());

        let input = WindowEvent::Focused(false);
        let event = route_window(window_id, &input).unwrap();
        let focus = event.downcast_ref::<window::Focused>().unwrap();
        assert_eq!(focus.window_id, window_id);
        assert!(!focus.focused);
    }

    #[test]
    #[allow(deprecated)]
    fn window_mouse_routes_preserve_device_button_and_modifiers() {
        // SAFETY: IDs are never supplied to any Tao operation.
        let window_id = unsafe { tao::window::WindowId::dummy() };
        let device_id = unsafe { tao::event::DeviceId::dummy() };
        let input = WindowEvent::MouseInput {
            device_id,
            state: tao::event::ElementState::Pressed,
            button: tao::event::MouseButton::Right,
            modifiers: tao::keyboard::ModifiersState::SHIFT,
        };
        let event = route_window(window_id, &input).unwrap();
        let mouse = event.downcast_ref::<window::MouseInput>().unwrap();
        assert_eq!((mouse.window_id, mouse.device_id), (window_id, device_id));
        assert_eq!(mouse.button, tao::event::MouseButton::Right);
        assert_eq!(mouse.state, tao::event::ElementState::Pressed);
        assert_eq!(mouse.modifiers, tao::keyboard::ModifiersState::SHIFT);
    }

    #[test]
    fn device_routes_preserve_raw_key_data_and_are_distinct_from_window_input() {
        // SAFETY: this ID is only compared as test data.
        let device_id = unsafe { tao::event::DeviceId::dummy() };
        let key = tao::event::RawKeyEvent {
            physical_key: tao::keyboard::KeyCode::KeyA,
            state: tao::event::ElementState::Released,
        };
        let event = route_device(device_id, &DeviceEvent::Key(key)).unwrap();
        let routed = event.downcast_ref::<device::Key>().unwrap();
        assert_eq!(routed.device_id, device_id);
        assert_eq!(routed.event, key);
        assert!(event.downcast_ref::<window::KeyboardInput>().is_none());
    }
}
