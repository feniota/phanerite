use slotmap::{DefaultKey, SecondaryMap, SlotMap, SparseSecondaryMap};
use std::any::TypeId;
use taffy::{Style, TaffyTree};

use crate::{
    aspect::{Aspect, AspectMap, Storage},
    batch::{Batch, Bundle},
    commands::Commands,
    context::Context,
    node::NodeMetadata,
    resources::Resources,
    system::{AspectSet, ErasedSystem, System, SystemKey, SystemWrapper},
};

/// The layout and UI data of a window.
///
/// It is actually an ECS world and most of the logic runs inside it.
#[derive(Default)]
pub struct World {
    taffy_tree: TaffyTree<()>,
    metadatas: SecondaryMap<DefaultKey, NodeMetadata>,
    aspects: AspectMap,
    systems: SlotMap<SystemKey, Box<dyn ErasedSystem>>,
    resources: Resources,
    commands: Commands,

    /// Whether there are system(s) running.
    ///
    /// Storage should not be deleted or replaced once the system iteration is started. (because
    /// we pass unwrapped mutable references directly to the system) This variable controls
    /// this behavior.
    lock: bool,
}

impl World {
    /// Create an empty world.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a system, validating its aspect set. Returns a key for later removal.
    pub fn insert_system<T>(&mut self, system: impl System<T>) -> SystemKey
    where
        T: AspectSet,
    {
        T::validate();
        self.systems.insert(SystemWrapper::new_boxed(system))
    }

    /// Remove a previously registered system. Returns whether it existed.
    pub fn remove_system(&mut self, key: SystemKey) -> bool {
        // I don't think the system would be used after its removal
        self.systems.remove(key).is_some()
    }

    /// Register a dense storage for `T`. Panics if a storage for `T` already exists.
    pub fn register_dense<T>(&mut self)
    where
        T: Aspect + 'static,
    {
        let type_id = TypeId::of::<T>();
        let map = SecondaryMap::new();
        let storage: Storage<T> = Storage::Dense(map);
        if self.aspects.insert(type_id, Box::new(storage)).is_some() {
            panic!(
                "Refusing to replace the existing storage for aspect `{}`",
                std::any::type_name::<T>()
            )
        }
    }

    /// Register a sparse storage for `T`. Panics if a storage for `T` already exists.
    pub fn register<T>(&mut self)
    where
        T: Aspect + 'static,
    {
        let type_id = TypeId::of::<T>();
        let map = SparseSecondaryMap::new();
        let storage: Storage<T> = Storage::Sparse(map);
        if self.aspects.insert(type_id, Box::new(storage)).is_some() {
            panic!(
                "Refusing to replace the existing storage for aspect `{}`",
                std::any::type_name::<T>()
            )
        }
    }

    /// Read the storage of `T`. Returns `None` if `T` is not registered.
    #[must_use]
    pub fn storage<T>(&self) -> Option<&Storage<T>>
    where
        T: Aspect,
    {
        Some(
            self.aspects
                .get(&TypeId::of::<T>())?
                .as_ref()
                .downcast_storage_ref::<T>()
                .unwrap_or_else(|| {
                    panic!(
                        "Storage for type `{}` does not match the actual type",
                        std::any::type_name::<T>()
                    )
                }),
        )
    }

    /// Mutably access the storage of `T`. Returns `None` if `T` is not registered.
    #[must_use]
    pub fn storage_mut<T>(&mut self) -> Option<&mut Storage<T>>
    where
        T: Aspect,
    {
        Some(
            self.aspects
                .get_mut(&TypeId::of::<T>())?
                .as_mut()
                .downcast_storage_mut::<T>()
                .unwrap_or_else(|| {
                    panic!(
                        "Storage for type `{}` does not match the actual type",
                        std::any::type_name::<T>()
                    )
                }),
        )
    }

    /// Raw pointer to the storage of `T`, used by system and batch internals to avoid
    /// holding borrows across iterations.
    #[must_use]
    pub(crate) fn storage_ptr<T>(&mut self) -> Option<*mut Storage<T>>
    where
        T: Aspect,
    {
        Some(
            self.aspects
                .get_mut(&TypeId::of::<T>())?
                .downcast_storage_mut::<T>()
                .unwrap_or_else(|| {
                    panic!(
                        "Storage for type `{}` does not match the actual type",
                        std::any::type_name::<T>()
                    )
                }) as *mut Storage<T>,
        )
    }

    /// The underlying taffy layout tree.
    pub fn taffy_tree(&self) -> &TaffyTree<()> {
        &self.taffy_tree
    }
    /// The underlying taffy layout tree, mutably.
    pub fn taffy_tree_mut(&mut self) -> &mut TaffyTree<()> {
        &mut self.taffy_tree
    }

    /// Insert an aspect on `key`, returning the replaced value if any. Registers a sparse
    /// storage on first use.
    pub fn insert_aspect<T>(&mut self, key: impl Into<DefaultKey>, aspect: T) -> Option<T>
    where
        T: Aspect,
    {
        crate::aspect::insert_aspect(&mut self.aspects, key.into(), aspect)
    }

    /// The metadata of a node, if it exists.
    pub fn metadata(&self, key: DefaultKey) -> Option<&NodeMetadata> {
        self.metadatas.get(key)
    }

    /// The metadata of a node, mutably, if it exists.
    pub fn metadata_mut(&mut self, key: DefaultKey) -> Option<&mut NodeMetadata> {
        self.metadatas.get_mut(key)
    }

    /// Create a node and return its key. Attach components with [`World::insert_aspect`].
    pub fn spawn(&mut self, style: Style, metadata: NodeMetadata) -> DefaultKey {
        let key: DefaultKey = self
            .taffy_tree
            .new_leaf(style)
            .expect("taffy failed to create a node")
            .into();
        self.metadatas.insert(key, metadata);
        key
    }

    /// Remove an entity and all its descendants: taffy nodes, metadata, and every aspect.
    pub fn despawn(&mut self, key: DefaultKey) {
        let mut pending = vec![key];
        while let Some(key) = pending.pop() {
            if let Ok(children) = self.taffy_tree.children(key.into()) {
                pending.extend(children.into_iter().map(DefaultKey::from));
            }
            for storage in self.aspects.values_mut() {
                storage.remove_any(key);
            }
            self.metadatas.remove(key);
            let _ = self.taffy_tree.remove(key.into());
        }
    }

    /// Insert a resource, returning the previous value of the same type if any.
    pub fn insert_resource<R: 'static>(&mut self, value: R) -> Option<R> {
        self.resources.insert(value)
    }

    /// Panics if the resource has not been inserted.
    pub fn resource<R: 'static>(&self) -> &R {
        self.resources.get::<R>().unwrap_or_else(|| {
            panic!(
                "Resource `{}` has not been inserted",
                std::any::type_name::<R>()
            )
        })
    }

    /// Mutable access to a resource. Panics if it has not been inserted.
    pub fn resource_mut<R: 'static>(&mut self) -> &mut R {
        self.resources.get_mut::<R>().unwrap_or_else(|| {
            panic!(
                "Resource `{}` has not been inserted",
                std::any::type_name::<R>()
            )
        })
    }

    /// Remove a resource, returning it if present.
    pub fn remove_resource<R: 'static>(&mut self) -> Option<R> {
        self.resources.remove()
    }

    /// Resolve all storages of a bundle once for repeated insertion.
    /// Returns `None` if any member aspect is not registered.
    pub fn batch<B: Bundle>(&mut self) -> Option<Batch<'_, B>> {
        Batch::new(self)
    }

    /// Build a [`Context`] over the world for the given queried aspect set.
    pub(crate) fn context(&mut self, queried: Vec<TypeId>) -> Context<'_> {
        Context::new(
            &mut self.taffy_tree,
            &mut self.metadatas,
            &mut self.aspects,
            &mut self.resources,
            &mut self.commands,
            queried,
        )
    }

    /// Run a registered system, then flush its deferred commands.
    pub(crate) fn execute_system(&mut self, key: SystemKey) {
        // Take the system table out so the system can borrow the world mutably.
        let systems = std::mem::take(&mut self.systems);
        if let Some(system) = systems.get(key) {
            system.execute(self);
        }
        self.systems = systems;
        self.flush_commands();
    }

    /// Apply all queued commands (currently only despawns).
    pub(crate) fn flush_commands(&mut self) {
        let despawns: Vec<DefaultKey> = self.commands.drain_despawns().collect();
        for key in despawns {
            self.despawn(key);
        }
    }
}
