use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::Stream;

use crate::event::Event;

/// Requests application shutdown.
#[derive(Event, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shutdown;

pub struct ExitGuard {
    _tx: async_channel::Sender<()>,
}

#[derive(Clone)]
pub struct Exit {
    rx: Pin<Box<async_channel::Receiver<()>>>,
}

impl ExitGuard {
    pub fn exit(self) {}
}

impl Future for Exit {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.rx.as_mut().poll_next(cx).map(|_| ())
    }
}

pub fn exit_guard() -> (ExitGuard, Exit) {
    let (_tx, rx) = async_channel::bounded(1);
    (ExitGuard { _tx }, Exit { rx: Box::pin(rx) })
}

impl Exit {
    pub async fn race<F>(self, future: F)
    where
        F: Future,
    {
        futures_lite::future::race(self, async {
            future.await;
        })
        .await
    }

    // pub async fn race_with_return<F, R>(self, future: F) -> Option<R>
    // where
    //     F: Future<Output = R>,
    // {
    //     futures_lite::future::race(
    //         async {
    //             self.await;
    //             None
    //         },
    //         async { Some(future.await) },
    //     )
    //     .await
    // }
}
