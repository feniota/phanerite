//! Event-loop events. Taao 0.1.3 filters NewEvents, MainEventsCleared,
//! RedrawEventsCleared, LoopDestroyed, and internal UserEvent wake notifications.
//! Those types can be triggered explicitly but will not arrive from that backend.

use tao::{event::StartCause, window::WindowId};

use crate::event::Event;

#[derive(Event, Clone, Debug, PartialEq)]
pub struct NewEvents {
    pub cause: StartCause,
}

#[derive(Event, Clone, Debug, PartialEq)]
pub struct UserEvent;

#[derive(Event, Clone, Debug, PartialEq)]
pub struct MainEventsCleared;

#[derive(Event, Clone, Debug, PartialEq)]
pub struct RedrawRequested {
    pub window_id: WindowId,
}

#[derive(Event, Clone, Debug, PartialEq)]
pub struct RedrawEventsCleared;

#[derive(Event, Clone, Debug, PartialEq)]
pub struct LoopDestroyed;

#[derive(Event, Clone, Debug, PartialEq)]
pub struct Opened {
    pub urls: Vec<url::Url>,
}

#[derive(Event, Clone, Debug, PartialEq)]
pub struct Reopen {
    pub has_visible_windows: bool,
}
