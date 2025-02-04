use crate::{call_iter::MapServiceCall, FutureExt, MapMExt, MapOk};
use alloc::vec::Vec;
use core::{
	iter::Zip,
	marker::PhantomData,
	mem::replace,
	task::{Context, Poll},
};
use futures::future::{join_all, JoinAll};
use tower::Service;

/// Useful for running a set of services with the same signature concurrently.
///
/// Note that although calling the [`ServiceMultiset::call`] function seems to return a
/// `Result<Vec<S::Response, S::Error>, S::Error>`, the outer error is gaurenteed never to be
/// an error.
///
/// Your two options for handling this are:
///
/// 1. Use [`Result::unwrap`] in the inner service.
/// 2. Call [`collect::<Result<Vec<T>, E>>()`] on the result of the future.
#[derive(Clone)]
pub struct ServiceMultiset<S, I, Si> {
	services: Vec<S>,
	_marker: PhantomData<(I, Si)>,
}
impl<S, I, Si> Default for ServiceMultiset<S, I, Si> {
	fn default() -> Self {
		ServiceMultiset { services: Vec::new(), _marker: PhantomData }
	}
}
impl<S, I, Si> ServiceMultiset<S, I, Si> {
	pub fn from(s: S) -> Self {
		ServiceMultiset { services: Vec::from([s]), _marker: PhantomData }
	}
	pub fn push(&mut self, svc: S) {
		self.services.push(svc);
	}
	pub fn clone_expand(&mut self, size: usize)
	where
		S: Clone,
	{
		// SAFETY: this will panic if we don't start with an initial service.
		// but there is no way to create a ServiceMultiset without an initial service.
		let i = self.services[0].clone();
		for _ in 0..size {
			self.services.push(i.clone());
		}
	}
}

impl<S, Req, I, Si> Service<I> for ServiceMultiset<S, I, Si>
where
	S: Service<Req> + Clone,
	I: Iterator<Item = Req>,
	Si: Iterator<Item = S>,
{
	type Response = Vec<Result<S::Response, S::Error>>;
	type Error = S::Error;
	type Future = MapOk<
		JoinAll<<MapServiceCall<Zip<Si, I>, S, Req> as Iterator>::Item>,
		Self::Error,
		Self::Response,
	>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		for svc in &mut self.services {
			let _ = svc.poll_ready(cx)?;
		}
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: I) -> Self::Future {
		let clone = self.services.clone();
		let services = replace(&mut self.services, clone);
		join_all(services.into_iter().zip(req).map_service_call()).wrap_ok()
	}
}
