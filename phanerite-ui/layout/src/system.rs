use pastey::paste;
use std::{any::TypeId, marker::PhantomData};

use crate::{aspect::Aspect, context::Context, world::World};

/// A set of [`Aspect`]s that will be passed to a [`System`]
pub trait AspectSet: 'static {
    /// Check if there is any duplicated aspect about which a system cares.
    ///
    /// # Panics
    ///
    /// This method SHOULD panic once a duplicated aspect is detected, since it is an undefined
    /// behavior for systems to care about the same aspect more than once and will lead to
    /// unexpected problems.
    fn validate();

    fn type_ids() -> Vec<TypeId>;
}

pub trait System<T: AspectSet>: 'static {
    /// Iterate over the associated aspects and execute the inner system function.
    fn execute(&self, world: &mut World);
}

/// Type-erased [`System`]
pub(crate) trait ErasedSystem {
    // Validation is performed on the AspectSet before the System is inserted into the World.
    fn execute(&self, world: &mut World);
}

/// A wrapper containing a [`System`] that implements the [`ErasedSystem`] trait.
pub(crate) struct SystemWrapper<F, T>
where
    F: System<T>,
    T: AspectSet,
{
    _marker: PhantomData<T>,
    system: F,
}

impl<F, T> ErasedSystem for SystemWrapper<F, T>
where
    F: System<T>,
    T: AspectSet,
{
    fn execute(&self, world: &mut World) {
        self.system.execute(world);
    }
}
impl<F, T> SystemWrapper<F, T>
where
    F: System<T>,
    T: AspectSet,
{
    pub(crate) fn new_boxed(system: F) -> Box<Self> {
        Box::new(Self {
            system,
            _marker: PhantomData,
        })
    }
}
macro_rules! count_exprs {
    () => { 0usize };
    ($head:expr, $($tail:expr,)*) => {
        1usize + count_exprs!($($tail,)*)
    };
}
macro_rules! implement_systems {
    ( $idx:ident $(, $aspect:ident )* ) =>{
        impl<$idx: $crate::Aspect, $($aspect: $crate::Aspect,)*> AspectSet for ($idx, $($aspect,)*) {
            fn validate() {
                use std::collections::HashSet;
                use std::any::type_name;
                let mut type_ids = HashSet::new();
                type_ids.insert(TypeId::of::<$idx>());

                // emulate goto (to prevent multiple panic strings being compiled into the binary)
                let conflicting_type = loop {
                    $(
                        if !type_ids.insert(TypeId::of::<$aspect>()) {
                            break type_name::<$aspect>();
                        }
                    )*
                    return;
                };
                panic!(
                    "phanerite_layout: You cannot retrieve the same type multiple times in a single System, but found a type `{}` that was duplicated. (while processing `{}`)",
                    conflicting_type,
                    type_name::<Self>()
                )
            }

            fn type_ids() -> Vec<TypeId> {
                let mut ret = Vec::with_capacity(count_exprs!{ $idx, $($aspect,)* });
                ret.push(TypeId::of::<$idx>());
                $(
                    ret.push(TypeId::of::<$aspect>());
                )*
                ret
            }
        }

        impl<F, $idx, $($aspect,)*> $crate::System<($idx, $($aspect,)*)> for F
        where
            $idx: $crate::Aspect,
            $($aspect: $crate::Aspect,)*
            F: 'static + Fn(&mut $crate::Context, &mut $idx $(, &mut $aspect)*) -> ()
        {
            fn execute(&self, world: &mut $crate::World)
            {
                paste!{
                    let Some(storage_idx) = world.storage_ptr::<$idx>() else {return};
                    $(
                        let Some([<storage_ $aspect:snake>]) = world.storage_ptr::<$aspect>()
                        else {
                            return;
                        };
                    )*
                    let mut ctx = world.context(<($idx, $($aspect,)*) as $crate::AspectSet>::type_ids());
                    // SAFETY: All aspect types are distinct, as enforced by `AspectSet::validate`,
                    // so these pointers refer to different storages. They were derived from the
                    // exclusive `world` borrow and remain valid throughout this iteration.
                    // Context's accessors refuse to touch any queried storage, so the pointers
                    // can neither be invalidated nor aliased through it.
                    unsafe {
                        let storage_idx = &mut *storage_idx;
                        $(
                            let [<storage_ $aspect:snake>] = &mut *[<storage_ $aspect:snake>];
                        )*
                        for (key, asp_idx) in storage_idx.iter_mut() {
                            $(
                                let Some([<asp_ $aspect:snake>]) = [<storage_ $aspect:snake>].get_mut(key) else {continue};
                            )*
                            ctx.set_node(key);
                            self(&mut ctx, asp_idx $(,[<asp_ $aspect:snake>])*);
                        }
                    }
                }
            }
        }
    }
}

impl<T: Aspect> AspectSet for (T,) {
    fn validate() {}

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}
impl<F, T> System<(T,)> for F
where
    T: Aspect,
    F: 'static + Fn(&mut Context, &mut T),
{
    fn execute(&self, world: &mut World) {
        let Some(storage) = world.storage_ptr::<T>() else {
            return;
        };
        let mut ctx = world.context(<(T,) as AspectSet>::type_ids());
        // SAFETY: same contract as the multi-aspect impls above; the single queried storage
        // is protected from Context access by the query-conflict check.
        unsafe {
            let storage = &mut *storage;
            for (key, asp) in storage.iter_mut() {
                ctx.set_node(key);
                self(&mut ctx, asp);
            }
        }
    }
}

implement_systems!(T, T1);
implement_systems!(T, T1, T2);
implement_systems!(T, T1, T2, T3);
implement_systems!(T, T1, T2, T3, T4);
implement_systems!(T, T1, T2, T3, T4, T5);
implement_systems!(T, T1, T2, T3, T4, T5, T6);
implement_systems!(T, T1, T2, T3, T4, T5, T6, T7);
implement_systems!(T, T1, T2, T3, T4, T5, T6, T7, T8);
implement_systems!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9);
implement_systems!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
implement_systems!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
implement_systems!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
implement_systems!(T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
implement_systems!(
    T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
implement_systems!(
    T, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

slotmap::new_key_type!(
    pub struct SystemKey;
);
