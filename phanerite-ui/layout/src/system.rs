use pastey::paste;
use std::{any::TypeId, marker::PhantomData};

use crate::{aspect::Aspect, context::Context, world::World};

/// A set of [`Aspect`]s that will be passed to a [`System`]
pub trait AspectSet: 'static {
    /// Check if there is any duplicated aspect about which a system cares.
    ///
    /// # Panics
    ///
    /// This method SHOULD panic once a duplicated aspect is detected, since it is undefined
    /// behavior for systems to care about the same aspect more than once and will lead to
    /// unexpected problems.
    fn validate();

    /// The [`TypeId`]s of all aspects in the set.
    fn type_ids() -> Vec<TypeId>;
}

impl AspectSet for () {
    fn type_ids() -> Vec<TypeId> {
        Vec::new()
    }
    fn validate() {}
}

/// A unit of logic executed repeatedly on every entity that has all aspects in its query.
///
/// Typically you do not need to implement this trait yourself. This crate provides blanket
/// implementations for all functions that meet the contraints, as decribed later.
///
/// Systems are typically functions **with `'static` lifetimes**, that is, functions defined with
/// the `fn` keyword, or closures that do not capture variables with limited lifetimes.
///
/// The first parameters of system functions are always [`&mut Context`](crate::context::Context).
/// Queries (with which aspects does this system want to process?) are later specified as
/// [`&mut Aspect`](crate::aspect::Aspect)s in the parameter list. You can query up to 16 aspects
/// per system.
///
/// For example, the `movement` is a system that update the positions of entities with both
/// `Position` and `Velocity` aspects.
///
/// ```
/// use phanerite_layout::taffy::Style;
/// use phanerite_layout::{Aspect, Context, NodeMetadata, World};
///
/// #[derive(Aspect)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// #[derive(Aspect)]
/// struct Velocity {
///     x: f32,
///     y: f32,
/// }
///
/// fn movement(_ctx: &mut Context, pos: &mut Position, vel: &mut Velocity) {
///     pos.x += vel.x;
///     pos.y += vel.y;
/// }
///
/// let mut world = World::new();
/// world.insert_system(movement);
/// ```
///
/// When you finish defining a system, you can insert it into the World by calling
/// [`World::insert_system`] with your system function.
///
/// # Performance
///
/// There is no archetype in this crate, so iterations are implemented in a very simple manner.
/// The first [`Aspect`] in your query is **always** fully iterated over, and the entities (retrieved
/// from the first aspect's storage) are used to query the remaining storages, and then to determine
/// whether to call the system.
///
/// So, let's say you have 2 aspects, `A` with 10,000 entries and `B` with 200 entries. When writing
/// a system for `(A, B)`, you should define your function as
///
/// ```
/// # use phanerite_layout::Context;
/// # struct A;
/// # struct B;
/// # #[allow(unused)]
/// fn system(ctx: &mut Context, b: &mut B, a: &mut A) {}
/// ```
///
/// but not `Fn(&mut Context, &mut A, &mut B)`, because the latter will lead to 9,800 useless
/// iterations.
///
/// In other words, **always put the sparsest aspect in the front**.
///
/// # Safety
///
/// The system receive *raw* mutable references to the aspects at the same time. So trying to retrive
/// the same aspect multiple times leads to aliasing the same mutable reference, (as there can only
/// be one storage per aspect,) which is undefined behavior, and is therefore rejected.
///
/// There is no way to enforce the limitation without macros in current version of Rust, so the check
/// is performed at runtime, when you insert the system into the world.
///
/// ```should_panic
/// use phanerite_layout::{Aspect, Context, World};
/// #[derive(Aspect)]
/// struct A;
///
/// let mut world = World::new();
/// world.insert_system(|_: &mut Context, aspect1: &mut A, aspect2: &mut A| {}); // Panics
/// ```
///
/// # Context
///
/// When you need to work with the world itself in a system, [`Context`] is there to help. See
/// its doc for more details. But please note that the same safety rule is still enforced even when
/// you access the aspect through the [`Context`].
///
/// # Implement this trait yourself
///
/// It is possible to implement this trait yourself and insert it into the system. In that case, you
/// just need to use `()` as the type parameter in this trait.
///
/// The World would always run [`AspectSet::validate`], even if the system is not the blanket
/// implementation, while the validation is only reasonable when the system receives raw mutable
/// references to the aspects, which may not fit your case. So there is a blanket implementation
/// for [`()`](unit) whose [`validate()`](AspectSet::validate) does nothing.
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
    /// Box a system into a type-erased wrapper.
    pub(crate) fn new_boxed(system: F) -> Box<Self> {
        Box::new(Self {
            system,
            _marker: PhantomData,
        })
    }
}

/// Count the number of expressions in a comma-separated list.
macro_rules! count_exprs {
    () => { 0usize };
    ($head:expr, $($tail:expr,)*) => {
        1usize + count_exprs!($($tail,)*)
    };
}
/// Implement [`AspectSet`] and [`System`] for a tuple of aspects, plus the blanket
/// `System` impl for matching closures.
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
    /// Key identifying a system registered in the [`World`].
    pub struct SystemKey;
);
