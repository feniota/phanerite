use std::marker::PhantomData;

use pastey::paste;
use slotmap::DefaultKey;

use crate::{
    aspect::{Aspect, Storage},
    system::AspectSet,
    world::World,
};

/// A tuple of aspects that can be inserted into entities in one batch.
pub trait Bundle: AspectSet {
    type Ptrs: Copy;

    fn resolve(world: &mut World) -> Option<Self::Ptrs>;

    /// # Safety
    ///
    /// `ptrs` must come from [`Bundle::resolve`] on the same world, and no storage may have
    /// been removed or replaced since.
    unsafe fn insert(self, key: DefaultKey, ptrs: Self::Ptrs);
}

/// A batch inserter: resolves all storages of a bundle once, then inserts without any further
/// type erasure. Holds an exclusive borrow of the World until dropped.
pub struct Batch<'w, B: Bundle> {
    ptrs: B::Ptrs,
    _world: PhantomData<&'w mut World>,
}

impl<'w, B: Bundle> Batch<'w, B> {
    pub(crate) fn new(world: &'w mut World) -> Option<Self> {
        B::validate();
        Some(Self {
            ptrs: B::resolve(world)?,
            _world: PhantomData,
        })
    }

    /// Insert all components of the bundle. Replaced components are dropped.
    pub fn insert(&mut self, key: DefaultKey, bundle: B) {
        // SAFETY: ptrs came from resolve on this world, and the PhantomData keeps the exclusive
        // borrow alive, so no storage can be removed or replaced while the batch exists.
        // Bundle members are distinct types, enforced by validate().
        unsafe { B::insert(bundle, key, self.ptrs) }
    }
}

macro_rules! impl_bundle {
    ($($t:ident),*) => {
        paste! {
            impl<$($t: Aspect),*> Bundle for ($($t,)*) {
                type Ptrs = ($(*mut Storage<$t>,)*);

                fn resolve(world: &mut World) -> Option<Self::Ptrs> {
                    Some(($(world.storage_ptr::<$t>()?,)*))
                }

                unsafe fn insert(self, key: DefaultKey, ptrs: Self::Ptrs) {
                    let ($([<val_ $t:snake>],)*) = self;
                    let ($([<ptr_ $t:snake>],)*) = ptrs;
                    // SAFETY: the caller guarantees ptrs came from resolve on this world and no
                    // storage was removed or replaced since; members are distinct types.
                    unsafe {
                        $(
                            (*[<ptr_ $t:snake>]).insert(key, [<val_ $t:snake>]);
                        )*
                    }
                }
            }
        }
    };
}

impl_bundle!(T);
impl_bundle!(T, T1);
impl_bundle!(T, T1, T2);
impl_bundle!(T, T1, T2, T3);
impl_bundle!(T, T1, T2, T3, T4);
impl_bundle!(T, T1, T2, T3, T4, T5);
impl_bundle!(T, T1, T2, T3, T4, T5, T6);
impl_bundle!(T, T1, T2, T3, T4, T5, T6, T7);
impl_bundle!(T, T1, T2, T3, T4, T5, T6, T7, T8);
impl_bundle!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_bundle!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_bundle!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_bundle!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_bundle!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_bundle!(
    T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
impl_bundle!(
    T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
