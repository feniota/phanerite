use std::any::TypeId;

use slotmap::{DefaultKey, Key as _, SecondaryMap};
use taffy::{Style, TaffyTree};

use crate::{
    aspect::{Aspect, AspectMap, Storage, insert_aspect},
    commands::Commands,
    node::NodeMetadata,
    resources::Resources,
};

/// Panic if `A` is part of the system's query, since Context access would alias the query's
/// mutable references.
fn ensure_not_queried<A: Aspect>(queried: &[TypeId]) {
    if queried.contains(&TypeId::of::<A>()) {
        panic!(
            "phanerite_layout: aspect `{}` is part of the current system's query; accessing it through Context would alias the query's mutable references",
            std::any::type_name::<A>()
        );
    }
}

/// Additional context passed to a system as its first parameter, with which the system can
/// access the layout details (since taffy's data and NodeMetadata are not [`crate::Aspect`]s)
/// and do other things on the World.
///
/// Access to any aspect that is part of the system's own query panics: the query already
/// hands out mutable references to those storages, and a second access path would alias them.
pub struct Context<'w> {
    node: DefaultKey,
    taffy: &'w mut TaffyTree<()>,
    metadatas: &'w mut SecondaryMap<DefaultKey, NodeMetadata>,
    aspects: &'w mut AspectMap,
    resources: &'w mut Resources,
    commands: &'w mut Commands,
    queried: &'static [TypeId],
}

impl<'w> Context<'w> {
    /// Assemble a context over the world's data, recording the aspects queried by the system.
    pub(crate) fn new(
        taffy: &'w mut TaffyTree<()>,
        metadatas: &'w mut SecondaryMap<DefaultKey, NodeMetadata>,
        aspects: &'w mut AspectMap,
        resources: &'w mut Resources,
        commands: &'w mut Commands,
        queried: &'static [TypeId],
    ) -> Self {
        Self {
            node: DefaultKey::null(),
            taffy,
            metadatas,
            aspects,
            resources,
            commands,
            queried,
        }
    }

    /// Point the context at the entity currently being processed.
    pub(crate) fn set_node(&mut self, key: DefaultKey) {
        self.node = key;
    }

    /// The entity currently being processed by the system.
    pub fn node(&self) -> DefaultKey {
        self.node
    }

    /// The underlying taffy layout tree.
    pub fn taffy(&self) -> &TaffyTree<()> {
        self.taffy
    }

    /// The underlying taffy layout tree, mutably.
    pub fn taffy_mut(&mut self) -> &mut TaffyTree<()> {
        self.taffy
    }

    /// The metadata of a node, if it exists.
    pub fn metadata(&self, key: DefaultKey) -> Option<&NodeMetadata> {
        self.metadatas.get(key)
    }

    /// The metadata of a node, mutably, if it exists.
    pub fn metadata_mut(&mut self, key: DefaultKey) -> Option<&mut NodeMetadata> {
        self.metadatas.get_mut(key)
    }

    /// All node metadata.
    pub fn metadatas(&self) -> &SecondaryMap<DefaultKey, NodeMetadata> {
        self.metadatas
    }

    /// All node metadata, mutably.
    pub fn metadatas_mut(&mut self) -> &mut SecondaryMap<DefaultKey, NodeMetadata> {
        self.metadatas
    }

    /// Resolve a typed storage handle once, then access it without any further type erasure.
    /// This is the hot-path exit; the conflict check happens here, once.
    ///
    /// Returns `None` if the aspect is not registered. Panics if it is part of the current query.
    pub fn storage<A: Aspect>(&self) -> Option<&Storage<A>> {
        self.shared().storage()
    }

    /// Mutable access to a typed storage. Returns `None` if the aspect is not registered.
    /// Panics if it is part of the current query.
    pub fn storage_mut<A: Aspect>(&mut self) -> Option<&mut Storage<A>> {
        ensure_not_queried::<A>(&self.queried);
        self.aspects.get_mut(&TypeId::of::<A>()).map(|storage| {
            storage.downcast_storage_mut::<A>().unwrap_or_else(|| {
                panic!(
                    "Storage for type `{}` does not match the actual type",
                    std::any::type_name::<A>()
                )
            })
        })
    }

    /// Read the aspect `A` of `key`. Panics if `A` is part of the current query.
    pub fn get<A: Aspect>(&self, key: DefaultKey) -> Option<&A> {
        self.shared().get(key)
    }

    /// Mutably access the aspect `A` of `key`. Panics if `A` is part of the current query.
    pub fn get_mut<A: Aspect>(&mut self, key: DefaultKey) -> Option<&mut A> {
        self.storage_mut::<A>()?.get_mut(key)
    }

    /// Insert an aspect on `key`, returning the replaced value if any. Registers a sparse
    /// storage on first use. Panics if `A` is part of the current query.
    pub fn insert<A: Aspect>(&mut self, key: DefaultKey, aspect: A) -> Option<A> {
        ensure_not_queried::<A>(&self.queried);
        insert_aspect(self.aspects, key, aspect)
    }

    /// Remove the aspect `A` from `key`, returning it if present.
    pub fn remove<A: Aspect>(&mut self, key: DefaultKey) -> Option<A> {
        self.storage_mut::<A>()?.remove(key)
    }

    /// Panics if the resource has not been inserted.
    pub fn resource<R: 'static>(&self) -> &R {
        self.shared().resource()
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

    /// Create a node immediately and return its key, so parent-child links can be built
    /// right away. Only touches taffy and metadata, which are disjoint from aspect storages;
    /// attach components afterwards with [`Context::insert`].
    pub fn spawn(&mut self, style: Style, metadata: NodeMetadata) -> DefaultKey {
        let key: DefaultKey = self
            .taffy
            .new_leaf(style)
            .expect("taffy failed to create a node")
            .into();
        self.metadatas.insert(key, metadata);
        key
    }

    /// Deferred structural commands, flushed after the system finishes.
    pub fn commands(&mut self) -> &mut Commands {
        self.commands
    }

    /// Split into a mutable taffy borrow and a shared view of everything else, so a measure
    /// closure can read aspects/resources while taffy runs the layout pass.
    pub fn split(&mut self) -> (&mut TaffyTree<()>, SharedContext<'_>) {
        let shared = SharedContext {
            aspects: self.aspects,
            resources: self.resources,
            metadatas: self.metadatas,
            queried: &self.queried,
        };
        (&mut *self.taffy, shared)
    }

    /// Build a shared view over the same data, for read-only access without splitting.
    fn shared(&self) -> SharedContext<'_> {
        SharedContext {
            aspects: self.aspects,
            resources: self.resources,
            metadatas: self.metadatas,
            queried: &self.queried,
        }
    }
}

/// The shared half of [`Context::split`]: read-only access to aspects, resources and metadata,
/// with the same query-conflict checks as [`Context`].
pub struct SharedContext<'a> {
    aspects: &'a AspectMap,
    resources: &'a Resources,
    metadatas: &'a SecondaryMap<DefaultKey, NodeMetadata>,
    queried: &'a [TypeId],
}

impl<'a> SharedContext<'a> {
    /// Read a typed storage. Returns `None` if the aspect is not registered.
    /// Panics if it is part of the current query.
    pub fn storage<A: Aspect>(&self) -> Option<&'a Storage<A>> {
        ensure_not_queried::<A>(self.queried);
        self.aspects.get(&TypeId::of::<A>()).map(|storage| {
            storage.downcast_storage_ref::<A>().unwrap_or_else(|| {
                panic!(
                    "Storage for type `{}` does not match the actual type",
                    std::any::type_name::<A>()
                )
            })
        })
    }

    /// Read the aspect `A` of `key`. Panics if `A` is part of the current query.
    pub fn get<A: Aspect>(&self, key: DefaultKey) -> Option<&'a A> {
        self.storage::<A>()?.get(key)
    }

    /// Read a resource. Panics if it has not been inserted.
    pub fn resource<R: 'static>(&self) -> &'a R {
        self.resources.get::<R>().unwrap_or_else(|| {
            panic!(
                "Resource `{}` has not been inserted",
                std::any::type_name::<R>()
            )
        })
    }

    /// The metadata of a node, if it exists.
    pub fn metadata(&self, key: DefaultKey) -> Option<&'a NodeMetadata> {
        self.metadatas.get(key)
    }
}
