#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

impl From<&str> for SessionId {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for SessionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionSummary {
    pub id: SessionId,
    pub instance_id: String,
    pub started_at: String,
    pub exit_code: Option<i32>,
    pub running: bool,
}
#[derive(Default)]
pub struct SessionStore {
    sessions: Vec<SessionSummary>,
    revision: u64,
}
impl SessionStore {
    pub fn all(&self) -> &[SessionSummary] {
        &self.sessions
    }
    pub fn start(&mut self, s: SessionSummary) -> bool {
        if self.sessions.iter().any(|existing| existing.id == s.id) {
            return false;
        }
        self.sessions.push(s);
        self.revision += 1;
        true
    }
    pub fn finish(&mut self, id: &str, code: i32) -> bool {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id.as_ref() == id) {
            if !s.running && s.exit_code == Some(code) {
                return false;
            }
            s.running = false;
            s.exit_code = Some(code);
            self.revision += 1;
            return true;
        }
        false
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
