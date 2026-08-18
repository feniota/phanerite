use crate::route::StorageId;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashFinding {
    pub rule: String,
    pub title: String,
    pub explanation: String,
    pub evidence_lines: Vec<u32>,
    pub implicated_mod_ids: Vec<String>,
    pub suggested_memory: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashEnvironment {
    pub mc_version: String,
    pub loader: String,
    pub loader_version: String,
    pub java_name: String,
    pub java_version: u32,
    pub java_path: String,
    pub memory: u32,
    pub os: String,
    pub gpu: String,
    pub enabled_mods: Vec<(String, String)>,
    pub active_overrides: Vec<String>,
    pub source: String,
    pub aphanite_server: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashReport {
    pub storage_id: StorageId,
    pub id: String,
    pub instance_id: String,
    pub when: String,
    pub exit_code: i32,
    pub lines: Option<Vec<String>>,
    pub stderr_tail: Vec<String>,
    pub hs_err_path: Option<String>,
    pub findings: Vec<CrashFinding>,
    pub environment: CrashEnvironment,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AphaniteSummary {
    pub storage_id: StorageId,
    pub server: String,
    pub online: bool,
    pub player_count: u32,
}
#[derive(Default)]
pub struct CrashStore {
    reports: Vec<CrashReport>,
    current_storage: Option<StorageId>,
    revision: u64,
}
impl CrashStore {
    pub fn new(reports: Vec<CrashReport>) -> Self {
        Self {
            reports,
            ..Default::default()
        }
    }
    pub fn get(&self, s: StorageId, id: &str) -> Option<&CrashReport> {
        self.reports
            .iter()
            .find(|r| r.storage_id == s && r.id == id)
    }
    pub fn all(&self) -> &[CrashReport] {
        &self.reports
    }
    pub fn set_storage_context(&mut self, s: StorageId) {
        self.current_storage = Some(s)
    }
    pub fn apply_for_storage(&mut self, s: StorageId, r: Vec<CrashReport>) -> bool {
        if self.current_storage != Some(s) {
            return false;
        }
        self.reports = r;
        self.revision += 1;
        true
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
