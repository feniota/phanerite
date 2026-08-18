use crate::{route::StorageId, state::StorageRegistry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModSummary {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub file_name: String,
    pub loader: Option<String>,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePackSummary {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub size: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderPackSummary {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub gpu: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSummary {
    pub id: String,
    pub name: String,
    pub seed: String,
    pub version: String,
    pub last_played: String,
    pub players: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaRuntimeSummary {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub path: String,
    pub managed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceLaunchOverrides {
    pub memory: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceSummary {
    pub storage_id: StorageId,
    pub id: String,
    pub icon_seed: u64,
    pub name: String,
    pub aphanite: bool,
    pub favorite: bool,
    pub description: String,
    pub loader: String,
    pub mc_version: String,
    pub loader_version: String,
    pub java: String,
    pub java_runtime_id: String,
    pub created_at: String,
    pub last_played: Option<String>,
    pub play_count: u32,
    pub last_crash_id: Option<String>,
    pub launch_overrides: InstanceLaunchOverrides,
    pub mods: Vec<ModSummary>,
    pub resource_packs: Vec<ResourcePackSummary>,
    pub shader_packs: Vec<ShaderPackSummary>,
    pub worlds: Vec<WorldSummary>,
}

#[derive(Default)]
pub struct InstanceStore {
    instances: Vec<InstanceSummary>,
    current_storage: Option<StorageId>,
    revision: u64,
}
impl InstanceStore {
    pub fn new(instances: Vec<InstanceSummary>) -> Self {
        Self {
            instances,
            ..Default::default()
        }
    }
    pub fn all(&self) -> &[InstanceSummary] {
        &self.instances
    }
    pub fn get(&self, storage: StorageId, id: &str) -> Option<&InstanceSummary> {
        self.instances
            .iter()
            .find(|i| i.storage_id == storage && i.id == id)
    }
    pub fn set_storage_context(&mut self, id: StorageId) {
        self.current_storage = Some(id);
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn set_favorite(&mut self, storage: StorageId, id: &str, value: bool) -> bool {
        self.mutate(storage, id, |i| &mut i.favorite, value)
    }
    pub fn set_mod_enabled(
        &mut self,
        storage: StorageId,
        instance: &str,
        mod_id: &str,
        value: bool,
    ) -> bool {
        if self.current_storage != Some(storage) {
            return false;
        }
        if let Some(m) = self
            .instances
            .iter_mut()
            .find(|i| i.storage_id == storage && i.id == instance)
            .and_then(|i| i.mods.iter_mut().find(|m| m.id == mod_id))
        {
            if m.enabled == value {
                return false;
            }
            m.enabled = value;
            self.revision += 1;
            return true;
        }
        false
    }
    fn mutate(
        &mut self,
        storage: StorageId,
        id: &str,
        f: impl FnOnce(&mut InstanceSummary) -> &mut bool,
        value: bool,
    ) -> bool {
        if self.current_storage != Some(storage) {
            return false;
        }
        if let Some(i) = self
            .instances
            .iter_mut()
            .find(|i| i.storage_id == storage && i.id == id)
        {
            let field = f(i);
            if *field == value {
                return false;
            }
            *field = value;
            self.revision += 1;
            return true;
        }
        false
    }
    pub fn apply_for_storage(
        &mut self,
        registry: &StorageRegistry,
        storage: StorageId,
        result: Vec<InstanceSummary>,
    ) -> bool {
        if registry.get(storage).is_some()
            && self.current_storage == Some(storage)
            && self.instances != result
        {
            self.instances = result;
            self.revision += 1;
            true
        } else {
            false
        }
    }
}
