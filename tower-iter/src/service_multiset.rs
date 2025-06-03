use core::{
	iter::Zip,
	marker::PhantomData,
	mem::replace,
	task::{Context, Poll},
};
use std::vec::Vec;

use futures_util::future::{join_all, JoinAll};
use tower::Service;

use crate::{call_iter::MapServiceCall, FutureExt, MapMExt, MapOk};

/// Useful for running a set of services with the same signature concurrently.
///
/// Note that although calling the [`ServiceMultiset::call`] function seems to return a
/// `Result<Vec<S::Response, S::Error>, S::Error>`, the outer error is gaurenteed never to be
/// an error.
///
/// Your two options for handling this are:
///
/// 1. Use [`Result::unwrap`] in the inner service.
/// 2. Call [`Iterator::collect::<Result<Vec<T>, E>>()`] on the result of the future.
///
/// ```
/// use core::{
///   convert::Infallible,
///   iter::repeat_n,
/// };
/// use tower::{service_fn, Service};
/// use futures_lite::future::block_on;
/// use tower_iter::service_multiset::ServiceMultiset;
///
/// async fn mul_2(i: u32) -> Result<u32, Infallible> {
///   Ok(i * 2)
/// }
/// let mut mul_svc = ServiceMultiset::from(service_fn(mul_2));
/// mul_svc.clone_expand(5);
/// let mut fut = mul_svc
///   .call([
///     5, 10, 15, 20, 25
///   ].into_iter());
///
/// assert_eq!(block_on(fut),
///     Ok(vec![
///         Ok(10),
///         Ok(20),
///         Ok(30),
///         Ok(40),
///         Ok(50)
///     ])
/// );
/// ```
#[derive(Clone)]
pub struct ServiceMultiset<S, I> {
	services: Vec<S>,
	_marker: PhantomData<I>,
}
impl<S, I> ServiceMultiset<S, I> {
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

impl<S, Req, I> Service<I> for ServiceMultiset<S, I>
where
	S: Service<Req> + Clone,
	I: Iterator<Item = Req>,
{
	type Response = Vec<Result<S::Response, S::Error>>;
	type Error = S::Error;
	type Future = MapOk<
		JoinAll<<MapServiceCall<Zip<<Vec<S> as IntoIterator>::IntoIter, I>, S, Req> as Iterator>::Item>,
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
