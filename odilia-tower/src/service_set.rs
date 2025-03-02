use core::{
	future::Future,
	iter::repeat_n,
	marker::PhantomData,
	mem::replace,
	pin::Pin,
	task::{Context, Poll},
};
use futures::future::{join_all, JoinAll};
use pin_project::pin_project;
use tower::Service;

/// Useful for running a set of services with the same signature in parallel.
///
/// Note that although calling the [`ServiceSet::call`] function seems to return a
/// `Result<Vec<S::Response, S::Error>, S::Error>`, the outer error is gaurenteed never to be
/// returned and can safely be unwrapped _from the caller function_.
///
/// Or feel free to use the [`crate::UnwrapService`] also provided by this crate.
#[derive(Clone)]
pub struct ServiceSet<S> {
	services: Vec<S>,
}
impl<S> Default for ServiceSet<S> {
	fn default() -> Self {
		ServiceSet { services: vec![] }
	}
}
impl<S> ServiceSet<S> {
	pub fn push(&mut self, svc: S) {
		self.services.push(svc);
	}
}

impl<S, Req> Service<Req> for ServiceSet<S>
where
	S: Service<Req> + Clone,
	Req: Clone,
{
	type Response = Vec<Result<S::Response, S::Error>>;
	type Error = S::Error;
	type Future = MapOk<JoinAll<<S as Service<Req>>::Future>, Self::Error, Self::Response>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		for svc in &mut self.services {
			let _ = svc.poll_ready(cx)?;
		}
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: Req) -> Self::Future {
		let clone = self.services.clone();
		let services = replace(&mut self.services, clone);
		let req_rep = repeat_n(req, services.len());
		services.into_iter().zip(req_rep).call2().join_all().map_ok()
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

trait MapOkExt<O, E>: Future<Output = O> {
	fn map_ok(self) -> MapOk<Self, E, O>
	where
		Self: Sized,
	{
		MapOk { f: self, _marker: PhantomData }
	}
}
impl<F, O, E> MapOkExt<O, E> for F where F: Future<Output = O> {}

trait Call2Ext: Iterator + Sized {
	fn call2<S, I, O>(self) -> Call2<Self, S, I, O> {
		Call2 { inner: self, _marker: PhantomData }
	}
	fn join_all(self) -> JoinAll<Self::Item>
	where
		Self::Item: Future,
	{
		join_all(self)
	}
}
impl<I> Call2Ext for I where I: Iterator + Sized {}

pub struct Call2<Iter, S, I, O> {
	inner: Iter,
	_marker: PhantomData<fn(S, I) -> O>,
}

impl<Iter, S, I, O> Iterator for Call2<Iter, S, I, O>
where
	Iter: Iterator<Item = (S, I)>,
	S: Service<I, Response = O>,
{
	type Item = S::Future;
	fn next(&mut self) -> Option<Self::Item> {
		let (mut s, i) = self.inner.next()?;
		Some(s.call(i))
	}
}
