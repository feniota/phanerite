use crate::storage::Storage;
use self_cell::self_cell;
use std::ops::Deref;
use uuid::Uuid;

/// 多个 Storage 的并发存储。
///
/// `TreeIndex` 负责管理 Storage 的生命周期和并发访问；
/// 通过 `scc::Guard` 获取的引用由 EBR 机制保护，在 Guard 存活期间
/// 对应 entry 的内存不会被回收。
#[derive(Default)]
pub struct MultiStorage {
    storages: scc::TreeIndex<Uuid, Storage>,
}

struct Guard<'a> {
    /// 保证 TreeIndex 在 StorageGuard 存活期间有效。
    container: &'a scc::TreeIndex<Uuid, Storage>,
    /// 保护 TreeIndex entry 的内存。
    guard: scc::Guard,
}

type RefStorage<'a> = &'a Storage;
self_cell!(
    /// 受 Guard 保护的 Storage 引用。
    pub struct StorageGuard<'a> {
        owner: Guard<'a>,
        #[covariant]
        dependent: RefStorage,
    }
);

impl Deref for StorageGuard<'_> {
    type Target = Storage;
    fn deref(&self) -> &Self::Target {
        self.borrow_dependent()
    }
}

impl MultiStorage {
    pub fn new() -> Self {
        Default::default()
    }

    /// 添加一个 Storage。
    ///
    /// 如果已经存在内容相等的 Storage，则返回传入的 Storage 所有权。
    #[allow(clippy::result_large_err)]
    pub async fn insert(&self, storage: Storage) -> Result<(), Storage> {
        let guard = scc::Guard::new();

        if self.storages.iter(&guard).any(|(_, s)| *s == storage) {
            return Err(storage);
        }

        // 使用 UUID v7 作为 TreeIndex 的 key。理论上 UUID 冲突不应该发生，
        // 因此插入失败时直接 panic。
        self.storages
            .insert_async(Uuid::now_v7(), storage)
            .await
            .expect("Unexpected UUID conflict");

        Ok(())
    }

    /// 获取指定 UUID 对应的 Storage。
    ///
    /// 返回的 [`StorageGuard`] 同时持有 `TreeIndex` 的引用和
    /// `scc::Guard`，因此其生命周期不能超过 `MultiStorage` 内部的
    /// `TreeIndex`。
    ///
    /// # EBR
    ///
    /// `peek` 返回的 Storage 引用受 `guard` 保护。
    ///
    /// 如果该 entry 在之后被逻辑删除，EBR 不会在此 Guard 存活期间
    /// 回收其内存，因此已经取得的 `StorageGuard` 仍然可以访问原来的
    /// Storage。
    ///
    /// 注意，这并不意味着 UUID 之后仍然对应这个 Storage：
    ///
    /// ```text
    /// TreeIndex:
    ///     UUID -> Storage A
    ///
    /// StorageGuard -> Storage A
    ///
    /// upsert(UUID, Storage B)
    ///
    /// TreeIndex:
    ///     UUID -> Storage B
    ///
    /// StorageGuard:
    ///     -> Storage A
    /// ```
    pub fn get(&self, id: &Uuid) -> Option<StorageGuard<'_>> {
        StorageGuard::try_new(
            Guard {
                container: &self.storages,
                guard: scc::Guard::new(),
            },
            |Guard { container, guard }| {
                container.peek(id, guard).ok_or(Err::<StorageGuard, ()>(()))
            },
        )
        .ok()
    }

    /// 在回调函数中使用迭代器
    pub fn iter<F, R>(&self, f: F) -> R
    where
        F: FnOnce(scc::tree_index::Iter<Uuid, Storage>) -> R,
    {
        let guard = scc::Guard::new();
        let iter = self.storages.iter(&guard);
        f(iter)
    }

    /// 在异步回调函数中使用迭代器
    pub async fn iter_async<F, R>(&self, f: F) -> R
    where
        F: AsyncFnOnce(scc::tree_index::Iter<Uuid, Storage>) -> R,
    {
        let guard = scc::Guard::new();
        let iter = self.storages.iter(&guard);
        f(iter).await
    }

    /// 对所有 Storage 依次执行回调。
    pub fn for_each<F: FnMut(&Uuid, &Storage)>(&self, mut f: F) {
        self.iter(|iter| iter.for_each(|(k, v)| f(k, v)))
    }

    /// 异步对所有 Storage 依次执行回调。
    pub async fn for_each_async<F: AsyncFnMut(&Uuid, &Storage)>(&self, mut f: F) {
        self.iter_async(async |iter| {
            for (k, v) in iter {
                f(k, v).await;
            }
        })
        .await
    }

    /// 删除指定 UUID 对应的 Storage。
    pub async fn remove(&self, id: &Uuid) -> bool {
        self.storages.remove_async(id).await
    }

    /// 判断指定 UUID 是否存在。
    /// 结果仅代表检查发生时的状态，不能用于同步并发修改。
    pub fn contains(&self, id: &Uuid) -> bool {
        self.storages.contains(id)
    }

    /// 获取当前 Storage 数量。
    /// 结果仅代表检查发生时的状态，不能用于同步并发修改。
    pub fn len(&self) -> usize {
        self.storages.len()
    }

    /// 判断当前是否为空。
    /// 结果仅代表检查发生时的状态，不能用于同步并发修改。
    pub fn is_empty(&self) -> bool {
        self.storages.is_empty()
    }
}
