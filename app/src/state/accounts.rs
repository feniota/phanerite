#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerProfileSummary {
    pub id: String,
    pub name: String,
    pub skin_url: String,
    pub is_slim: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountSummary {
    pub id: String,
    pub username: String,
    pub account_type: String,
    pub last_used: String,
    pub active_profile_id: String,
    pub profiles: Vec<PlayerProfileSummary>,
}
#[derive(Default)]
pub struct AccountStore {
    accounts: Vec<AccountSummary>,
    revision: u64,
}
impl AccountStore {
    pub fn new(accounts: Vec<AccountSummary>) -> Self {
        Self {
            accounts,
            ..Default::default()
        }
    }
    pub fn all(&self) -> &[AccountSummary] {
        &self.accounts
    }
    pub fn get(&self, id: &str) -> Option<&AccountSummary> {
        self.accounts.iter().find(|a| a.id == id)
    }
    pub fn set_active_profile(&mut self, id: &str, profile: impl Into<String>) -> bool {
        let p = profile.into();
        if let Some(a) = self.accounts.iter_mut().find(|a| a.id == id) {
            if a.active_profile_id == p {
                return false;
            }
            a.active_profile_id = p;
            self.revision += 1;
            return true;
        }
        false
    }
    pub fn remove(&mut self, id: &str) -> bool {
        let n = self.accounts.len();
        self.accounts.retain(|a| a.id != id);
        if n != self.accounts.len() {
            self.revision += 1;
            true
        } else {
            false
        }
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
}
