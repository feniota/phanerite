use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
};

#[derive(Default)]
pub(crate) struct GlobalStates {
    registry: HashMap<TypeId, Box<dyn Any>>,
}

impl GlobalStates {
    pub(crate) fn register<T: Any>(&mut self, value: T) -> Result<(), T> {
        match self.registry.entry(TypeId::of::<T>()) {
            Entry::Vacant(entry) => {
                entry.insert(Box::new(value));
                Ok(())
            }
            Entry::Occupied(_) => Err(value),
        }
    }

    pub(crate) fn get<T: Any>(&self) -> Option<&T> {
        self.registry.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub(crate) fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.registry.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }
}
