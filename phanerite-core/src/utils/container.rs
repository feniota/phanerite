use std::ops::Deref;
use uuid::Uuid;

/// 多个共享元素的并发存储。
///
/// 内置去重（插入时线性查找，不宜存放大量元素）
///
/// `TreeIndex` 负责管理元素的生命周期和并发访问；
/// 通过 `scc::Guard` 获取的引用由 EBR 机制保护，在 Guard 存活期间
/// 对应 entry 的内存不会被回收。
#[derive(Default)]
pub struct Container<T: Eq> {
    container: scc::TreeIndex<Uuid, T>,
}

/// 受 Guard 保护的 Item 引用。
#[ouroboros::self_referencing]
pub struct Guard<'a, T: 'a> {
    /// 保证 TreeIndex 在 ItemGuard 存活期间有效。
    container: &'a scc::TreeIndex<Uuid, T>,
    /// 保护 TreeIndex entry 的内存。
    guard: scc::Guard,

    #[borrows(container, guard)]
    inner: &'this T,
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

    /// 添加一个 Item。
    ///
    /// 如果已经存在内容相等的 Item，则返回传入的 Item 所有权。
    #[allow(clippy::result_large_err)]
    pub async fn insert(&self, item: T) -> Result<(), T> {
        let guard = scc::Guard::new();

        if self.container.iter(&guard).any(|(_, s)| *s == item) {
            return Err(item);
        }

        // 使用 UUID v7 作为 TreeIndex 的 key。理论上 UUID 冲突不应该发生，
        // 因此插入失败时直接 panic。
        if self
            .container
            .insert_async(Uuid::now_v7(), item)
            .await
            .is_err()
        {
            panic!("Unexpected UUID conflict")
        }

        Ok(())
    }

    /// 获取指定 UUID 对应的 Item。
    ///
    /// 返回的 [`Guard`] 同时持有 `TreeIndex` 的引用和
    /// `scc::Guard`，因此其生命周期不能超过 `Container` 内部的
    /// `TreeIndex`。
    ///
    /// # EBR
    ///
    /// `peek` 返回的 Item 引用受 `guard` 保护。
    ///
    /// 如果该 entry 在之后被逻辑删除，EBR 不会在此 Guard 存活期间
    /// 回收其内存，因此已经取得的 `ItemGuard` 仍然可以访问原来的
    /// Item。
    ///
    /// 注意，这并不意味着 UUID 之后仍然对应这个 Item：
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
    pub fn get(&self, id: &Uuid) -> Option<Guard<'_, T>> {
        Guard::try_new(&self.container, scc::Guard::new(), |container, guard| {
            container.peek(id, guard).ok_or(Err::<Guard<T>, ()>(()))
        })
        .ok()
    }

    /// 在回调函数中使用迭代器
    pub fn iter<F, R>(&self, f: F) -> R
    where
        F: FnOnce(scc::tree_index::Iter<Uuid, T>) -> R,
    {
        let guard = scc::Guard::new();
        let iter = self.container.iter(&guard);
        f(iter)
    }

    /// 在异步回调函数中使用迭代器
    pub async fn iter_async<F, R>(&self, f: F) -> R
    where
        F: AsyncFnOnce(scc::tree_index::Iter<Uuid, T>) -> R,
    {
        let guard = scc::Guard::new();
        let iter = self.container.iter(&guard);
        f(iter).await
    }

    /// 对所有 Item 依次执行回调。
    pub fn for_each<F: FnMut(&Uuid, &T)>(&self, mut f: F) {
        self.iter(|iter| iter.for_each(|(k, v)| f(k, v)))
    }

    /// 异步对所有 Item 依次执行回调。
    pub async fn for_each_async<F: AsyncFnMut(&Uuid, &T)>(&self, mut f: F) {
        self.iter_async(async |iter| {
            for (k, v) in iter {
                f(k, v).await;
            }
        })
        .await
    }

    /// 删除指定 UUID 对应的 Item。
    pub async fn remove(&self, id: &Uuid) -> bool {
        self.container.remove_async(id).await
    }

    /// 判断指定 UUID 是否存在。
    /// 结果仅代表检查发生时的状态，不能用于同步并发修改。
    #[deprecated(
        note = "this operation is not transactional; its result may be invalidated by concurrent modifications"
    )]
    pub fn contains(&self, id: &Uuid) -> bool {
        self.container.contains(id)
    }

    /// 获取当前 Item 数量。
    /// 结果仅代表检查发生时的状态，不能用于同步并发修改。
    #[deprecated(
        note = "this operation is not transactional; its result may be invalidated by concurrent modifications"
    )]
    pub fn len(&self) -> usize {
        self.container.len()
    }

    /// 判断当前是否为空。
    /// 结果仅代表检查发生时的状态，不能用于同步并发修改。
    #[deprecated(
        note = "this operation is not transactional; its result may be invalidated by concurrent modifications"
    )]
    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }
}
