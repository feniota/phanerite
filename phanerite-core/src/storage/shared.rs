//! Registries for shared files in a [`Storage`](super::Storage).
//!
//! A registry maps a content [`enum@Hash`] to the path of the corresponding file
//! in the storage's shared bucket. It is used by the downloader to avoid
//! downloading a file that is already available locally.
//!
//! Unlike the older [`crate::download::dedup::StorageRegistry`] interface,
//! this registry also models references. A successful
//! [`Registry::query_and_increase`] call keeps the returned path borrowed from
//! the registry and increases its reference count. This allows implementations
//! backed by concurrent maps, such as `scc::HashMap`, to return a guard instead
//! of cloning the stored [`PathBuf`].
//!
//! # Important implementation notes
//!
//! - `query` does not change the reference count.
//! - `query_and_increase` must make the increment and the returned guard part
//!   of one logically consistent operation.
//! - A returned [`Path`] may keep a map bucket locked or otherwise retain a
//!   resource. Callers should drop it as soon as they no longer need it.
//! - Implementations must not return a borrowed path whose backing storage can
//!   be destroyed before the returned guard is dropped.
//! - Reference decrementing is part of the trait contract, but automatic
//!   deletion of files is deliberately left to the storage lifecycle code.

use crate::storage::StorageIdent;
use crate::utils::Hash;
use anyhow::anyhow;
use async_trait::async_trait;
use scc::hash_map::{Entry, OccupiedEntry};
use std::{collections::hash_map::RandomState, ops::Deref, path::PathBuf, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested registry entry does not exist.
    #[error("The specified entry is not found in the storage")]
    EntryNotFound,
    /// An implementation-specific operation failed.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// The result type used by shared-file registries.
pub type Result<T> = core::result::Result<T, Error>;

/// A path returned by a shared-file registry.
///
/// `Owned` is useful for registries that naturally return an owned path.
/// `Guard` is intended for concurrent containers whose lookup operation
/// returns a guard borrowing the container. In that case, dereferencing this
/// value does not clone the underlying [`PathBuf`].
///
/// The lifetime is tied to the registry borrow for guarded paths. Keep the
/// value alive for as long as the path is being used, and do not convert a
/// guard-backed path into an independent path unless an owned copy is
/// explicitly required.
pub enum Path<'a> {
    /// An independently owned path.
    Owned(PathBuf),
    /// A guard that dereferences to a path stored by the registry.
    Guard(Box<dyn Deref<Target = PathBuf> + 'a>),
}

impl Path<'static> {
    /// Wraps an owned path.
    ///
    /// This constructor is only available for an owned path because an owned
    /// value does not borrow from a registry.
    pub fn new_owned(path: PathBuf) -> Self {
        Self::Owned(path)
    }
}

impl<'a> Path<'a> {
    /// Wraps a guard or smart pointer that dereferences to a [`PathBuf`].
    ///
    /// The returned value retains `guard` until it is dropped. For an `scc`
    /// lookup guard, this also retains the borrow needed to safely access the
    /// map entry.
    pub fn new_guard(guard: impl Deref<Target = PathBuf> + 'a) -> Self {
        Self::Guard(Box::new(guard))
    }
}

impl Deref for Path<'_> {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(path) => path,
            Self::Guard(guard) => guard.deref(),
        }
    }
}

/// A registry of files stored in a shared bucket.
///
/// The trait is object-safe through `async_trait`, so a [`Storage`](super::Storage)
/// can own a boxed registry without knowing its concrete implementation.
///
/// Implementations must be safe to use concurrently because the registry is
/// shared by concurrent download tasks. The `Send + Sync` bounds are therefore
/// part of the contract.
#[async_trait]
pub trait Registry: Send + Sync {
    /// Looks up a path without changing its reference count.
    ///
    /// The returned path may be backed by a guard. In particular, callers
    /// should not assume that this operation clones the stored path.
    async fn query<'a>(&'a self, key: &Hash) -> Option<Path<'a>>;

    /// Looks up a path and increases its reference count.
    ///
    /// If that file is found in the storage, its reference count
    /// should increase by 1.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(path))` if this entry is found and its reference count
    ///   is successfully increased.
    /// - `Ok(None)` if this entry is not found.
    /// - `Err` if the query failed.
    async fn query_and_increase<'a>(&'a self, key: &Hash) -> Result<Option<Path<'a>>>;

    /// Registers a path with an initial reference count of one.
    ///
    /// Implementations should define how duplicate keys are handled. A
    /// downloader may encounter a duplicate when concurrent tasks finish at
    /// nearly the same time.
    async fn insert(&self, key: Hash, val: PathBuf) -> Result<()>;

    /// Decreases an item's reference count by one.
    ///
    /// # Returns
    ///
    /// - `Ok(new_count)` if the operation succeeds. The entry may be removed
    ///   when the new count reaches zero.
    /// - `Err(Error::EntryNotFound)` if the entry does not exist.
    /// - another error if the implementation cannot update the count.
    async fn decrease(&self, key: &Hash) -> Result<u32>;
}

/// A no-op registry for [`SharePreference::Move`](super::SharePreference).
///
/// Move mode does not retain a shared-bucket file after placing the file at
/// its destination, so there is nothing useful to index. Lookups therefore
/// always miss. Attempts to mutate this registry return an error, which helps
/// expose an accidental attempt to use shared-file accounting in Move mode.
#[async_trait]
impl Registry for () {
    async fn query<'a>(&'a self, _: &Hash) -> Option<Path<'a>> {
        None
    }

    async fn query_and_increase<'a>(&'a self, _: &Hash) -> Result<Option<Path<'a>>> {
        Ok(None)
    }

    async fn insert(&self, _: Hash, _: PathBuf) -> Result<()> {
        Err(anyhow!("Trying to insert storage item into dummy registry").into())
    }

    async fn decrease(&self, _: &Hash) -> Result<u32> {
        Err(anyhow!("Trying to query reference count on dummy registry").into())
    }
}

/// A guard adapting an `scc::HashMap` entry containing `(PathBuf, u32)` into a
/// guard that dereferences directly to the stored path.
struct SccPathGuard<'a> {
    entry: OccupiedEntry<'a, Hash, (PathBuf, u32), RandomState>,
}

impl Deref for SccPathGuard<'_> {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.entry.get().0
    }
}

/// In-memory registry backed by `scc::HashMap`.
///
/// The key is the content hash, the first tuple field is the shared-bucket
/// path, and the second tuple field is the logical reference count. Lookup
/// guards borrow the map entry, so querying a path does not clone its
/// [`PathBuf`].
///
/// This registry is process-local. It does not reconstruct entries from an
/// existing shared bucket after a restart, and its reference counts are not
/// persisted.
#[async_trait]
impl Registry for scc::HashMap<Hash, (PathBuf, u32)> {
    async fn query<'a>(&'a self, key: &Hash) -> Option<Path<'a>> {
        self.get_async(key)
            .await
            .map(|entry| Path::new_guard(SccPathGuard { entry }))
    }

    async fn query_and_increase<'a>(&'a self, key: &Hash) -> Result<Option<Path<'a>>> {
        match self.entry_async(key.clone()).await {
            Entry::Vacant(_) => Ok(None),
            Entry::Occupied(mut entry) => {
                entry.get_mut().1 += 1;
                Ok(Some(Path::new_guard(SccPathGuard { entry })))
            }
        }
    }

    async fn insert(&self, key: Hash, val: PathBuf) -> Result<()> {
        self.insert_async(key, (val, 1))
            .await
            .map_err(|(_, _)| anyhow!("Storage registry entry already exists").into())
    }

    async fn decrease(&self, key: &Hash) -> Result<u32> {
        match self.entry_async(key.clone()).await {
            Entry::Vacant(_) => Err(Error::EntryNotFound),
            Entry::Occupied(mut entry) => {
                if entry.get().1 == 0 {
                    return Err(anyhow!("Storage registry reference count underflow").into());
                }
                let count = entry.get().1 - 1;
                entry.get_mut().1 = count;
                if count == 0 {
                    let _ = entry.remove();
                }
                Ok(count)
            }
        }
    }
}
/// Adapts a storage-aware [`MultiRegistry`] into a [`Registry`].
///
/// The adapter stores one [`StorageIdent`] and automatically includes it in
/// every operation. This is useful when one registry serves several storage
/// roots and the public caller should only provide a content hash.
///
/// The inner registry is held in an [`Arc`] so the adapter can be cheaply
/// cloned or shared by components that use the same multi-storage index.
pub struct MultiRegistryAdapter {
    storage: StorageIdent,
    inner: Arc<dyn MultiRegistry + Send + Sync>,
}

/// A registry whose keys include the storage identity.
///
/// This variant prevents equal content hashes belonging to different storage
/// roots from being treated as the same file. Implementations must ensure
/// that a returned path belongs to the [`StorageIdent`] supplied in the key;
/// returning a path from another root can make linking fail across filesystems
/// or silently connect unrelated storage trees.
#[async_trait]
impl Registry for MultiRegistryAdapter {
    /// Looks up a path for a storage and content-hash pair.
    async fn query<'a>(&'a self, key: &Hash) -> Option<Path<'a>> {
        self.inner.query((&self.storage, key)).await
    }

    /// Looks up a path and increases its reference count.
    async fn query_and_increase<'a>(&'a self, key: &Hash) -> Result<Option<Path<'a>>> {
        self.inner.query_and_increase((&self.storage, key)).await
    }

    async fn insert(&self, key: Hash, val: PathBuf) -> Result<()> {
        self.inner.insert((&self.storage, key), val).await
    }

    async fn decrease(&self, key: &Hash) -> Result<u32> {
        self.inner.decrease((&self.storage, key)).await
    }
}

/// Storage-aware version of [`Registry`].
///
/// This can be implemented if you want multiple storages' items tracked in the same databsse.
/// See [`MultiRegistryAdapter`] for how to put this in a [`super::Storage`].
#[async_trait]
pub trait MultiRegistry {
    async fn query<'a>(&'a self, key: (&StorageIdent, &Hash)) -> Option<Path<'a>>;

    /// Looks up the path of a file identified by its storage and content hash.
    async fn query_and_increase<'a>(
        &'a self,
        key: (&StorageIdent, &Hash),
    ) -> Result<Option<Path<'a>>>;

    /// Registers a path with an initial reference count of one.
    async fn insert(&self, key: (&StorageIdent, Hash), val: PathBuf) -> Result<()>;

    /// Decreases an item's reference count by one.
    ///
    /// The entry should get deleted right in this function if the new reference
    /// count reaches 0.
    async fn decrease(&self, key: (&StorageIdent, &Hash)) -> Result<u32>;
}
