use alloc::vec::Vec;
use core::{
	future::Future,
	iter::Repeat,
	marker::PhantomData,
	pin::Pin,
	task::{Context, Poll},
};
use futures::future::{join_all, JoinAll};
use pin_project::pin_project;
use tower::Service;

use crate::{call_iter::FullServiceFut, service_multiset::ServiceMultiset};

#[pin_project]
pub struct MapFutureMultiSet<F, S, Lf, Lo> {
	#[pin]
	inner: F,
	_marker: PhantomData<fn(Lf) -> Lo>,
	_marker2: PhantomData<fn(S)>,
}
impl<F, S, Lf, Lo> Future for MapFutureMultiSet<F, S, Lf, Lo>
where
	F: Future<Output = (Lf, S)>,
	Lf: Iterator<Item = Lo>,
	S: Service<Lo> + Clone,
{
	type Output = MapOk<
		JoinAll<
			FullServiceFut<
				S,
				Lo,
				<S as Service<Lo>>::Response,
				<S as Service<Lo>>::Error,
				<S as Service<Lo>>::Future,
			>,
		>,
		<S as Service<Lo>>::Error,
		Vec<Result<<S as Service<Lo>>::Response, <S as Service<Lo>>::Error>>,
	>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.project();
		let Poll::Ready((iter, svc)) = this.inner.poll(cx) else {
			return Poll::Pending;
		};
		let mut msvc: ServiceMultiset<S, Lf, Repeat<S>> = ServiceMultiset::from(svc);
		Poll::Ready(msvc.call(iter))
	}
}

pub trait FutureExt<O, E>: Future<Output = O> {
	fn map_future_multiset<S, Lf, Lo>(self) -> MapFutureMultiSet<Self, S, Lf, Lo>
	where
		Self: Sized,
	{
		MapFutureMultiSet { inner: self, _marker: PhantomData, _marker2: PhantomData }
	}
	fn ok_join_all<Iter, I>(self) -> OkJoinAll<Self, E, O, Iter, I>
	where
		Self: Sized,
		I: Future,
	{
		OkJoinAll { f: self, res: None, _marker: PhantomData }
	}
	fn wrap_ok(self) -> MapOk<Self, E, O>
	where
		Self: Sized,
	{
		MapOk { f: self, _marker: PhantomData }
	}
}
#[pin_project]
pub struct OkJoinAll<F, E, O, Iter, I>
where
	I: Future,
{
	#[pin]
	f: F,
	#[pin]
	res: Option<JoinAll<I>>,
	_marker: PhantomData<(O, E, Iter, I)>,
}
impl<F, E, O, Iter, I> Future for OkJoinAll<F, E, O, Iter, I>
where
	F: Future<Output = Result<Iter, E>>,
	Iter: Iterator<Item = I>,
	I: Future<Output = O>,
{
	type Output = Result<Vec<O>, E>;
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let mut this = self.as_mut().project();
		if let Some(res) = this.res.as_mut().as_pin_mut() {
			res.poll(cx).map(Ok)
		} else {
			let res = match this.f.poll(cx) {
				Poll::Ready(Ok(o)) => o,
				Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
				Poll::Pending => return Poll::Pending,
			};
			let x = Some(join_all(res));
			*this.res = x;
			self.poll(cx)
		}
	}
}

#[pin_project]
pub struct MapOk<F, E, O> {
	#[pin]
	f: F,
	_marker: PhantomData<(O, E)>,
}
impl<F, E, O> Future for MapOk<F, E, O>
where
	F: Future<Output = O>,
{
	type Output = Result<O, E>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.project();
		match this.f.poll(cx) {
			Poll::Ready(o) => Poll::Ready(Ok(o)),
			Poll::Pending => Poll::Pending,
		}
	}
}

impl<F, O, E> FutureExt<O, E> for F where F: Future<Output = O> {}
