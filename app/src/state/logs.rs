//! Bounded live-log buffers and parsing helpers for process output.

use std::collections::{HashMap, VecDeque};

use super::sessions::SessionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSource {
    Stdout,
    Stderr,
}

/// Severity parsed out of a Minecraft log line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warning",
            LogLevel::Error => "Error",
            LogLevel::Debug => "Debug",
        }
    }

    /// The prototype matches on the `[thread/LEVEL]` marker inside the line.
    pub fn of(line: &str) -> LogLevel {
        if line.contains("/ERROR]") {
            LogLevel::Error
        } else if line.contains("/WARN]") {
            LogLevel::Warn
        } else if line.contains("/DEBUG]") {
            LogLevel::Debug
        } else {
            LogLevel::Info
        }
    }

    /// Case-insensitive marker used by the level filter, e.g. `/warn]`.
    pub fn marker(self) -> &'static str {
        match self {
            LogLevel::Info => "/info]",
            LogLevel::Warn => "/warn]",
            LogLevel::Error => "/error]",
            LogLevel::Debug => "/debug]",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveLogLine {
    pub source: LogSource,
    pub text: String,
}

impl LiveLogLine {
    pub fn stdout(text: impl Into<String>) -> Self {
        Self {
            source: LogSource::Stdout,
            text: text.into(),
        }
    }

    pub fn stderr(text: impl Into<String>) -> Self {
        Self {
            source: LogSource::Stderr,
            text: text.into(),
        }
    }
}

/// Live process output, one bounded ring buffer per session. This never
/// replaces file-log loading; `latest.log` and `debug.log` are read separately.
#[derive(Default)]
pub struct LiveLogStore {
    buffers: HashMap<SessionId, VecDeque<LiveLogLine>>,
    revision: u64,
}

impl LiveLogStore {
    pub const MAX_LINES: usize = 10_000;

    pub fn append(&mut self, session: impl Into<SessionId>, line: LiveLogLine) {
        self.append_batch(session, [line]);
    }

    /// Appends a batch produced by the log pump. Batching keeps a chatty JVM
    /// from turning every line into its own UI update.
    pub fn append_batch(
        &mut self,
        session: impl Into<SessionId>,
        lines: impl IntoIterator<Item = LiveLogLine>,
    ) -> usize {
        let buffer = self.buffers.entry(session.into()).or_default();
        let mut appended = 0;
        for line in lines {
            buffer.push_back(line);
            appended += 1;
            while buffer.len() > Self::MAX_LINES {
                buffer.pop_front();
            }
        }
        if appended > 0 {
            self.revision += 1;
        }
        appended
    }

    pub fn lines(&self, session: &SessionId) -> Option<&VecDeque<LiveLogLine>> {
        self.buffers.get(session)
    }

    pub fn len(&self, session: &SessionId) -> usize {
        self.buffers.get(session).map_or(0, VecDeque::len)
    }

    pub fn clear(&mut self, session: &SessionId) {
        if self.buffers.remove(session).is_some() {
            self.revision += 1;
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}
