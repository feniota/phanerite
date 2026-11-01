//! Native window creation on the platform event loop.

use std::sync::Arc;
use taao::event::Submitter;
use tao::window::WindowBuilder as TaoWindowBuilder;

use super::Window;
use crate::Context;

pub struct WindowBuilder {
    window: TaoWindowBuilder,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            window: TaoWindowBuilder::default().with_title("PhaneriteUI"),
        }
    }
}

impl WindowBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_window_builder<F>(mut self, op: F) -> Self
    where
        F: FnOnce(TaoWindowBuilder) -> TaoWindowBuilder,
    {
        self.window = op(self.window);
        self
    }
    pub async fn build(self, cx: &Context) -> Result<Window, taao::event::SubmitError> {
        // Create window
        let window = cx
            .submitter
            .submit(|event, _| self.window.build(event))
            .await?
            .expect("OS Error: failed to create window");
        let window = Arc::new(window);

        Ok(Window { window })
    }
}
