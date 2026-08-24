//! Minecraft instance models and instance store operations.

use crate::{
    route::{InstanceRef, StorageIdent},
    state::{LaunchOverrides, Loader},
};
use phanerite_core::storage::multi::MultiStorage;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModSummary {
    pub id: String,
    /// `None` when the jar metadata could not be read.
    pub name: Option<String>,
    pub version: Option<String>,
    pub file_name: String,
    pub loader: Option<Loader>,
    pub enabled: bool,
}

impl ModSummary {
    /// The label the prototype falls back to when metadata is unreadable.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.file_name)
    }
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
    pub version_string: String,
    pub path: PathBuf,
    pub managed: bool,
}

impl JavaRuntimeSummary {
    /// Projects the core runtime into owned presentation data. The phanerite-app never
    /// stores a borrowed core runtime or loses the executable path.
    pub fn from_core(runtime: &phanerite_core::runtime::java::JavaRuntime) -> Self {
        Self {
            id: runtime.path.to_string_lossy().into_owned(),
            name: runtime.name.clone(),
            version: runtime.major,
            version_string: runtime.version.clone(),
            path: runtime.path.clone(),
            managed: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceSummary {
    pub storage: StorageIdent,
    pub id: String,
    /// Deterministic seed string for the pixel crystal icon, in the
    /// prototype's `name|mcVersion|loader` form.
    pub icon_seed: String,
    pub name: String,
    pub aphanite: bool,
    pub favorite: bool,
    pub description: String,
    pub loader: Loader,
    pub mc_version: String,
    pub loader_version: String,
    pub java: String,
    pub java_runtime_id: String,
    pub created_at: String,
    pub last_played: Option<String>,
    pub play_count: u32,
    pub last_crash_id: Option<String>,
    pub launch_overrides: LaunchOverrides,
    pub mods: Vec<ModSummary>,
    pub resource_packs: Vec<ResourcePackSummary>,
    pub shader_packs: Vec<ShaderPackSummary>,
    pub worlds: Vec<WorldSummary>,
}

impl InstanceSummary {
    pub fn reference(&self) -> InstanceRef {
        InstanceRef::new(self.storage.clone(), self.id.clone())
    }

    pub fn enabled_mods(&self) -> usize {
        self.mods.iter().filter(|item| item.enabled).count()
    }

    /// `Fabric 0.115.1+1.21.4`, or just `Vanilla` when there is no loader.
    pub fn loader_label(&self) -> String {
        if self.loader == Loader::Vanilla {
            self.loader.label().to_string()
        } else {
            format!("{} {}", self.loader.label(), self.loader_version)
        }
    }
}

#[derive(Default)]
pub struct InstanceStore {
    instances: Vec<InstanceSummary>,
    current_storage: Option<StorageIdent>,
    next_id: u64,
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

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn get(&self, storage: StorageIdent, id: &str) -> Option<&InstanceSummary> {
        self.instances
            .iter()
            .find(|instance| instance.storage == storage && instance.id == id)
    }

    pub fn find(&self, reference: &InstanceRef) -> Option<&InstanceSummary> {
        self.get(reference.storage.clone(), &reference.instance_id)
    }

    pub fn favorites(&self) -> impl Iterator<Item = &InstanceSummary> {
        self.instances.iter().filter(|instance| instance.favorite)
    }

    pub fn local(&self) -> impl Iterator<Item = &InstanceSummary> {
        self.instances
            .iter()
            .filter(|instance| !instance.aphanite && !instance.favorite)
    }

    pub fn aphanite_unfavorited(&self) -> impl Iterator<Item = &InstanceSummary> {
        self.instances
            .iter()
            .filter(|instance| instance.aphanite && !instance.favorite)
    }

    pub fn aphanite(&self) -> impl Iterator<Item = &InstanceSummary> {
        self.instances.iter().filter(|instance| instance.aphanite)
    }

    pub fn set_storage_context(&mut self, id: StorageIdent) {
        self.current_storage = Some(id);
    }

    pub fn storage_context(&self) -> Option<StorageIdent> {
        self.current_storage.clone()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn instance_mut(&mut self, reference: &InstanceRef) -> Option<&mut InstanceSummary> {
        if self.current_storage != Some(reference.storage.clone()) {
            return None;
        }
        self.instances.iter_mut().find(|instance| {
            instance.storage == reference.storage && instance.id == reference.instance_id
        })
    }

    /// Allocates a deterministic id inside this store, mirroring the
    /// prototype's `inst-xxxxxxxx` shape without depending on randomness.
    fn allocate_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{:08x}", self.next_id)
    }

    pub fn set_favorite(&mut self, reference: &InstanceRef, value: bool) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        if instance.favorite == value {
            return false;
        }
        instance.favorite = value;
        self.revision += 1;
        true
    }

    pub fn toggle_favorite(&mut self, reference: &InstanceRef) -> bool {
        let Some(current) = self.find(reference).map(|instance| instance.favorite) else {
            return false;
        };
        self.set_favorite(reference, !current)
    }

    pub fn set_mod_enabled(&mut self, reference: &InstanceRef, mod_id: &str, value: bool) -> bool {
        let Some(entry) = self
            .instance_mut(reference)
            .and_then(|instance| instance.mods.iter_mut().find(|item| item.id == mod_id))
        else {
            return false;
        };
        if entry.enabled == value {
            return false;
        }
        entry.enabled = value;
        self.revision += 1;
        true
    }

    pub fn set_resource_pack_enabled(
        &mut self,
        reference: &InstanceRef,
        pack_id: &str,
        value: bool,
    ) -> bool {
        let Some(entry) = self.instance_mut(reference).and_then(|instance| {
            instance
                .resource_packs
                .iter_mut()
                .find(|item| item.id == pack_id)
        }) else {
            return false;
        };
        if entry.enabled == value {
            return false;
        }
        entry.enabled = value;
        self.revision += 1;
        true
    }

    pub fn set_shader_pack_enabled(
        &mut self,
        reference: &InstanceRef,
        pack_id: &str,
        value: bool,
    ) -> bool {
        let Some(entry) = self.instance_mut(reference).and_then(|instance| {
            instance
                .shader_packs
                .iter_mut()
                .find(|item| item.id == pack_id)
        }) else {
            return false;
        };
        if entry.enabled == value {
            return false;
        }
        entry.enabled = value;
        self.revision += 1;
        true
    }

    pub fn remove_mod(&mut self, reference: &InstanceRef, mod_id: &str) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        let before = instance.mods.len();
        instance.mods.retain(|item| item.id != mod_id);
        if instance.mods.len() == before {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn remove_resource_pack(&mut self, reference: &InstanceRef, pack_id: &str) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        let before = instance.resource_packs.len();
        instance.resource_packs.retain(|item| item.id != pack_id);
        if instance.resource_packs.len() == before {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn remove_shader_pack(&mut self, reference: &InstanceRef, pack_id: &str) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        let before = instance.shader_packs.len();
        instance.shader_packs.retain(|item| item.id != pack_id);
        if instance.shader_packs.len() == before {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn remove_world(&mut self, reference: &InstanceRef, world_id: &str) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        let before = instance.worlds.len();
        instance.worlds.retain(|item| item.id != world_id);
        if instance.worlds.len() == before {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn add_mods(&mut self, reference: &InstanceRef, mods: Vec<ModSummary>) -> bool {
        if mods.is_empty() {
            return false;
        }
        let ids: Vec<String> = (0..mods.len()).map(|_| String::new()).collect();
        let _ = ids;
        let mut prepared = mods;
        for entry in &mut prepared {
            if entry.id.is_empty() {
                entry.id = self.allocate_id("mod");
            }
        }
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        instance.mods.extend(prepared);
        self.revision += 1;
        true
    }

    pub fn add_resource_packs(
        &mut self,
        reference: &InstanceRef,
        packs: Vec<ResourcePackSummary>,
    ) -> bool {
        if packs.is_empty() {
            return false;
        }
        let mut prepared = packs;
        for entry in &mut prepared {
            if entry.id.is_empty() {
                entry.id = self.allocate_id("pack");
            }
        }
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        instance.resource_packs.extend(prepared);
        self.revision += 1;
        true
    }

    pub fn add_shader_packs(
        &mut self,
        reference: &InstanceRef,
        packs: Vec<ShaderPackSummary>,
    ) -> bool {
        if packs.is_empty() {
            return false;
        }
        let mut prepared = packs;
        for entry in &mut prepared {
            if entry.id.is_empty() {
                entry.id = self.allocate_id("shader");
            }
        }
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        instance.shader_packs.extend(prepared);
        self.revision += 1;
        true
    }

    pub fn set_java_runtime(
        &mut self,
        reference: &InstanceRef,
        runtime_id: &str,
        version: u32,
    ) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        if instance.java_runtime_id == runtime_id && instance.java == version.to_string() {
            return false;
        }
        instance.java_runtime_id = runtime_id.to_string();
        instance.java = version.to_string();
        self.revision += 1;
        true
    }

    pub fn set_launch_override(
        &mut self,
        reference: &InstanceRef,
        field: super::LaunchField,
        value: super::LaunchValue,
    ) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        if !instance.launch_overrides.set(field, value) {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn clear_launch_override(
        &mut self,
        reference: &InstanceRef,
        field: super::LaunchField,
    ) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        if !instance.launch_overrides.clear(field) {
            return false;
        }
        self.revision += 1;
        true
    }

    /// Records a successful launch: the prototype bumps play count and stamps
    /// "just now" onto the instance.
    pub fn record_launch(&mut self, reference: &InstanceRef) -> bool {
        let Some(instance) = self.instance_mut(reference) else {
            return false;
        };
        instance.last_played = Some("just now".into());
        instance.play_count += 1;
        self.revision += 1;
        true
    }

    pub fn create(&mut self, storage: StorageIdent, input: NewInstance) -> Option<InstanceRef> {
        if self.current_storage != Some(storage.clone()) {
            return None;
        }
        let id = self.allocate_id("inst");
        let is_modern = input.mc_version.starts_with("1.21");
        let mut overrides = LaunchOverrides::default();
        overrides.set(
            super::LaunchField::Memory,
            super::LaunchValue::Number(input.memory),
        );
        let icon_seed = icon_seed(&input.name, &input.mc_version, input.loader);
        let instance = InstanceSummary {
            storage: storage.clone(),
            id: id.clone(),
            icon_seed,
            name: input.name,
            aphanite: false,
            favorite: false,
            description: if input.description.is_empty() {
                "A new Phanerite instance.".into()
            } else {
                input.description
            },
            loader: input.loader,
            mc_version: input.mc_version,
            loader_version: input.loader_version,
            java: if is_modern { "21".into() } else { "17".into() },
            java_runtime_id: if is_modern {
                "zulu-21".into()
            } else {
                "temurin-17".into()
            },
            created_at: "just now".into(),
            last_played: None,
            play_count: 0,
            last_crash_id: None,
            launch_overrides: overrides,
            mods: Vec::new(),
            resource_packs: Vec::new(),
            shader_packs: Vec::new(),
            worlds: Vec::new(),
        };
        self.instances.push(instance);
        self.revision += 1;
        Some(InstanceRef::new(storage, id))
    }

    pub fn duplicate(&mut self, reference: &InstanceRef) -> Option<InstanceRef> {
        if self.current_storage != Some(reference.storage.clone()) {
            return None;
        }
        let source = self.find(reference)?.clone();
        let id = self.allocate_id("inst");
        let copy = InstanceSummary {
            id: id.clone(),
            name: format!("{} (copy)", source.name),
            play_count: 0,
            last_played: None,
            last_crash_id: None,
            ..source
        };
        self.instances.push(copy);
        self.revision += 1;
        Some(InstanceRef::new(reference.storage.clone(), id))
    }

    pub fn remove(&mut self, reference: &InstanceRef) -> bool {
        if self.current_storage != Some(reference.storage.clone()) {
            return false;
        }
        let before = self.instances.len();
        self.instances.retain(|instance| {
            instance.storage != reference.storage || instance.id != reference.instance_id
        });
        if self.instances.len() == before {
            return false;
        }
        self.revision += 1;
        true
    }

    pub fn apply_for_storage(
        &mut self,
        storages: &MultiStorage,
        storage: StorageIdent,
        result: Vec<InstanceSummary>,
    ) -> bool {
        if storages.contains(&storage)
            && self.current_storage == Some(storage.clone())
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

/// Fields the create-instance dialog collects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewInstance {
    pub name: String,
    pub description: String,
    pub mc_version: String,
    pub loader: Loader,
    pub loader_version: String,
    pub memory: u32,
}

/// The seed string the prototype builds for an instance's generated artwork.
pub fn icon_seed(name: &str, mc_version: &str, loader: Loader) -> String {
    format!("{name}|{mc_version}|{}", loader.key())
}
