use axum::extract::ConnectInfo;
use axum::AddExtension;
use futures::future::Ready;
use futures::Future;
use pin_project::pin_project;
use std::convert::Infallible;

use std::pin::Pin;

#[pin_project]
pub struct SelfPollingFuture<S, C> {
    #[pin]
    future: Ready<Result<AddExtension<S, ConnectInfo<C>>, Infallible>>,
}

impl<S, C> Future for SelfPollingFuture<S, C>
where
    Ready<Result<AddExtension<S, ConnectInfo<C>>, Infallible>>: Future,
{
    type Output = <Ready<Result<AddExtension<S, ConnectInfo<C>>, Infallible>> as Future>::Output;
    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}

impl<S, C> SelfPollingFuture<S, C> {
    pub fn new(future: Ready<Result<AddExtension<S, ConnectInfo<C>>, Infallible>>) -> Self {
        Self { future }
    }
}
