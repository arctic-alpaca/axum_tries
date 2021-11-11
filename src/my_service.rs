use crate::self_polling::SelfPollingFuture;
use axum::extract::connect_info::Connected;
use axum::extract::ConnectInfo;
use axum::{AddExtension, AddExtensionLayer};
use futures::future::ready;
use std::convert::Infallible;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Debug)]
pub struct MyMaker<S, C> {
    inner: S,
    _connect_info: PhantomData<fn() -> C>,
}

impl<C, S> MyMaker<S, C> {
    pub fn new(inner: S) -> Self {
        MyMaker {
            inner,
            _connect_info: PhantomData,
        }
    }
}

impl<S, C, T> Service<T> for MyMaker<S, C>
where
    S: Clone,
    C: Connected<T>,
{
    type Response = AddExtension<S, ConnectInfo<C>>;
    type Error = Infallible;
    type Future = SelfPollingFuture<S, C>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: T) -> Self::Future {
        let connect_info = ConnectInfo(C::connect_info(target));
        let svc = AddExtensionLayer::new(connect_info).layer(self.inner.clone());
        SelfPollingFuture::new(ready(Ok(svc)))
    }
}
impl<S, C> Clone for MyMaker<S, C>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _connect_info: PhantomData,
        }
    }
}
