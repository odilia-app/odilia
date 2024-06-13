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

pub trait Handler<T> {
	type Response;
	type Future: Future<Output = Self::Response>;
	fn into_service<R>(self) -> HandlerService<Self, T, R>
	where
		Self: Sized,
	{
		HandlerService::new(self)
	}
	fn call(self, params: T) -> Self::Future;
}

macro_rules! impl_handler {
    ($($type:ident,)+) => {
        #[allow(non_snake_case)]
        impl<F, Fut, R, $($type,)+> Handler<($($type,)+)> for F
        where
            F: FnOnce($($type,)+) -> Fut + Send,
            Fut: Future<Output = R> + Send,
            $($type: Send,)+ {
      type Response = R;
      type Future = impl Future<Output = R>;
      fn call(self, params: ($($type,)+)) -> Self::Future {
          let ($($type,)+) = params;
          self($($type,)+)
      }
    }
}
}
impl_handler!(T1,);
impl_handler!(T1, T2,);
impl_handler!(T1, T2, T3,);
impl_handler!(T1, T2, T3, T4,);
impl_handler!(T1, T2, T3, T4, T5,);
impl_handler!(T1, T2, T3, T4, T5, T6,);
impl_handler!(T1, T2, T3, T4, T5, T6, T7,);

#[allow(clippy::type_complexity)]
pub struct HandlerService<H, T, R> {
	handler: H,
	_marker: PhantomData<fn(T) -> R>,
}
impl<H, T, R> Clone for HandlerService<H, T, R>
where
	H: Clone,
{
	fn clone(&self) -> Self {
		HandlerService { handler: self.handler.clone(), _marker: PhantomData }
	}
}
impl<H, T, R> HandlerService<H, T, R> {
	fn new(handler: H) -> Self
	where
		H: Handler<T>,
	{
		HandlerService { handler, _marker: PhantomData }
	}
}

impl<H, T, R> Service<T> for HandlerService<H, T, R>
where
	H: Handler<T> + Clone,
{
	type Response = H::Response;
	type Future = impl Future<Output = Result<H::Response, Infallible>>;
	type Error = Infallible;

	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, params: T) -> Self::Future {
		self.handler.clone().call(params).map(|o| Ok(o))
	}
}
