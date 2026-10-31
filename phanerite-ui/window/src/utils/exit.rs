use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::Stream;

pub(crate) struct ExitGuard(async_channel::Sender<()>);

#[derive(Clone)]
pub(crate) struct Exit(Pin<Box<async_channel::Receiver<()>>>);

impl ExitGuard {
    pub(crate) fn exit(self) {}
}

impl Future for Exit {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.as_mut().poll_next(cx).map(|_| ())
    }
}

pub(crate) fn exit_guard() -> (ExitGuard, Exit) {
    let (tx, rx) = async_channel::bounded(1);
    (ExitGuard(tx), Exit(Box::pin(rx)))
}
