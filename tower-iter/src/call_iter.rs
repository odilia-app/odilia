//! A set of types dedicated to running multiple services.

use core::{
	future::Future,
	marker::PhantomData,
	pin::Pin,
	task::{Context, Poll},
};

use tower::{
	util::{Oneshot, ReadyOneshot},
	Service, ServiceExt,
};

/// Converts an [`Iterator`] over a set of (S, I) where `S` is a service that takes the input `I`
/// into an iterator over the future from [`ServiceExt::oneshot`].
pub struct MapServiceCall<Iter, S, I> {
	inner: Iter,
	_marker: PhantomData<(S, I)>,
}
impl<Iter, S, I> Iterator for MapServiceCall<Iter, S, I>
where
	Iter: Iterator<Item = (S, I)>,
	S: Service<I>,
{
	type Item = Oneshot<S, I>;
	fn next(&mut self) -> Option<Self::Item> {
		let (svc, input) = self.inner.next()?;
		Some(svc.oneshot(input))
	}
}

/// Converts an [`Iterator`] over a set of (S, I) where `S` is a service that takes the input `I`
/// into an iterator over the futures that yield the service once it is ready.
pub struct MapReady<Iter, S, I> {
	inner: Iter,
	_marker: PhantomData<(S, I)>,
}
impl<Iter, S, I> Iterator for MapReady<Iter, S, I>
where
	S: Service<I>,
	Iter: Iterator<Item = S>,
{
	type Item = ReadyOneshot<S, I>;
	fn next(&mut self) -> Option<Self::Item> {
		let s: S = self.inner.next()?;
		Some(s.ready_oneshot())
	}
}

impl<F, Res, E> ServiceCall<F, Res, E> {
	pub fn new<S, Req>(mut s: S, req: Req) -> Self
	where
		S: Service<Req, Future = F>,
	{
		ServiceCall { f: s.call(req), _marker: PhantomData }
	}
}

#[pin_project::pin_project]
pub struct ServiceCall<F, Res, E> {
	#[pin]
	f: F,
	_marker: PhantomData<Result<Res, E>>,
}
impl<F, Res, E> Future for ServiceCall<F, Res, E>
where
	F: Future<Output = Result<Res, E>>,
{
	type Output = Result<Res, E>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		self.project().f.poll(cx)
	}
}

/// Converts an iterator of `(Left,Right)` into an iterator of `(Right,Left)`.
pub struct ReverseTuple<Iter, Left, Right> {
	inner: Iter,
	_marker: PhantomData<(Left, Right)>,
}
impl<Iter, Left, Right> Iterator for ReverseTuple<Iter, Left, Right>
where
	Iter: Iterator<Item = (Left, Right)>,
{
	type Item = (Right, Left);
	fn next(&mut self) -> Option<Self::Item> {
		let (i1, i2) = self.inner.next()?;
		Some((i2, i1))
	}
}

pub trait MapMExt: Iterator + Sized {
	/// Maps an iterator over `(S, I)`, where `S` is a [`Service`] and `I` is the input to that
	/// service.
	/// Into an iterator over the results of calling each `S` with the given inputs.
	///
	/// ```
	/// use core::{
	///   convert::Infallible,
	///   iter::repeat_n,
	/// };
	/// use tower::service_fn;
	/// use tower::Service;
	/// use futures_lite::future::block_on;
	/// use tower_iter::MapMExt;
	///
	/// async fn mul_2(i: u32) -> Result<u32, Infallible> {
	///   Ok(i * 2)
	/// }
	/// let mut mul_svc = service_fn(mul_2);
	/// let mut iter = repeat_n(mul_svc, 5)
	///   .zip([
	///       5, 10, 15, 20, 25
	///   ].into_iter())
	///   .map_service_call();
	///
	/// assert_eq!(block_on(iter.next().unwrap()), Ok(10));
	/// assert_eq!(block_on(iter.next().unwrap()), Ok(20));
	/// assert_eq!(block_on(iter.next().unwrap()), Ok(30));
	/// assert_eq!(block_on(iter.next().unwrap()), Ok(40));
	/// assert_eq!(block_on(iter.next().unwrap()), Ok(50));
	/// assert!(iter.next().is_none());
	/// ```
	fn map_service_call<S, I>(self) -> MapServiceCall<Self, S, I> {
		MapServiceCall { inner: self, _marker: PhantomData }
	}
	/// Reverses a 2-tuple's order.
	/// ```
	/// use tower_iter::MapMExt;
	/// let iter = &mut [
	///   (0, 1),
	///   (2, 3),
	///   (4, 5),
	/// ].into_iter().reverse_tuple();
	///
	/// assert_eq!(iter.next(), Some((1, 0)));
	/// assert_eq!(iter.next(), Some((3, 2)));
	/// assert_eq!(iter.next(), Some((5, 4)));
	/// assert_eq!(iter.next(), None);
	/// ```
	fn reverse_tuple<I1, I2>(self) -> ReverseTuple<Self, I1, I2> {
		ReverseTuple { inner: self, _marker: PhantomData }
	}
}
impl<I> MapMExt for I where I: Iterator + Sized {}

pub struct MapM<Iter, S, I, O> {
	inner: Iter,
	_marker: PhantomData<fn(S, I) -> O>,
}

impl<Iter, S, I, O> Iterator for MapM<Iter, S, I, O>
where
	Iter: Iterator<Item = (S, I)>,
	S: Service<I, Response = O>,
{
	type Item = Oneshot<S, I>;
	fn next(&mut self) -> Option<Self::Item> {
		let (s, i) = self.inner.next()?;
		Some(s.oneshot(i))
	}
}
