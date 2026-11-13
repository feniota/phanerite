use std::any::TypeId;

use slotmap::{DefaultKey, Key as _, SecondaryMap};
use taffy::{Style, TaffyTree};

use crate::{
    aspect::{Aspect, AspectMap, Storage, insert_aspect},
    commands::Commands,
    node::NodeMetadata,
    resources::Resources,
};

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
    queried: Vec<TypeId>,
}

impl<'w> Context<'w> {
    pub(crate) fn new(
        taffy: &'w mut TaffyTree<()>,
        metadatas: &'w mut SecondaryMap<DefaultKey, NodeMetadata>,
        aspects: &'w mut AspectMap,
        resources: &'w mut Resources,
        commands: &'w mut Commands,
        queried: Vec<TypeId>,
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

    pub(crate) fn set_node(&mut self, key: DefaultKey) {
        self.node = key;
    }

    /// The entity currently being processed by the system.
    pub fn node(&self) -> DefaultKey {
        self.node
    }

    pub fn taffy(&self) -> &TaffyTree<()> {
        self.taffy
    }

    pub fn taffy_mut(&mut self) -> &mut TaffyTree<()> {
        self.taffy
    }

    pub fn metadata(&self, key: DefaultKey) -> Option<&NodeMetadata> {
        self.metadatas.get(key)
    }

    pub fn metadata_mut(&mut self, key: DefaultKey) -> Option<&mut NodeMetadata> {
        self.metadatas.get_mut(key)
    }

    pub fn metadatas(&self) -> &SecondaryMap<DefaultKey, NodeMetadata> {
        self.metadatas
    }

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

    pub fn get<A: Aspect>(&self, key: DefaultKey) -> Option<&A> {
        self.shared().get(key)
    }

    pub fn get_mut<A: Aspect>(&mut self, key: DefaultKey) -> Option<&mut A> {
        self.storage_mut::<A>()?.get_mut(key)
    }

    pub fn insert<A: Aspect>(&mut self, key: DefaultKey, aspect: A) -> Option<A> {
        ensure_not_queried::<A>(&self.queried);
        insert_aspect(self.aspects, key, aspect)
    }

    pub fn remove<A: Aspect>(&mut self, key: DefaultKey) -> Option<A> {
        self.storage_mut::<A>()?.remove(key)
    }

    /// Panics if the resource has not been inserted.
    pub fn resource<R: 'static>(&self) -> &R {
        self.shared().resource()
    }

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

    pub fn get<A: Aspect>(&self, key: DefaultKey) -> Option<&'a A> {
        self.storage::<A>()?.get(key)
    }

    pub fn resource<R: 'static>(&self) -> &'a R {
        self.resources.get::<R>().unwrap_or_else(|| {
            panic!(
                "Resource `{}` has not been inserted",
                std::any::type_name::<R>()
            )
        })
    }

    pub fn metadata(&self, key: DefaultKey) -> Option<&'a NodeMetadata> {
        self.metadatas.get(key)
    }
}
