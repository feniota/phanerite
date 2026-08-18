#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionSummary {
    pub id: String,
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
        self.sessions.push(s);
        self.revision += 1;
        true
    }
    pub fn finish(&mut self, id: &str, code: i32) -> bool {
        if let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) {
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
