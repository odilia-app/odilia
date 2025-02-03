use core::future::ready;
use core::future::Future;
use core::future::Ready;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::future::join;
use futures::future::FutureExt;
use futures::future::TryFutureExt;
use futures::future::{Join, Then};
use tower::util::{Oneshot, ReadyOneshot};
use tower::Service;
use tower::ServiceExt as OtherServiceExt;

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
		let mut s: S = self.inner.next()?;
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

pub struct ReverseTuple<Iter, I1, I2> {
	inner: Iter,
	_marker: PhantomData<(I1, I2)>,
}
impl<Iter, I1, I2> Iterator for ReverseTuple<Iter, I2, I2>
where
	Iter: Iterator<Item = (I1, I2)>,
{
	type Item = (I2, I1);
	fn next(&mut self) -> Option<Self::Item> {
		let (i1, i2) = self.inner.next()?;
		Some((i2, i1))
	}
}

pub trait MapMExt: Iterator + Sized {
	fn map_service_call<S, I>(self) -> MapServiceCall<Self, S, I> {
		MapServiceCall { inner: self, _marker: PhantomData }
	}
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
		let (mut s, i) = self.inner.next()?;
		Some(s.oneshot(i))
	}
}
