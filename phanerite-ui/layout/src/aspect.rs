use slotmap::{DefaultKey, SecondaryMap, SparseSecondaryMap};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Component (as in ECS). Implement it with `#[derive(Aspect)]`.
pub trait Aspect: 'static {}

/// Map from an aspect's [`TypeId`] to its type-erased storage.
pub(crate) type AspectMap = HashMap<TypeId, Box<dyn AnyStorage>>;

/// Insert a component, registering a sparse storage on first use. Single map lookup.
pub(crate) fn insert_aspect<A: Aspect>(
    aspects: &mut AspectMap,
    key: DefaultKey,
    aspect: A,
) -> Option<A> {
    let storage = aspects.entry(TypeId::of::<A>()).or_insert_with(|| {
        tracing::info!(
            "insert_aspect called for type {}, whose storage does not exist; creating a new sparse storage",
            std::any::type_name::<A>()
        );
        Box::new(Storage::<A>::Sparse(SparseSecondaryMap::new()))
    });
    storage
        .downcast_storage_mut::<A>()
        .unwrap_or_else(|| {
            panic!(
                "Storage for type `{}` does not match the actual type",
                std::any::type_name::<A>()
            )
        })
        .insert(key, aspect)
}

/// Per-aspect component storage, either dense or sparse.
pub enum Storage<T: Aspect> {
    /// Dense storage; best for aspects held by most entities.
    Dense(SecondaryMap<DefaultKey, T>),
    /// Sparse storage; best for rarely used aspects.
    Sparse(SparseSecondaryMap<DefaultKey, T>),
}

impl<T> Storage<T>
where
    T: Aspect,
{
    /// Get a shared reference to the aspect of `key`.
    pub fn get(&self, key: DefaultKey) -> Option<&T> {
        match self {
            Self::Dense(map) => map.get(key),
            Self::Sparse(map) => map.get(key),
        }
    }
    /// Get a mutable reference to the aspect of `key`.
    pub fn get_mut(&mut self, key: DefaultKey) -> Option<&mut T> {
        match self {
            Self::Dense(map) => map.get_mut(key),
            Self::Sparse(map) => map.get_mut(key),
        }
    }
    /// Insert an aspect on `key`, returning the replaced value if any.
    pub fn insert(&mut self, key: DefaultKey, value: T) -> Option<T> {
        match self {
            Self::Dense(map) => map.insert(key, value),
            Self::Sparse(map) => map.insert(key, value),
        }
    }
    /// Remove the aspect of `key`, returning it if present.
    pub fn remove(&mut self, key: DefaultKey) -> Option<T> {
        match self {
            Self::Dense(map) => map.remove(key),
            Self::Sparse(map) => map.remove(key),
        }
    }
    /// Iterate over all `(key, aspect)` pairs mutably.
    pub fn iter_mut(&mut self) -> StorageIterator<'_, T> {
        match self {
            Self::Sparse(storage) => StorageIterator::Sparse(storage.iter_mut()),
            Self::Dense(storage) => StorageIterator::Dense(storage.iter_mut()),
        }
    }

    /// Whether `key` has an aspect in this storage.
    pub fn contains_key(&self, key: DefaultKey) -> bool {
        match self {
            Self::Dense(storage) => storage.contains_key(key),
            Self::Sparse(storage) => storage.contains_key(key),
        }
    }
}

/// Type-erased storage interface, letting the world manage all aspect storages uniformly.
pub trait AnyStorage: Any {
    /// Type-erased removal, for cleaning up all aspects of a despawned entity.
    fn remove_any(&mut self, key: DefaultKey);
}
impl dyn AnyStorage {
    /// Downcast to a typed storage, returning `None` on type mismatch.
    pub(crate) fn downcast_storage_ref<T>(&self) -> Option<&Storage<T>>
    where
        T: Aspect + Any,
    {
        (self as &dyn Any).downcast_ref::<Storage<T>>()
    }

    /// Downcast to a typed storage mutably, returning `None` on type mismatch.
    pub(crate) fn downcast_storage_mut<T>(&mut self) -> Option<&mut Storage<T>>
    where
        T: Aspect + Any,
    {
        (self as &mut dyn Any).downcast_mut::<Storage<T>>()
    }
}
impl<T> AnyStorage for Storage<T>
where
    T: Aspect + Any,
{
    fn remove_any(&mut self, key: DefaultKey) {
        self.remove(key);
    }
}

/// Iterator over the `(key, aspect)` pairs of a [`Storage`].
pub enum StorageIterator<'a, T>
where
    T: Aspect,
{
    /// Iterator over a dense storage.
    Dense(slotmap::secondary::IterMut<'a, DefaultKey, T>),
    /// Iterator over a sparse storage.
    Sparse(slotmap::sparse_secondary::IterMut<'a, DefaultKey, T>),
}
impl<'a, T> Iterator for StorageIterator<'a, T>
where
    T: Aspect,
{
    type Item = (DefaultKey, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Dense(iter) => iter.next(),
            Self::Sparse(iter) => iter.next(),
        }
    }
}
