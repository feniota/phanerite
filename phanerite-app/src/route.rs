//! Navigation routes and stable references to storage-backed resources.

use std::fmt;

/// A stable identifier for an phanerite-app-managed storage entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StorageId(u64);

impl StorageId {
    /// Creates an identifier. App code should allocate these through its registry.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Constructor used by pure contract tests and seed data.
    #[doc(hidden)]
    pub const fn for_test(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for StorageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

pub type InstanceId = String;
pub type CrashReportId = String;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceRef {
    pub storage_id: StorageId,
    pub instance_id: InstanceId,
}

impl InstanceRef {
    pub fn new(storage_id: StorageId, instance_id: impl Into<String>) -> Self {
        Self {
            storage_id,
            instance_id: instance_id.into(),
        }
    }

    #[doc(hidden)]
    pub fn for_test(storage_id: StorageId, instance_id: impl Into<String>) -> Self {
        Self::new(storage_id, instance_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CrashRef {
    pub storage_id: StorageId,
    pub report_id: CrashReportId,
}

impl CrashRef {
    pub fn new(storage_id: StorageId, report_id: impl Into<String>) -> Self {
        Self {
            storage_id,
            report_id: report_id.into(),
        }
    }

    #[doc(hidden)]
    pub fn for_test(storage_id: StorageId, report_id: impl Into<String>) -> Self {
        Self::new(storage_id, report_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Setup,
    Play,
    Instances,
    Aphanite,
    InstanceDetail(InstanceRef),
    Mods(InstanceRef),
    Packs(InstanceRef),
    Shaders(InstanceRef),
    Worlds(InstanceRef),
    Logs(InstanceRef),
    LaunchSettings(InstanceRef),
    Crash(CrashRef),
    Accounts,
    Settings,
}

pub struct Navigation {
    current: Route,
    stack: Vec<Route>,
}

impl Navigation {
    pub fn new(initial: Route) -> Self {
        Self {
            current: initial,
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, route: Route) {
        if self.current == route {
            return;
        }

        // Keep the root entry (the first route) while bounding the total
        // number of entries, including the current route, to 32.
        if self.stack.len() >= 31 {
            self.stack.remove(1);
        }
        self.stack.push(self.current.clone());
        self.current = route;
    }

    pub fn back(&mut self) {
        self.current = self.stack.pop().unwrap_or(Route::Play);
    }

    pub fn replace(&mut self, route: Route) {
        self.current = route;
    }

    pub fn current(&self) -> &Route {
        &self.current
    }

    pub fn history_len(&self) -> usize {
        self.stack.len() + 1
    }
}
