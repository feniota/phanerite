//! Account resources and their UI projections.

use crate::state::AccountType;
use phanerite_core::auth::{Account, AccountIdent, MultiAccount};

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

    pub fn detail(&self) -> String {
        match &self.auth_server {
            Some(server) => server.clone(),
            None => format!("last used {}", self.last_used),
        }
    }

    /// Builds the renderable account projection from a core account.
    ///
    /// The core account remains the source of truth. This projection is
    /// intentionally created on demand for UI rendering.
    // FIXME: Once the GUI is connected to the real account workflow, build
    // this projection asynchronously instead of blocking the UI with
    // `crate::block_on`.
    pub fn from_account(account: &Account) -> Self {
        let ident = account.identifier();
        let (username, account_type, auth_server, active_profile_id, profiles) = match account {
            Account::Offline(auth) => {
                let id = auth.uuid.to_string();
                (
                    auth.nickname.clone(),
                    AccountType::Offline,
                    None,
                    id.clone(),
                    vec![PlayerProfileSummary {
                        id: id.clone(),
                        name: auth.nickname.clone(),
                        skin_url: format!("https://mc-heads.net/skin/{}", auth.nickname),
                        is_slim: false,
                    }],
                )
            }
            Account::Microsoft(auth) => {
                let profile = crate::block_on(auth.profile());
                let id = profile.id.to_string();
                let (skin_url, is_slim) = profile
                    .skin()
                    .map(|skin| {
                        (
                            skin.url.to_string(),
                            matches!(
                                skin.variant,
                                phanerite_core::auth::microsoft::SkinVariant::Slim
                            ),
                        )
                    })
                    .unwrap_or_else(|| {
                        (format!("https://mc-heads.net/skin/{}", profile.name), false)
                    });
                (
                    profile.name.clone(),
                    AccountType::Microsoft,
                    None,
                    id.clone(),
                    vec![PlayerProfileSummary {
                        id,
                        name: profile.name,
                        skin_url,
                        is_slim,
                    }],
                )
            }
            Account::Yggdrasil(auth) => {
                let profiles = crate::block_on(auth.available_profiles());
                let selected = crate::block_on(auth.selected_profile());
                let selected_id = auth
                    .selected()
                    .map(|id| id.to_string())
                    .or_else(|| selected.as_ref().map(|profile| profile.id.to_string()))
                    .unwrap_or_else(|| auth.username.clone());
                let profiles = profiles
                    .into_iter()
                    .map(|profile| {
                        let id = profile.id.to_string();
                        let name = profile.name.unwrap_or_else(|| auth.username.clone());
                        PlayerProfileSummary {
                            skin_url: format!("https://mc-heads.net/skin/{name}"),
                            id,
                            name,
                            is_slim: false,
                        }
                    })
                    .collect();
                (
                    auth.username.clone(),
                    AccountType::Yggdrasil,
                    Some(auth.server.to_string()),
                    selected_id,
                    profiles,
                )
            }
        };
        Self {
            id: ident.to_string(),
            username,
            account_type,
            last_used: "just now".into(),
            auth_server,
            active_profile_id,
            profiles,
        }
    }
}

#[derive(Default)]
pub struct AccountStore {
    accounts: MultiAccount,
    active_id: Option<AccountIdent>,
    revision: u64,
    #[cfg(feature = "seed")]
    preview_accounts: Vec<AccountSummary>,
    #[cfg(feature = "seed")]
    preview_active_id: Option<String>,
}

impl AccountStore {
    pub fn new(accounts: Vec<Account>) -> Self {
        let store = Self::default();
        for account in accounts {
            let key = account.identifier();
            assert!(
                crate::block_on(store.accounts.insert(key, account)).is_ok(),
                "seed account key must be unique"
            );
        }
        store
    }

    pub fn all(&self) -> Vec<AccountSummary> {
        let mut accounts = Vec::new();
        self.accounts.for_each(|_, account| {
            accounts.push(AccountSummary::from_account(account));
        });
        #[cfg(feature = "seed")]
        accounts.extend(self.preview_accounts.iter().cloned());
        accounts.sort_by(|left, right| {
            left.username
                .cmp(&right.username)
                .then(left.id.cmp(&right.id))
        });
        accounts
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let count = self.accounts.len();
        #[cfg(feature = "seed")]
        let count = count + self.preview_accounts.len();
        count
    }

    fn key_for_id(&self, id: &str) -> Option<AccountIdent> {
        let mut result = None;
        self.accounts.for_each(|key, account| {
            if AccountSummary::from_account(account).id == id {
                result = Some(key.clone());
            }
        });
        result
    }

    pub fn get(&self, id: &str) -> Option<AccountSummary> {
        #[cfg(feature = "seed")]
        if let Some(account) = self
            .preview_accounts
            .iter()
            .find(|account| account.id == id)
        {
            return Some(account.clone());
        }
        let key = self.key_for_id(id)?;
        self.accounts
            .try_get(&key)
            .map(|account| AccountSummary::from_account(&account))
    }

    pub fn active_id(&self) -> Option<String> {
        #[cfg(feature = "seed")]
        if self.preview_active_id.is_some() {
            return self.preview_active_id.clone();
        }
        self.active_id.as_ref().map(ToString::to_string)
    }

    pub fn active(&self) -> Option<AccountSummary> {
        #[cfg(feature = "seed")]
        if let Some(id) = &self.preview_active_id {
            return self.get(id);
        }
        let key = self.active_id.as_ref()?;
        self.accounts
            .try_get(key)
            .map(|account| AccountSummary::from_account(&account))
    }

    pub fn active_profile(&self) -> Option<PlayerProfileSummary> {
        let account = self.active()?;
        account.active_profile().cloned()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn set_active(&mut self, id: &str) -> bool {
        #[cfg(feature = "seed")]
        if self.preview_accounts.iter().any(|account| account.id == id) {
            if self.preview_active_id.as_deref() == Some(id) {
                return false;
            }
            self.preview_active_id = Some(id.to_owned());
            self.active_id = None;
            self.revision += 1;
            return true;
        }
        let Some(key) = self.key_for_id(id) else {
            return false;
        };
        if self.active_id.as_ref() == Some(&key) {
            return false;
        }
        self.active_id = Some(key);
        #[cfg(feature = "seed")]
        {
            self.preview_active_id = None;
        }
        self.revision += 1;
        true
    }

    pub fn set_active_profile(&mut self, _id: &str, _profile: impl Into<String>) -> bool {
        #[cfg(feature = "seed")]
        if let Some(account) = self
            .preview_accounts
            .iter_mut()
            .find(|account| account.id == _id)
        {
            let profile = _profile.into();
            if account.active_profile_id == profile
                || !account.profiles.iter().any(|item| item.id == profile)
            {
                return false;
            }
            account.active_profile_id = profile;
            self.revision += 1;
            return true;
        }
        // Profile selection belongs to the core authentication object and is
        // asynchronous. The GUI currently only exposes the active profile.
        false
    }

    /// Adds an offline account and makes it active.
    pub fn add(
        &mut self,
        username: impl Into<String>,
        account_type: AccountType,
        _auth_server: Option<String>,
        _profiles: Vec<PlayerProfileSummary>,
        _active_profile_id: Option<String>,
    ) -> Option<String> {
        if account_type != AccountType::Offline {
            return None;
        }
        let account = Account::Offline(phanerite_core::auth::offline::Authentication::new(
            username.into(),
        ));
        let key = account.identifier();
        let id = key.to_string();
        if crate::block_on(self.accounts.insert(key.clone(), account)).is_err() {
            return None;
        }
        self.active_id = Some(key);
        #[cfg(feature = "seed")]
        {
            self.preview_active_id = None;
        }
        self.revision += 1;
        Some(id)
    }

    pub fn remove(&mut self, id: &str) -> bool {
        #[cfg(feature = "seed")]
        if self.preview_accounts.iter().any(|account| account.id == id) {
            self.preview_accounts.retain(|account| account.id != id);
            if self.preview_active_id.as_deref() == Some(id) {
                self.preview_active_id = None;
                if let Some(account) = self.all().first() {
                    self.set_active(&account.id);
                }
            }
            self.revision += 1;
            return true;
        }
        let Some(key) = self.key_for_id(id) else {
            return false;
        };
        if !crate::block_on(self.accounts.remove(&key)) {
            return false;
        }
        if self.active_id.as_ref() == Some(&key) {
            self.active_id = None;
            if let Some(account) = self.all().first() {
                self.set_active(&account.id);
            }
        }
        self.revision += 1;
        true
    }

    /// In-memory profiles for exercising provider UI without authentication.
    /// These never enter the core credential store or launch requests.
    #[cfg(feature = "seed")]
    pub fn add_preview(
        &mut self,
        username: String,
        account_type: AccountType,
        auth_server: Option<String>,
        profiles: Vec<PlayerProfileSummary>,
        active_profile_id: String,
    ) -> Option<String> {
        if account_type == AccountType::Offline
            || username.trim().is_empty()
            || !profiles
                .iter()
                .any(|profile| profile.id == active_profile_id)
        {
            return None;
        }
        let id = format!(
            "preview:{}:{}:{username}",
            account_type.key(),
            auth_server.as_deref().unwrap_or_default()
        );
        if self.get(&id).is_some() {
            return None;
        }
        self.preview_accounts.push(AccountSummary {
            id: id.clone(),
            username,
            account_type,
            last_used: "just now".into(),
            auth_server,
            active_profile_id,
            profiles,
        });
        self.set_active(&id);
        Some(id)
    }
}
