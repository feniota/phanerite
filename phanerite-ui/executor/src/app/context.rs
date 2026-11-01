//! Shared services available while the application is running.

use taao::event::EventSubmitter;
use wgpu::Instance;

use crate::event::EventHandle;

pub struct Context {
    pub submitter: EventSubmitter,
    pub events: EventHandle,
    pub instance: Instance,
}
