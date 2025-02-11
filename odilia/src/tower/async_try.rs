#![allow(clippy::module_name_repetitions)]

use crate::tower::from_state::TryFromState;
use futures::TryFutureExt;
use std::{
	future::Future,
	marker::PhantomData,
	task::{Context, Poll},
};
use tower::{Layer, Service};

impl<T, S, U> AsyncTryFrom<(S, T)> for U
where
	U: TryFromState<S, T>,
{
	type Error = U::Error;
	type Future = U::Future;
	fn try_from_async(value: (S, T)) -> Self::Future {
		U::try_from_state(value.0, value.1)
	}
}

pub trait AsyncTryFrom<T>: Sized {
	type Error;
	type Future: Future<Output = Result<Self, Self::Error>>;

	fn try_from_async(value: T) -> Self::Future;
}
pub trait AsyncTryInto<T>: Sized {
	type Error;
	type Future: Future<Output = Result<T, Self::Error>>;

	fn try_into_async(self) -> Self::Future;
}
impl<T, U: AsyncTryFrom<T>> AsyncTryInto<U> for T {
	type Error = U::Error;
	type Future = U::Future;
	fn try_into_async(self: T) -> Self::Future {
		U::try_from_async(self)
	}
}

pub struct AsyncTryIntoService<O, I: AsyncTryInto<O>, S, R, Fut1> {
	inner: S,
	_marker: PhantomData<fn(O, I, Fut1) -> R>,
}
impl<O, E, I: AsyncTryInto<O, Error = E>, S, R, Fut1> AsyncTryIntoService<O, I, S, R, Fut1> {
	pub fn new(inner: S) -> Self {
		AsyncTryIntoService { inner, _marker: PhantomData }
	}
}
pub struct AsyncTryIntoLayer<O, I: AsyncTryInto<O>> {
	_marker: PhantomData<fn(I) -> O>,
}
impl<O, I: AsyncTryInto<O>> Clone for AsyncTryIntoLayer<O, I> {
	fn clone(&self) -> Self {
		AsyncTryIntoLayer { _marker: PhantomData }
	}
}
impl<O, E, I: AsyncTryInto<O, Error = E>> AsyncTryIntoLayer<O, I> {
	pub fn new() -> Self {
		AsyncTryIntoLayer { _marker: PhantomData }
	}
}

impl<I: AsyncTryInto<O>, O, S, Fut1> Layer<S> for AsyncTryIntoLayer<O, I>
where
	S: Service<O, Future = Fut1>,
{
	type Service = AsyncTryIntoService<O, I, S, <S as Service<O>>::Response, Fut1>;
	fn layer(&self, inner: S) -> Self::Service {
		AsyncTryIntoService::new(inner)
	}
}

impl<O, I: AsyncTryInto<O>, S, R, Fut1> Clone for AsyncTryIntoService<O, I, S, R, Fut1>
where
	S: Clone,
{
	fn clone(&self) -> Self {
		AsyncTryIntoService { inner: self.inner.clone(), _marker: PhantomData }
	}
}

impl<O, E, E2, I: AsyncTryInto<O>, S, R, Fut1> Service<I> for AsyncTryIntoService<O, I, S, R, Fut1>
where
	I: AsyncTryInto<O, Error = E2>,
	E: From<E2>,
	S: Service<O, Response = R, Future = Fut1, Error = E> + Clone,
	Fut1: Future<Output = Result<R, E>>,
{
	type Response = R;
	type Future = Flatten<AndThenCall<ErrInto<I::Future, E>, S, O, E>>;
	type Error = E;
	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: I) -> Self::Future {
		let clone = self.inner.clone();
		let inner = std::mem::replace(&mut self.inner, clone);
		req.try_into_async().err_into::<E>().and_then_call(inner).flatten()
		/*
			    async move {
				    match req.try_into_async().await {
					    Ok(resp) => inner.call(resp).err_into().await,
					    Err(e) => Err(e.into()),
				    }
			    }
		*/
	}
}

use futures::future::{err, Either, ErrInto, Flatten, FutureExt, Ready};
use std::pin::Pin;
use tower::util::Oneshot;
use tower::ServiceExt;

#[pin_project::pin_project]
pub struct AndThenCall<F, S, O, E> {
	#[pin]
	fut: F,
	svc: S,
	_marker: PhantomData<(E, O)>,
}
impl<F, S, O, E> Future for AndThenCall<F, S, O, E>
where
	S: Service<O, Error = E> + Clone,
	F: Future<Output = Result<O, E>>,
{
	type Output = Either<Ready<Result<S::Response, E>>, Oneshot<S, O>>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.project();
		match this.fut.poll(cx) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(Err(e)) => Poll::Ready(err(e).left_future()),
			Poll::Ready(Ok(input)) => {
				Poll::Ready(Either::Right(this.svc.clone().oneshot(input)))
			}
		}
	}
}

trait AndThenCallExt: Future {
	fn and_then_call<S, O, E>(self, svc: S) -> AndThenCall<Self, S, O, E>
	where
		Self: Sized,
	{
		AndThenCall { fut: self, svc, _marker: PhantomData }
	}
}
impl<F> AndThenCallExt for F where F: Future {}
