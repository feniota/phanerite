use std::collections::{HashMap, VecDeque};

use super::sessions::SessionId;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogSource {
    Stdout,
    Stderr,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveLogLine {
    pub source: LogSource,
    pub text: String,
}
#[derive(Default)]
pub struct LiveLogStore {
    buffers: HashMap<SessionId, VecDeque<LiveLogLine>>,
}
impl LiveLogStore {
    pub const MAX_LINES: usize = 10_000;
    pub fn append(&mut self, session: impl Into<SessionId>, line: LiveLogLine) {
        let b = self.buffers.entry(session.into()).or_default();
        b.push_back(line);
        while b.len() > Self::MAX_LINES {
            b.pop_front();
        }
    }
    pub fn lines(&self, session: &SessionId) -> Option<&VecDeque<LiveLogLine>> {
        self.buffers.get(session)
    }
    pub fn clear(&mut self, session: &SessionId) {
        self.buffers.remove(session);
    }
}
