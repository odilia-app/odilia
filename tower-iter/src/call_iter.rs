use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};
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
		let (s, i) = self.inner.next()?;
		Some(s.oneshot(i))
	}
}
