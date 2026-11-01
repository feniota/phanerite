//! Application runtime, generic event dispatch, and Tao platform integration.

extern crate self as phanerite_executor;
mod app;
pub mod event;
pub mod tao_event;
pub mod window;

pub use app::{Application, Context, Shutdown};
