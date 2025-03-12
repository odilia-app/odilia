//! A trait [`AsyncTryFrom`], its associated layer [`AsyncTryIntoLayer`], and a blanket
//! implementation of [`odilia_common::from_state::TryFromState`].
//!
//! Due to the blanket implementation, this is required to be a trait defined by us.
//! This means that even in the future, if a crate were to become available that offered similar
//! functionality, we could still not remove this.

use core::{
	future::Future,
	marker::PhantomData,
	task::{Context, Poll},
};
use futures::TryFutureExt;
#[allow(clippy::module_name_repetitions)]
use odilia_common::from_state::TryFromState;
use static_assertions::const_assert_eq;
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

/// An async version of [`TryFrom`] with an associated future.
pub trait AsyncTryFrom<T>: Sized {
	/// The possible conversion error.
	type Error;
	/// Named future (for use with other [`tower`] components.
	/// Will be dropped in favour of ITIAT or RTN if either of them land.
	type Future: Future<Output = Result<Self, Self::Error>>;

	/// Attempt to asynchronously convert a value from `T` to [`Self`].
	fn try_from_async(value: T) -> Self::Future;
}
/// An async version of [`TryInto`] with an associated future.
pub trait AsyncTryInto<T>: Sized {
	/// The possible conversion error.
	type Error;
	/// Named future (for use with other [`tower`] components.
	/// Will be dropped in favour of ITIAT or RTN if either of them land.
	type Future: Future<Output = Result<T, Self::Error>>;

	/// Attempt to asynchronously convert a value from `T` to [`Self`].
	fn try_into_async(self) -> Self::Future;
}
impl<T, U: AsyncTryFrom<T>> AsyncTryInto<U> for T {
	type Error = U::Error;
	type Future = U::Future;
	fn try_into_async(self: T) -> Self::Future {
		U::try_from_async(self)
	}
}

/// A service which applies an [`AsyncTryInto`] transformation to a service's input.
pub struct AsyncTryIntoService<O, I, S, R, Fut1> {
	inner: S,
	_marker: PhantomData<fn(O, I, Fut1) -> R>,
}

impl<O, I, S, R, Fut1> AsyncTryIntoService<O, I, S, R, Fut1> {
	/// Wrap the inner service with an [`AsyncTryInto`] function transforming its input.
	pub fn new(inner: S) -> Self {
		AsyncTryIntoService { inner, _marker: PhantomData }
	}
}

/// A [ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts) representing
/// a [`AsyncTryInto`] as a [`tower::Layer`].
pub struct AsyncTryIntoLayer<O, I> {
	_marker: PhantomData<fn(I) -> O>,
}
const_assert_eq!(size_of::<AsyncTryIntoLayer<(), ()>>(), 0);
const_assert_eq!(size_of::<AsyncTryIntoLayer<u128, Option<u16>>>(), 0);

impl<O, I> Clone for AsyncTryIntoLayer<O, I> {
	fn clone(&self) -> Self {
		AsyncTryIntoLayer { _marker: PhantomData }
	}
}
impl<O, I> Default for AsyncTryIntoLayer<O, I> {
	fn default() -> Self {
		AsyncTryIntoLayer { _marker: PhantomData }
	}
}
impl<O, I> AsyncTryIntoLayer<O, I> {
	/// Create a new `AsyncTryIntoLayer` from generic types.
	#[must_use]
	pub fn new() -> Self {
		AsyncTryIntoLayer { _marker: PhantomData }
	}
}

impl<I, O, S, Fut1> Layer<S> for AsyncTryIntoLayer<O, I>
where
	S: Service<O, Future = Fut1>,
	I: AsyncTryInto<O>,
{
	type Service = AsyncTryIntoService<O, I, S, <S as Service<O>>::Response, Fut1>;
	fn layer(&self, inner: S) -> Self::Service {
		AsyncTryIntoService::new(inner)
	}
}

impl<O, I, S, R, Fut1> Clone for AsyncTryIntoService<O, I, S, R, Fut1>
where
	S: Clone,
	I: AsyncTryInto<O>,
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
		let inner = core::mem::replace(&mut self.inner, clone);
		AndThenCall {
			fut: req.try_into_async().err_into::<E>(),
			svc: inner,
			_marker: PhantomData,
		}
		.flatten()
	}
}

use core::pin::Pin;
use futures::future::{err, Either, ErrInto, Flatten, FutureExt, Ready};
use tower::util::Oneshot;
use tower::ServiceExt;

/// A version of [`tower::util::future::AndThenFuture`] that is not generic over an un-namable
/// future type.
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
