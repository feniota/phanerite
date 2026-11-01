//! Errors returned when submitting events and observer commands.

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ObserveError {
    #[error("event dispatcher has been dropped")]
    DispatcherDropped,
    #[error("observer belongs to another dispatcher")]
    WrongDispatcher,
}

/// Failure to enqueue an event. The rejected event is dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum EmitError {
    #[error("event queue is full")]
    Full,
    #[error("event queue is closed")]
    Closed,
}
