use crate::{FutureExt, MapMExt, MapOk, call_iter::MapServiceCall, service_multiset::ServiceMultiset};
use alloc::vec::Vec;
use core::{
	iter::{Zip, repeat, Repeat},
	mem::replace,
	task::{Context, Poll},
};
use futures::future::{JoinAll, join_all};
use tower::Service;

/// Useful for running a set of services with the same signature in parallel.
///
/// Note that although calling the [`ServiceSet::call`] function seems to return a
/// `Result<Vec<S::Response, S::Error>, S::Error>`, the outer error is gaurenteed never to be
/// an error.
///
/// Your three options for handling this are:
///
/// 1. Use [`Result::unwrap`] in the inner service.
/// 2. To use the [`crate::UnwrapService`] also provided by this crate. Or,
/// 3. Call [`collect::<Result<Vec<T>, E>>()`] on the result of the future.
#[derive(Clone)]
pub struct ServiceSet<S, I, Si> {
	inner: ServiceMultiset<S, I, Si>,
}
impl<S, I, Si> Default for ServiceSet<S, I, Si> {
	fn default() -> Self {
    ServiceSet { inner: ServiceMultiset::default() }
	}
}
impl<S, I, Si> ServiceSet<S, I, Si> {
  pub fn from(s: S) -> ServiceSet<S, I, Si> {
      ServiceSet { inner: ServiceMultiset::from(s) }
  }
	pub fn push(&mut self, svc: S) {
		self.inner.push(svc);
	}
  pub fn clone_expand(&mut self, size: usize) 
  where S: Clone {
      self.inner.clone_expand(size);
  }
}

impl<S, Si, Req> Service<Req> for ServiceSet<S, Repeat<Req>, Si>
where
	S: Service<Req> + Clone,
  Req: Clone,
  Si: Iterator<Item = S>,
{
	type Response = Vec<Result<S::Response, S::Error>>;
	type Error = S::Error;
	type Future = MapOk<JoinAll<<MapServiceCall<Zip<Si, Repeat<Req>>, S, Req> as Iterator>::Item>, Self::Error, Self::Response>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
      self.inner.poll_ready(cx)
	}
	fn call(&mut self, req: Req) -> Self::Future {
      self.inner.call(repeat(req))
	}
}
