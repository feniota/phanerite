//! Callback adaptation and type erasure for caller-provided observers.

use std::{future::Future, marker::PhantomData, pin::Pin};

use super::{Event, On, payload::EventPayload};

/// A listener handle belonging to a particular dispatcher.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObserverId {
    pub(super) registry: u64,
    pub(super) sequence: u64,
}

/// Observer futures run on the dispatcher thread and need not be `Send`.
pub type ObserverFuture<'a> = Pin<Box<dyn Future<Output = ()> + 'a>>;

#[doc(hidden)]
pub struct SyncCallback;
#[doc(hidden)]
pub struct AsyncCallback;

/// Adapts an ordinary or asynchronous callback accepting `On<E>`.
///
/// The marker is inferred at call sites. Async closures can borrow their
/// captured state across `.await`. This trait describes invocation; it does
/// not register or construct a concrete listener.
pub trait ObserverCallback<E: Event, Marker>: 'static {
    #[doc(hidden)]
    fn call(&mut self, event: On<E>) -> ObserverFuture<'_>;
}

impl<E, F> ObserverCallback<E, SyncCallback> for F
where
    E: Event,
    F: FnMut(On<E>) + 'static,
{
    fn call(&mut self, event: On<E>) -> ObserverFuture<'_> {
        self(event);
        Box::pin(std::future::ready(()))
    }
}

impl<E, F> ObserverCallback<E, AsyncCallback> for F
where
    E: Event,
    F: AsyncFnMut(On<E>) + 'static,
{
    fn call(&mut self, event: On<E>) -> ObserverFuture<'_> {
        Box::pin(async move { self(event).await })
    }
}

pub(super) trait ErasedObserver {
    fn id(&self) -> ObserverId;
    fn call(&mut self, event: EventPayload) -> ObserverFuture<'_>;
}

struct TypedObserver<E, F, M> {
    id: ObserverId,
    callback: F,
    marker: PhantomData<fn(E, M)>,
}

impl<E, F, M> ErasedObserver for TypedObserver<E, F, M>
where
    E: Event,
    F: ObserverCallback<E, M>,
{
    fn id(&self) -> ObserverId {
        self.id
    }

    fn call(&mut self, event: EventPayload) -> ObserverFuture<'_> {
        // ErasedEvent construction and registry lookup preserve the payload type.
        let event = event.downcast::<E>().expect("event registry type mismatch");
        self.callback.call(On {
            event,
            observer: self.id,
        })
    }
}

pub(super) fn erase_observer<E, F, M: 'static>(
    id: ObserverId,
    callback: F,
) -> Box<dyn ErasedObserver>
where
    E: Event,
    F: ObserverCallback<E, M>,
{
    Box::new(TypedObserver {
        id,
        callback,
        marker: PhantomData::<fn(E, M)>,
    })
}
