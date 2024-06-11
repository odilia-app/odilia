#![allow(clippy::module_name_repetitions)]

use crate::tower::async_try::AsyncTryInto;
use atspi::Event;
use futures::{future::Map, join, FutureExt};
use std::{
	convert::Infallible,
	future::Future,
	marker::PhantomData,
	task::{Context, Poll},
};
use tower::Service;

type Request = Event;

pub trait Handler<T, E> {
	type Response;
	type Future: Future<Output = Self::Response>;
	fn into_service<R>(self) -> HandlerService<Self, T, E, R>
	where
		Self: Sized,
	{
		HandlerService::new(self)
	}
	fn call(self, req: E, params: T) -> Self::Future;
}

impl<F, Fut, E, R> Handler<((),), E> for F
where
	F: FnOnce() -> Fut,
	Fut: Future<Output = R>,
{
	type Response = R;
	type Future = Fut;
	fn call(self, _req: E, _param: ((),)) -> Self::Future {
		self()
	}
}

impl<F, Fut, E, R> Handler<(Request,), E> for F
where
	F: FnOnce(E) -> Fut,
	Fut: Future<Output = R>,
{
	type Response = R;
	type Future = Fut;
	fn call(self, req: E, _params: (Request,)) -> Self::Future {
		self(req)
	}
}

macro_rules! impl_handler {
    ($($type:ident,)+) => {
        #[allow(non_snake_case)]
        impl<F, Fut, E, R, $($type,)+> Handler<($($type,)+), E> for F
        where
            F: FnOnce(E, $($type,)+) -> Fut + Send,
            Fut: Future<Output = R> + Send,
            $($type: Send,)+
            E: Send {
      type Response = R;
      type Future = impl Future<Output = R>;
      fn call(self, req: E, params: ($($type,)+)) -> Self::Future {
          let ($($type,)+) = params;
          self(req, $($type,)+)
      }
    }
}
}
impl_handler!(T1, T2,);
impl_handler!(T1, T2, T3,);
impl_handler!(T1, T2, T3, T4,);
impl_handler!(T1, T2, T3, T4, T5,);
impl_handler!(T1, T2, T3, T4, T5, T6,);
impl_handler!(T1, T2, T3, T4, T5, T6, T7,);

#[allow(clippy::type_complexity)]
pub struct HandlerService<H, T, E, R> {
	handler: H,
	_marker: PhantomData<fn(E, T) -> R>,
}
impl<H, T, E, R> Clone for HandlerService<H, T, E, R>
where
	H: Clone,
{
	fn clone(&self) -> Self {
		HandlerService { handler: self.handler.clone(), _marker: PhantomData }
	}
}
impl<H, T, E, R> HandlerService<H, T, E, R> {
	fn new(handler: H) -> Self
	where
		H: Handler<T, E>,
	{
		HandlerService { handler, _marker: PhantomData }
	}
}

impl<H, T, E, R> Service<(E, T)> for HandlerService<H, T, E, R>
where
	H: Handler<T, E> + Clone,
{
	type Response = H::Response;
	type Future = impl Future<Output = Result<H::Response, Infallible>>;
	type Error = Infallible;

	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, params: (E, T)) -> Self::Future {
		self.handler.clone().call(params.0, params.1).map(|o| Ok(o))
	}
}
