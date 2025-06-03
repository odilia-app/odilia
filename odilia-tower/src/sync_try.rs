//! Synchronous version of [`crate::async_try`].
//! Create [`tower::Service`]s out of the ability to convert values via [`TryFrom`] and `TryInto`].

use core::{
	future::Future,
	marker::PhantomData,
	task::{Context, Poll},
};

use futures::future::{err, Either, Ready};
use static_assertions::const_assert_eq;
use tower::{Layer, Service};

/// A service which maps from a new input into the inner [`tower::Service`]'s input based on a
/// [`TryInto`] implementation between the two types.
pub struct TryIntoService<O, I: TryInto<O>, S, R, Fut1> {
	inner: S,
	_marker: PhantomData<fn(O, I, Fut1) -> R>,
}
impl<O, E, I: TryInto<O, Error = E>, S, R, Fut1> TryIntoService<O, I, S, R, Fut1> {
	/// Create a new [`TryIntoService`] from an inner [`tower::Service`].
	pub fn new(inner: S) -> Self {
		TryIntoService { inner, _marker: PhantomData }
	}
}
/// A [ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts) describing a
/// layer which runs the corresponding [`TryInto`] implementation.
pub struct TryIntoLayer<O, I: TryInto<O>> {
	_marker: PhantomData<fn(I) -> O>,
}
const_assert_eq!(size_of::<TryIntoLayer<u32, u16>>(), 0);

impl<O, E, I: TryInto<O, Error = E>> Default for TryIntoLayer<O, I> {
	fn default() -> Self {
		TryIntoLayer { _marker: PhantomData }
	}
}
impl<O, E, I: TryInto<O, Error = E>> TryIntoLayer<O, I> {
	/// Create a new [`TryIntoLayer`] from the generic types.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}
}

impl<I: TryInto<O>, O, S, Fut1> Layer<S> for TryIntoLayer<O, I>
where
	S: Service<O, Future = Fut1>,
{
	type Service = TryIntoService<O, I, S, <S as Service<O>>::Response, Fut1>;
	fn layer(&self, inner: S) -> Self::Service {
		TryIntoService::new(inner)
	}
}

impl<O, I: TryInto<O>, S, R, Fut1> Clone for TryIntoService<O, I, S, R, Fut1>
where
	S: Clone,
{
	fn clone(&self) -> Self {
		TryIntoService { inner: self.inner.clone(), _marker: PhantomData }
	}
}

impl<O, E, I: TryInto<O>, S, R, Fut1> Service<I> for TryIntoService<O, I, S, R, Fut1>
where
	I: TryInto<O>,
	E: From<<I as TryInto<O>>::Error>,
	S: Service<O, Response = R, Future = Fut1>,
	Fut1: Future<Output = Result<R, E>>,
{
	type Response = R;
	type Future = Either<Fut1, Ready<Result<R, E>>>;
	type Error = E;
	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: I) -> Self::Future {
		match req.try_into() {
			Ok(o) => Either::Left(self.inner.call(o)),
			Err(e) => Either::Right(err(e.into())),
		}
	}
}
