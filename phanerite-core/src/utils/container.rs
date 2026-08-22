use std::ops::Deref;
use uuid::Uuid;

// 用于储存全局资源的容器
//
// 取用时常用全量快照，不宜存放大量元素
//
// `TreeIndex` 负责管理元素的生命周期和并发访问；
// 通过 `scc::Guard` 获取的引用由 EBR 机制保护，在 Guard 存活期间
// 对应 entry 的内存不会被回收。
/// A container for storing global resources
///
/// Access normally goes through a full snapshot, so it is not meant to hold
/// large numbers of elements
///
/// `TreeIndex` manages element lifetimes and concurrent access; references
/// obtained through an `scc::Guard` are protected by EBR, so the memory of the
/// corresponding entry is not reclaimed while the guard is alive.
#[derive(Default)]
pub struct Container<T: Eq> {
    container: scc::TreeIndex<Uuid, T>,
}

// 受 Guard 保护的 Item 引用。
/// An item reference protected by a guard.
#[ouroboros::self_referencing]
pub struct Guard<'a, T: 'a> {
    // 保证 TreeIndex 在 ItemGuard 存活期间有效。
    /// Keeps the `TreeIndex` valid for as long as the `Guard` is alive.
    container: &'a scc::TreeIndex<Uuid, T>,
    // 保护 TreeIndex entry 的内存。
    /// Protects the memory of the `TreeIndex` entry.
    guard: scc::Guard,

    #[borrows(container, guard)]
    inner: &'this T,
}

// 用于产生快照的迭代器
/// Iterator used to produce a snapshot
#[ouroboros::self_referencing]
struct IterGuard<'a, T: 'a> {
    container: &'a scc::TreeIndex<Uuid, T>,
    guard: scc::Guard,

    #[borrows(container, guard)]
    #[covariant]
    inner: scc::tree_index::Iter<'this, Uuid, T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.borrow_inner()
    }
}

impl<T: Eq> Container<T> {
    pub fn new() -> Self {
        Container {
            container: scc::TreeIndex::new(),
        }
    }

    // 添加一个 Item。
    /// Adds an item.
    pub async fn insert(&self, item: T) -> Uuid {
        let id = Uuid::now_v7();
        // 使用 UUID v7 作为 TreeIndex 的 key。理论上 UUID 冲突不应该发生，
        // 因此插入失败时直接 panic。
        if self.container.insert_async(id, item).await.is_err() {
            panic!("Unexpected UUID conflict")
        }
        id
    }

    // 获取指定 UUID 对应的 Item。
    //
    // # 在任何情况下调用此方法都有可能返回 None
    // 如果需要基于当前 Item 的状态进行并发修改，调用方必须自行进行 CAS
    // 或其他形式的并发协调。
    //
    // 如果不需要通过 ID 定位或操作单个元素，建议使用 [`Self::snapshot`]。
    //
    // # EBR
    //
    // 返回的 [`Guard`] 同时持有 `TreeIndex` 的引用和
    // `scc::Guard`，因此其生命周期不能超过 `Container` 内部的
    // `TreeIndex`。
    //
    // `peek` 返回的 Item 引用受 `guard` 保护。
    //
    // 如果该 entry 在之后被逻辑删除，EBR 不会在此 Guard 存活期间
    // 回收其内存，因此已经取得的 `ItemGuard` 仍然可以访问原来的
    // Item。
    //
    // 注意，这并不意味着 UUID 之后仍然对应这个 Item：
    /// Gets the item corresponding to the given UUID.
    ///
    /// # This method may return `None` under any circumstances
    /// If a concurrent modification has to be based on the item's current
    /// state, the caller must perform its own CAS or another form of
    /// concurrency coordination.
    ///
    /// If you do not need to locate or operate on a single element by ID,
    /// prefer [`Self::snapshot`].
    ///
    /// # EBR
    ///
    /// The returned [`Guard`] holds both a reference to the `TreeIndex` and an
    /// `scc::Guard`, so its lifetime must not outlive the `TreeIndex` inside
    /// the `Container`.
    ///
    /// The item reference returned by `peek` is protected by `guard`.
    ///
    /// If that entry is logically removed later, EBR will not reclaim its
    /// memory while this guard is alive, so an already-acquired `ItemGuard`
    /// can still access the original item.
    ///
    /// Note that this does not mean the UUID still maps to that item:
    ///
    /// ```text
    /// TreeIndex:
    ///     UUID -> Item A
    ///
    /// Guard -> Item A
    ///
    /// upsert(UUID, Item B)
    ///
    /// TreeIndex:
    ///     UUID -> Item B
    ///
    /// Guard:
    ///     -> Item A
    /// ```
    pub fn try_get(&self, id: &Uuid) -> Option<Guard<'_, T>> {
        Guard::try_new(&self.container, scc::Guard::new(), |container, guard| {
            container.peek(id, guard).ok_or(Err::<Guard<T>, ()>(()))
        })
        .ok()
    }

    // 在回调函数中使用迭代器
    /// Uses the iterator inside a callback
    pub fn iter<F, R>(&self, f: F) -> R
    where
        F: FnOnce(scc::tree_index::Iter<Uuid, T>) -> R,
    {
        let guard = scc::Guard::new();
        let iter = self.container.iter(&guard);
        f(iter)
    }

    // 在异步回调函数中使用迭代器
    /// Uses the iterator inside an async callback
    pub async fn iter_async<F, R>(&self, f: F) -> R
    where
        F: AsyncFnOnce(scc::tree_index::Iter<Uuid, T>) -> R,
    {
        let guard = scc::Guard::new();
        let iter = self.container.iter(&guard);
        f(iter).await
    }

    // 获得当前元素的弱一致快照。
    //
    // 快照过程中可能发生并发修改，因此返回的元素不保证对应于某个
    // 确定时刻的 Container 状态。
    //
    // 在 Container 较小的情况下，推荐使用此 API
    /// Takes a weakly consistent snapshot of the current elements.
    ///
    /// Concurrent modifications may happen while the snapshot is taken, so the
    /// returned elements are not guaranteed to correspond to the container's
    /// state at any single point in time.
    ///
    /// This is the recommended API when the container is small
    pub fn snapshot(&self) -> Vec<Guard<'_, T>> {
        let mut guard = IterGuard::new(&self.container, scc::Guard::new(), |container, guard| {
            container.iter(guard)
        });
        guard.with_inner_mut(|x| x.flat_map(|(id, _)| self.try_get(id)).collect::<Vec<_>>())
    }

    // 对所有 Item 依次执行回调。
    /// Runs the callback over every item in turn.
    pub fn for_each<F: FnMut(&Uuid, &T)>(&self, mut f: F) {
        self.iter(|iter| iter.for_each(|(k, v)| f(k, v)))
    }

    // 异步对所有 Item 依次执行回调。
    /// Runs the async callback over every item in turn.
    pub async fn for_each_async<F: AsyncFnMut(&Uuid, &T)>(&self, mut f: F) {
        self.iter_async(async |iter| {
            for (k, v) in iter {
                f(k, v).await;
            }
        })
        .await
    }

    // 删除指定 UUID 对应的 Item。
    /// Removes the item corresponding to the given UUID.
    pub async fn remove(&self, id: &Uuid) -> bool {
        self.container.remove_async(id).await
    }

    // 保留满足条件的 Item，删除其余 Item。
    //
    // 谓词接收到的是遍历时观察到的 Item。由于操作不具备事务一致性，
    // 并发修改可能导致最终结果与谓词判断时的状态不同。
    //
    // 如果需要基于 Item 当前状态安全地决定是否删除，调用方必须自行进行
    // CAS 或其他形式的并发协调。
    /// Keeps the items that satisfy the predicate and removes the rest.
    ///
    /// The predicate receives the item as observed during traversal. The
    /// operation is not transactionally consistent, so concurrent
    /// modifications may leave the final result different from the state the
    /// predicate judged.
    ///
    /// If removal has to be decided safely from an item's current state, the
    /// caller must perform its own CAS or another form of concurrency
    /// coordination.
    pub async fn retain<F: AsyncFnMut(&Uuid, &T) -> bool>(&self, mut f: F) {
        self.for_each_async(async |k, v| {
            if !f(k, v).await {
                self.remove(k).await;
            }
        })
        .await
    }

    // 判断指定 UUID 是否存在。
    // **结果仅代表检查发生时的状态，不能用于同步并发修改。**
    /// Returns whether the given UUID exists.
    /// **The result only reflects the state at the moment of the check and
    /// cannot be used to synchronize concurrent modifications.**
    pub fn contains(&self, id: &Uuid) -> bool {
        self.container.contains(id)
    }

    // 获取当前 Item 数量。
    // **结果仅代表检查发生时的状态，不能用于同步并发修改。**
    /// Returns the current number of items.
    /// **The result only reflects the state at the moment of the check and
    /// cannot be used to synchronize concurrent modifications.**
    pub fn len(&self) -> usize {
        self.container.len()
    }

    // 判断当前是否为空。
    // **结果仅代表检查发生时的状态，不能用于同步并发修改。**
    /// Returns whether the container is currently empty.
    /// **The result only reflects the state at the moment of the check and
    /// cannot be used to synchronize concurrent modifications.**
    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }
}
