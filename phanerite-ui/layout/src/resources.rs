use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Type-indexed singleton storage for arbitrary `'static` values, shared by all systems.
#[derive(Default)]
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    /// Insert a resource, returning the previous value of the same type if any.
    pub fn insert<R: 'static>(&mut self, value: R) -> Option<R> {
        self.map
            .insert(TypeId::of::<R>(), Box::new(value))
            .map(|old| *old.downcast::<R>().expect("resource type mismatch"))
    }

    /// Read a resource, returning `None` if it has not been inserted.
    pub fn get<R: 'static>(&self) -> Option<&R> {
        self.map
            .get(&TypeId::of::<R>())
            .map(|value| value.downcast_ref::<R>().expect("resource type mismatch"))
    }

    /// Mutably access a resource, returning `None` if it has not been inserted.
    pub fn get_mut<R: 'static>(&mut self) -> Option<&mut R> {
        self.map
            .get_mut(&TypeId::of::<R>())
            .map(|value| value.downcast_mut::<R>().expect("resource type mismatch"))
    }

    /// Remove a resource, returning it if present.
    pub fn remove<R: 'static>(&mut self) -> Option<R> {
        self.map
            .remove(&TypeId::of::<R>())
            .map(|value| *value.downcast::<R>().expect("resource type mismatch"))
    }
}
