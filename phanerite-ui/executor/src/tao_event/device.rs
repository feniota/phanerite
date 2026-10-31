//! Raw hardware events. These are independent of window focus and window input.

use tao::event::{AxisId, ButtonId, DeviceId, ElementState, MouseScrollDelta, RawKeyEvent};

use crate::event::Event;

macro_rules! device_events {
    ($($name:ident { $($field:ident: $ty:ty),* $(,)? })*) => {
        $(
            #[derive(Event, Clone, Debug, PartialEq)]
            pub struct $name {
                pub device_id: DeviceId,
                $(pub $field: $ty,)*
            }
        )*
    };
}

device_events! {
    Added {}
    Removed {}
    MouseMotion { delta: (f64, f64) }
    MouseWheel { delta: MouseScrollDelta }
    Motion { axis: AxisId, value: f64 }
    Button { button: ButtonId, state: ElementState }
    Key { event: RawKeyEvent }
    Text { codepoint: char }
}
