use core::{
	future::{ready, Future, IntoFuture, Ready},
	iter::{repeat, Repeat},
	marker::PhantomData,
	pin::Pin,
	task::{Context, Poll},
};
use std::vec::Vec;

use futures_util::future::{join_all, Either, FutureExt as OtherFutExt, JoinAll};
use pin_project_lite::pin_project;
use tower::{util::Oneshot, Service};

use crate::service_multi_iter::ServiceMultiIter;

pin_project! {
    pub struct MapFutureMultiSet<F, S, Lf, Lo, E> {
      #[pin]
      inner: F,
      svc: S,
      _marker: PhantomData<fn(Lf) -> Lo>,
      _marker2: PhantomData<fn(S) -> E>,
    }
}
impl<F, S, Lf, Lo, E> Future for MapFutureMultiSet<F, S, Lf, Lo, E>
where
	F: Future<Output = Result<Lf, E>>,
	Lf: Iterator<Item = Lo>,
	S: Service<Lo> + Clone,
{
	type Output = Either<
		Ready<Result<Vec<Result<S::Response, S::Error>>, E>>,
		MapOk<JoinAll<Oneshot<S, Lo>>, E, Vec<Result<S::Response, S::Error>>>,
	>;
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.as_mut().project();
		let Poll::Ready(maybe_iter) = this.inner.poll(cx) else {
			return Poll::Pending;
		};
		match maybe_iter {
			Err(e) => Poll::Ready(Either::Left(ready(Err(e)))),
			Ok(iter) => {
				let msvc: ServiceMultiIter<Repeat<S>, Lf, S, Lo> =
					ServiceMultiIter::new(repeat(self.svc.clone()), iter);
				Poll::Ready(msvc.into_future().wrap_ok().right_future())
			}
		}
	}
}

pub trait FutureExt<O, E>: Future<Output = O> {
	fn map_future_multiset<S, Lf, Lo, E2>(
		self,
		svc: S,
	) -> MapFutureMultiSet<Self, S, Lf, Lo, E2>
	where
		Self: Sized,
	{
		MapFutureMultiSet { inner: self, svc, _marker: PhantomData, _marker2: PhantomData }
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
pin_project! {
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

pin_project! {
    pub struct MapOk<F, E, O> {
      #[pin]
      f: F,
      _marker: PhantomData<(O, E)>,
    }
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
