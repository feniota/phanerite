//! Application state models and stores shared across pages and components.

pub mod accounts;
pub mod crash;
pub mod instances;
pub mod launch;
pub mod logs;
pub mod sessions;
pub mod settings;

pub use accounts::*;
pub use crash::*;
pub use instances::*;
pub use launch::*;
pub use logs::*;
pub use sessions::*;
pub use settings::*;

use gpui::{Context, Entity, Subscription};
use phanerite_core::storage::{Storage, StorageIdent, multi::MultiStorage};

use crate::route::{InstanceRef, Navigation, Route};

/// The four mod loaders Phanerite supports. Quilt is deliberately absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Loader {
    Vanilla,
    Fabric,
    Forge,
    NeoForge,
}

impl Loader {
    pub const ALL: [Loader; 4] = [
        Loader::Vanilla,
        Loader::Fabric,
        Loader::Forge,
        Loader::NeoForge,
    ];

    /// The display label used by `LOADER_LABEL` in the prototype.
    pub fn label(self) -> &'static str {
        match self {
            Loader::Vanilla => "Vanilla",
            Loader::Fabric => "Fabric",
            Loader::Forge => "Forge",
            Loader::NeoForge => "NeoForge",
        }
    }

    /// The lowercase identifier the prototype stores and renders capitalized.
    pub fn key(self) -> &'static str {
        match self {
            Loader::Vanilla => "vanilla",
            Loader::Fabric => "fabric",
            Loader::Forge => "forge",
            Loader::NeoForge => "neoforge",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|loader| loader.key() == key)
    }
}

/// The account providers the prototype offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccountType {
    Microsoft,
    Aphanite,
    Yggdrasil,
    Offline,
}

impl AccountType {
    pub fn label(self) -> &'static str {
        match self {
            AccountType::Microsoft => "Microsoft",
            AccountType::Aphanite => "Aphanite",
            AccountType::Yggdrasil => "Yggdrasil",
            AccountType::Offline => "Offline",
        }
    }

    /// Lowercase form, matching the prototype's `account.type` strings.
    pub fn key(self) -> &'static str {
        match self {
            AccountType::Microsoft => "microsoft",
            AccountType::Aphanite => "aphanite",
            AccountType::Yggdrasil => "yggdrasil",
            AccountType::Offline => "offline",
        }
    }
}

/// App-scoped context: storage resources, navigation, and the low and medium
/// frequency stores. Launch progress and live log output are deliberately not
/// reachable from here, so no consumer of this entity can observe them.
pub struct AppState {
    /// Storage roots backed by the core concurrent resource container.
    pub storages: MultiStorage,
    /// Storage root currently selected by the app.
    pub default_storage: Option<StorageIdent>,
    /// Current route and the navigation history used by page transitions.
    pub navigation: Navigation,
    /// Instance metadata and the currently loaded instances.
    pub instances: Entity<InstanceStore>,
    /// Configured Minecraft accounts and the active account selection.
    pub accounts: Entity<AccountStore>,
    /// Application settings, preferences, and discovered Java runtimes.
    pub settings: Entity<SettingsStore>,
    /// Crash reports associated with the registered storage roots.
    pub crashes: Entity<CrashStore>,
    /// Summaries of launched and recently finished game sessions.
    pub sessions: Entity<SessionStore>,
    /// Subscriptions that forward changes from child stores to this app state.
    _subscriptions: Vec<Subscription>,
}

impl AppState {
    pub fn new(
        storages: MultiStorage,
        default_storage: Option<StorageIdent>,
        instances: Entity<InstanceStore>,
        accounts: Entity<AccountStore>,
        settings: Entity<SettingsStore>,
        crashes: Entity<CrashStore>,
        sessions: Entity<SessionStore>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_route(
            storages,
            default_storage,
            instances,
            accounts,
            settings,
            crashes,
            sessions,
            None,
            cx,
        )
    }

    pub fn new_with_route(
        storages: MultiStorage,
        default_storage: Option<StorageIdent>,
        instances: Entity<InstanceStore>,
        accounts: Entity<AccountStore>,
        settings: Entity<SettingsStore>,
        crashes: Entity<CrashStore>,
        sessions: Entity<SessionStore>,
        initial_route: Option<Route>,
        cx: &mut Context<Self>,
    ) -> Self {
        let initial = initial_route.unwrap_or_else(|| {
            if default_storage.is_some() || cfg!(feature = "seed") {
                Route::Play
            } else {
                Route::Setup
            }
        });
        // Re-emit sub-store changes so a view only has to observe `AppState`
        // to stay current with everything reachable through it.
        let _subscriptions = vec![
            cx.observe(&instances, |_, _, cx| cx.notify()),
            cx.observe(&accounts, |_, _, cx| cx.notify()),
            cx.observe(&settings, |_, _, cx| cx.notify()),
            cx.observe(&crashes, |_, _, cx| cx.notify()),
            cx.observe(&sessions, |_, _, cx| cx.notify()),
        ];
        Self {
            storages,
            default_storage,
            navigation: Navigation::new(initial),
            instances,
            accounts,
            settings,
            crashes,
            sessions,
            _subscriptions,
        }
    }

    /// The storage the UI is currently scoped to, if any root is selected.
    pub fn storage(&self) -> Option<StorageIdent> {
        self.default_storage.clone()
    }

    /// Selects a registered storage root as the default for new UI work.
    pub fn set_default_storage(&mut self, storage: StorageIdent, cx: &mut Context<Self>) -> bool {
        if self.default_storage.as_ref() == Some(&storage) || !self.storages.contains(&storage) {
            return false;
        }
        self.default_storage = Some(storage.clone());
        cx.notify();
        true
    }

    /// Registers a Storage and selects it when no default exists yet.
    // FIXME: Replace these synchronous bridges with GPUI-spawned async work
    // when storage management is connected to the real application workflow.
    pub fn add_storage(
        &mut self,
        storage: Storage,
        cx: &mut Context<Self>,
    ) -> Option<StorageIdent> {
        let key = StorageIdent::from(&storage);
        if self.storages.contains(&key)
            || gpui::block_on(self.storages.insert(key.clone(), storage)).is_err()
        {
            return None;
        }
        if self.default_storage.is_none() {
            self.default_storage = Some(key.clone());
        }
        cx.notify();
        Some(key)
    }

    /// Removes a Storage, requiring a replacement when removing the default.
    pub fn remove_storage(
        &mut self,
        storage: &StorageIdent,
        replacement: Option<StorageIdent>,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.storages.contains(storage) {
            return false;
        }
        if self.default_storage.as_ref() == Some(storage) {
            if self.storages.len() > 1 {
                let Some(replacement) = replacement.as_ref() else {
                    return false;
                };
                if replacement == storage || !self.storages.contains(replacement) {
                    return false;
                }
            } else if replacement.is_some() {
                return false;
            }
        } else if replacement.is_some() {
            return false;
        }
        if !gpui::block_on(self.storages.remove(storage)) {
            return false;
        }
        if self.default_storage.as_ref() == Some(storage) {
            self.default_storage = replacement;
        }
        cx.notify();
        true
    }

    pub fn route(&self) -> &Route {
        self.navigation.current()
    }

    pub fn push(&mut self, route: Route, cx: &mut Context<Self>) {
        self.navigation.push(route);
        cx.notify();
    }

    pub fn replace(&mut self, route: Route, cx: &mut Context<Self>) {
        self.navigation.replace(route);
        cx.notify();
    }

    pub fn back(&mut self, cx: &mut Context<Self>) {
        self.navigation.back();
        cx.notify();
    }

    /// Opens an instance detail page, ignoring references that no longer exist.
    pub fn open_instance(&mut self, reference: InstanceRef, cx: &mut Context<Self>) {
        let exists = self
            .instances
            .read(cx)
            .get(reference.storage.clone(), &reference.instance_id)
            .is_some();
        if exists {
            self.push(Route::InstanceDetail(reference), cx);
        }
    }
}
