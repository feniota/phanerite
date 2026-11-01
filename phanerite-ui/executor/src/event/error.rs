//! Errors returned when submitting events and observer commands.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserveError {
    DispatcherDropped,
    WrongDispatcher,
}

impl std::fmt::Display for ObserveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DispatcherDropped => "event dispatcher has been dropped",
            Self::WrongDispatcher => "observer belongs to another dispatcher",
        })
    }
}

impl std::error::Error for ObserveError {}

/// Failure to enqueue an event. The rejected event is dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitError {
    Full,
    Closed,
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Full => "event queue is full",
            Self::Closed => "event queue is closed",
        })
    }
}

impl std::error::Error for EmitError {}
