//! Window-scoped Tao events. Positions and sizes retain Tao's physical units.

use std::path::PathBuf;

use tao::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{AxisId, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
    keyboard::ModifiersState,
    window::{Theme, WindowId},
};

use crate::event::Event;

macro_rules! window_events {
    ($($(#[$meta:meta])* $name:ident { $($field:ident: $ty:ty),* $(,)? })*) => {
        $(
            $(#[$meta])*
            #[derive(Event, Clone, Debug, PartialEq)]
            pub struct $name {
                pub window_id: WindowId,
                $(pub $field: $ty,)*
            }
        )*
    };
}

window_events! {
    Resized { size: PhysicalSize<u32> }
    Moved { position: PhysicalPosition<i32> }
    CloseRequested {}
    Destroyed {}
    Started {}
    Suspended {}
    Resumed {}
    Stopped {}
    DroppedFile { path: PathBuf }
    HoveredFile { path: PathBuf }
    HoveredFileCancelled {}
    ReceivedImeText { text: String }
    Focused { focused: bool }
    KeyboardInput { device_id: DeviceId, event: KeyEvent, is_synthetic: bool }
    ModifiersChanged { modifiers: ModifiersState }
    CursorMoved { device_id: DeviceId, position: PhysicalPosition<f64>, modifiers: ModifiersState }
    CursorEntered { device_id: DeviceId }
    CursorLeft { device_id: DeviceId }
    MouseWheel { device_id: DeviceId, delta: MouseScrollDelta, phase: TouchPhase, modifiers: ModifiersState }
    MouseInput { device_id: DeviceId, state: ElementState, button: MouseButton, modifiers: ModifiersState }
    TouchpadPressure { device_id: DeviceId, pressure: f32, stage: i64 }
    AxisMotion { device_id: DeviceId, axis: AxisId, value: f64 }
    Touch { touch: tao::event::Touch }
    /// An owned notification; suggested_inner_size cannot modify the OS callback's size.
    ScaleFactorChanged { scale_factor: f64, suggested_inner_size: PhysicalSize<u32> }
    ThemeChanged { theme: Theme }
    DecorationsClick {}
}
