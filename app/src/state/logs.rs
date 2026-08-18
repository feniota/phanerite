use std::collections::{HashMap, VecDeque};
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
    buffers: HashMap<String, VecDeque<LiveLogLine>>,
}
impl LiveLogStore {
    pub const MAX_LINES: usize = 10_000;
    pub fn append(&mut self, session: impl Into<String>, line: LiveLogLine) {
        let b = self.buffers.entry(session.into()).or_default();
        b.push_back(line);
        while b.len() > Self::MAX_LINES {
            b.pop_front();
        }
    }
    pub fn lines(&self, session: &str) -> Option<&VecDeque<LiveLogLine>> {
        self.buffers.get(session)
    }
    pub fn clear(&mut self, session: &str) {
        self.buffers.remove(session);
    }
}
