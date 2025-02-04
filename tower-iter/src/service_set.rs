use crate::{call_iter::MapServiceCall, FutureExt, MapMExt, MapOk};
use alloc::vec::Vec;
use core::{
	iter::{repeat, Repeat, Zip},
	mem::replace,
	task::{Context, Poll},
};
use futures::future::{join_all, JoinAll};
use tower::Service;

/// Useful for running a set of services with the same signature concurrently.
/// [`ServiceSet::call`] clones the argument to all the contained services.
///
/// Note that although calling the [`ServiceSet::call`] function seems to return a
/// `Result<Vec<S::Response, S::Error>, S::Error>`, the outer error is gaurenteed never to be
/// an error.
///
/// Your two options for handling this are:
///
/// 1. Use [`Result::unwrap`] in the inner service.
/// 2. Call [`collect::<Result<Vec<T>, E>>()`] on the result of the future.
#[derive(Clone)]
pub struct ServiceSet<S> {
	inner: Vec<S>,
}
impl<S> Default for ServiceSet<S> {
	fn default() -> Self {
		ServiceSet { inner: Vec::new() }
	}
}
impl<S> ServiceSet<S> {
	pub fn from(s: S) -> ServiceSet<S> {
		ServiceSet { inner: Vec::from([s]) }
	}
	pub fn push(&mut self, svc: S) {
		self.inner.push(svc);
	}
}

impl<S, Req> Service<Req> for ServiceSet<S>
where
	S: Service<Req> + Clone,
	Req: Clone,
{
	type Response = Vec<Result<S::Response, S::Error>>;
	type Error = S::Error;
	type Future = MapOk<
		JoinAll<
			<MapServiceCall<
				Zip<<Vec<S> as IntoIterator>::IntoIter, Repeat<Req>>,
				S,
				Req,
			> as Iterator>::Item,
		>,
		Self::Error,
		Self::Response,
	>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		for svc in &mut self.inner {
			let _ = svc.poll_ready(cx);
		}
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: Req) -> Self::Future {
		let clone = self.inner.clone();
		let inner = replace(&mut self.inner, clone);
		join_all(inner.into_iter().zip(repeat(req)).map_service_call()).wrap_ok()
	}
}
