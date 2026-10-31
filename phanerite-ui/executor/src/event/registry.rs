//! Runtime type indexing and storage for explicitly registered observers.

use std::{
    any::TypeId,
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use super::{
    Event, ObserverCallback, ObserverId,
    observer::{ErasedObserver, erase_observer},
    payload::ErasedEvent,
};

pub(super) struct ObserverRegistry {
    observers_by_event: HashMap<TypeId, Vec<Box<dyn ErasedObserver>>>,
    pub(super) id: u64,
    pub(super) next_observer: Arc<AtomicU64>,
}

impl ObserverRegistry {
    pub(super) fn new() -> Self {
        static NEXT_REGISTRY: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_REGISTRY
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .expect("observer registry ID space exhausted");
        Self {
            observers_by_event: HashMap::new(),
            id,
            next_observer: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(super) fn register_event<E: Event>(&mut self) {
        self.observers_by_event
            .entry(TypeId::of::<E>())
            .or_default();
    }

    pub(super) fn insert<E, F, M: 'static>(&mut self, callback: F) -> ObserverId
    where
        E: Event,
        F: ObserverCallback<E, M>,
    {
        let id = ObserverId {
            registry: self.id,
            sequence: self
                .next_observer
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("observer ID space exhausted"),
        };
        self.observers_by_event
            .entry(TypeId::of::<E>())
            .or_default()
            .push(erase_observer(id, callback));
        id
    }

    pub(super) fn remove(&mut self, id: ObserverId) -> bool {
        if id.registry != self.id {
            return false;
        }
        for observers in self.observers_by_event.values_mut() {
            if let Some(index) = observers.iter().position(|observer| observer.id() == id) {
                observers.remove(index);
                return true;
            }
        }
        false
    }

    pub(super) fn insert_erased(&mut self, event: TypeId, observer: Box<dyn ErasedObserver>) {
        self.observers_by_event
            .entry(event)
            .or_default()
            .push(observer);
    }

    pub(super) async fn dispatch(&mut self, event: ErasedEvent) {
        if let Some(observers) = self.observers_by_event.get_mut(&event.type_id) {
            for observer in observers {
                observer.call(event.payload.clone()).await;
            }
        }
    }
}
