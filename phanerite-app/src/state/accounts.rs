//! Account profiles and account store operations.

use crate::state::AccountType;

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
    pub account_type: AccountType,
    pub last_used: String,
    /// Present for Aphanite and third-party Yggdrasil accounts.
    pub auth_server: Option<String>,
    pub active_profile_id: String,
    pub profiles: Vec<PlayerProfileSummary>,
}

impl AccountSummary {
    pub fn active_profile(&self) -> Option<&PlayerProfileSummary> {
        self.profiles
            .iter()
            .find(|profile| profile.id == self.active_profile_id)
    }

    /// The secondary line on the accounts page.
    pub fn detail(&self) -> String {
        match &self.auth_server {
            Some(server) => server.clone(),
            None => format!("last used {}", self.last_used),
        }
    }
}

#[derive(Default)]
pub struct AccountStore {
    accounts: Vec<AccountSummary>,
    active_id: Option<String>,
    next_id: u64,
    revision: u64,
}

impl AccountStore {
    pub fn new(accounts: Vec<AccountSummary>) -> Self {
        let active_id = accounts.first().map(|account| account.id.clone());
        Self {
            accounts,
            active_id,
            ..Default::default()
        }
    }

    pub fn all(&self) -> &[AccountSummary] {
        &self.accounts
    }

    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    pub fn get(&self, id: &str) -> Option<&AccountSummary> {
        self.accounts.iter().find(|account| account.id == id)
    }

    pub fn active_id(&self) -> Option<&str> {
        self.active_id.as_deref()
    }

    pub fn active(&self) -> Option<&AccountSummary> {
        self.active_id.as_deref().and_then(|id| self.get(id))
    }

    pub fn active_profile(&self) -> Option<&PlayerProfileSummary> {
        self.active().and_then(AccountSummary::active_profile)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn allocate_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{:08x}", self.next_id)
    }

    pub fn set_active(&mut self, id: &str) -> bool {
        if self.get(id).is_none() || self.active_id.as_deref() == Some(id) {
            return false;
        }
        self.active_id = Some(id.to_string());
        if let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) {
            account.last_used = "just now".into();
        }
        self.revision += 1;
        true
    }

    pub fn set_active_profile(&mut self, id: &str, profile: impl Into<String>) -> bool {
        let profile = profile.into();
        let Some(account) = self.accounts.iter_mut().find(|account| account.id == id) else {
            return false;
        };
        if account.active_profile_id == profile
            || !account.profiles.iter().any(|item| item.id == profile)
        {
            return false;
        }
        account.active_profile_id = profile;
        self.revision += 1;
        true
    }

    /// Adds an account and makes it active, as the prototype's add flow does.
    pub fn add(
        &mut self,
        username: impl Into<String>,
        account_type: AccountType,
        auth_server: Option<String>,
        profiles: Vec<PlayerProfileSummary>,
        active_profile_id: Option<String>,
    ) -> Option<String> {
        let username = username.into();
        if username.trim().is_empty() {
            return None;
        }
        let id = self.allocate_id("acc");
        let profiles = if profiles.is_empty() {
            let profile_id = self.allocate_id("profile");
            vec![PlayerProfileSummary {
                id: profile_id,
                name: username.clone(),
                skin_url: format!("https://mc-heads.net/skin/{username}"),
                is_slim: false,
            }]
        } else {
            profiles
        };
        let active_profile_id = active_profile_id.unwrap_or_else(|| profiles[0].id.clone());
        self.accounts.push(AccountSummary {
            id: id.clone(),
            username,
            account_type,
            last_used: "just now".into(),
            auth_server,
            active_profile_id,
            profiles,
        });
        self.active_id = Some(id.clone());
        self.revision += 1;
        Some(id)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.accounts.len();
        self.accounts.retain(|account| account.id != id);
        if before == self.accounts.len() {
            return false;
        }
        if self.active_id.as_deref() == Some(id) {
            self.active_id = self.accounts.first().map(|account| account.id.clone());
        }
        self.revision += 1;
        true
    }
}
