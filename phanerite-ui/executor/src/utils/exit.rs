use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::Stream;

pub(crate) struct ExitGuard {
    _tx: async_channel::Sender<()>,
}

#[derive(Clone)]
pub(crate) struct Exit {
    rx: Pin<Box<async_channel::Receiver<()>>>,
}

impl ExitGuard {
    pub(crate) fn exit(self) {}
}

impl Future for Exit {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.rx.as_mut().poll_next(cx).map(|_| ())
    }
}

pub(crate) fn exit_guard() -> (ExitGuard, Exit) {
    let (_tx, rx) = async_channel::bounded(1);
    (ExitGuard { _tx }, Exit { rx: Box::pin(rx) })
}
