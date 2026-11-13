use slotmap::{DefaultKey, SecondaryMap, SparseSecondaryMap};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Component (as in ECS). Implement it with `#[derive(Aspect)]`.
pub trait Aspect: 'static {}

pub(crate) type AspectMap = HashMap<TypeId, Box<dyn AnyStorage>>;

/// Insert a component, registering a sparse storage on first use. Single map lookup.
pub(crate) fn insert_aspect<A: Aspect>(
    aspects: &mut AspectMap,
    key: DefaultKey,
    aspect: A,
) -> Option<A> {
    let storage = aspects.entry(TypeId::of::<A>()).or_insert_with(|| {
        tracing::info!(
            "Called insert for type {} whose storage does not exist, creating a new sparse one...",
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

pub enum Storage<T: Aspect> {
    Dense(SecondaryMap<DefaultKey, T>),
    Sparse(SparseSecondaryMap<DefaultKey, T>),
}

impl<T> Storage<T>
where
    T: Aspect,
{
    pub fn get(&self, key: DefaultKey) -> Option<&T> {
        match self {
            Self::Dense(map) => map.get(key),
            Self::Sparse(map) => map.get(key),
        }
    }
    pub fn get_mut(&mut self, key: DefaultKey) -> Option<&mut T> {
        match self {
            Self::Dense(map) => map.get_mut(key),
            Self::Sparse(map) => map.get_mut(key),
        }
    }
    pub fn insert(&mut self, key: DefaultKey, value: T) -> Option<T> {
        match self {
            Self::Dense(map) => map.insert(key, value),
            Self::Sparse(map) => map.insert(key, value),
        }
    }
    pub fn remove(&mut self, key: DefaultKey) -> Option<T> {
        match self {
            Self::Dense(map) => map.remove(key),
            Self::Sparse(map) => map.remove(key),
        }
    }
    pub fn iter_mut(&mut self) -> StorageIterator<'_, T> {
        match self {
            Self::Sparse(storage) => StorageIterator::Sparse(storage.iter_mut()),
            Self::Dense(storage) => StorageIterator::Dense(storage.iter_mut()),
        }
    }

    pub fn contains_key(&self, key: DefaultKey) -> bool {
        match self {
            Self::Dense(storage) => storage.contains_key(key),
            Self::Sparse(storage) => storage.contains_key(key),
        }
    }
}

pub trait AnyStorage: Any {
    /// Type-erased removal, for cleaning up all aspects of a despawned entity.
    fn remove_any(&mut self, key: DefaultKey);
}
impl dyn AnyStorage {
    pub(crate) fn downcast_storage_ref<T>(&self) -> Option<&Storage<T>>
    where
        T: Aspect + Any,
    {
        (self as &dyn Any).downcast_ref::<Storage<T>>()
    }

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

pub enum StorageIterator<'a, T>
where
    T: Aspect,
{
    Dense(slotmap::secondary::IterMut<'a, DefaultKey, T>),
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
