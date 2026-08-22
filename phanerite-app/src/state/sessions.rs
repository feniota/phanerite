//! Running-process session identities and lifecycle state.

use crate::route::InstanceRef;

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
    pub instance: InstanceRef,
    pub started_at: String,
    pub exit_code: Option<i32>,
    pub running: bool,
}

/// Process lifecycle for launched instances. Multi-instance is first class, so
/// several sessions may be running at once.
#[derive(Default)]
pub struct SessionStore {
    sessions: Vec<SessionSummary>,
    revision: u64,
}

impl SessionStore {
    pub fn all(&self) -> &[SessionSummary] {
        &self.sessions
    }

    pub fn running(&self) -> impl Iterator<Item = &SessionSummary> {
        self.sessions.iter().filter(|session| session.running)
    }

    pub fn running_count(&self) -> usize {
        self.running().count()
    }

    pub fn is_running(&self, instance: &InstanceRef) -> bool {
        self.sessions
            .iter()
            .any(|session| session.running && &session.instance == instance)
    }

    pub fn session_for(&self, instance: &InstanceRef) -> Option<&SessionSummary> {
        self.sessions
            .iter()
            .find(|session| session.running && &session.instance == instance)
    }

    pub fn get(&self, id: &SessionId) -> Option<&SessionSummary> {
        self.sessions.iter().find(|session| &session.id == id)
    }

    pub fn start(&mut self, session: SessionSummary) -> bool {
        if self.sessions.iter().any(|item| item.id == session.id) {
            return false;
        }
        self.sessions.push(session);
        self.revision += 1;
        true
    }

    pub fn finish(&mut self, id: &SessionId, code: i32) -> bool {
        let Some(session) = self.sessions.iter_mut().find(|session| &session.id == id) else {
            return false;
        };
        if !session.running && session.exit_code == Some(code) {
            return false;
        }
        session.running = false;
        session.exit_code = Some(code);
        self.revision += 1;
        true
    }

    /// Stops every running session of an instance, as the Stop action does.
    pub fn stop_instance(&mut self, instance: &InstanceRef, code: i32) -> bool {
        let mut changed = false;
        for session in self.sessions.iter_mut() {
            if session.running && &session.instance == instance {
                session.running = false;
                session.exit_code = Some(code);
                changed = true;
            }
        }
        if changed {
            self.revision += 1;
        }
        changed
    }

    /// Drops the history of an instance that no longer exists.
    pub fn remove_instance(&mut self, instance: &InstanceRef) -> bool {
        let before = self.sessions.len();
        self.sessions
            .retain(|session| &session.instance != instance);
        if self.sessions.len() == before {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}
