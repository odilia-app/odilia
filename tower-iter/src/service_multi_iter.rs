use crate::{call_iter::MapServiceCall, MapMExt};
use alloc::vec::Vec;
use core::{future::IntoFuture, iter::Zip, marker::PhantomData};
use futures::future::{join_all, JoinAll};
use tower::Service;

/// Useful for running a set of services with the same signature in parallel.
///
/// Note that although calling the [`ServiceMultiIter::into_future`] function seems to return a
/// future that resolves to
/// `Result<Vec<Result<S::Response, S::Error>>, S::Error>`, the outer error is gaurenteed never to be
/// an error. It is [`std::convert::Infallible`].
///
/// Your three options for handling this are:
///
/// 1. Use [`Result::unwrap`] in the inner service.
/// 2. Call [`Iterator::collect::<Result<Vec<T>, E>>()`] on the result of the future.
#[derive(Clone)]
pub struct ServiceMultiIter<Si, Ii, S, I> {
	s_iter: Si,
	i_iter: Ii,
	_marker: PhantomData<(S, I)>,
}
impl<Si, Ii, S, I> ServiceMultiIter<Si, Ii, S, I> {
	pub fn new(s_iter: Si, i_iter: Ii) -> Self {
		ServiceMultiIter { s_iter, i_iter, _marker: PhantomData }
	}
}

impl<Si, Ii, S, I> IntoFuture for ServiceMultiIter<Si, Ii, S, I>
where
	S: Clone + Service<I>,
	Ii: Iterator<Item = I>,
	Si: Iterator<Item = S>,
{
	type Output = Vec<Result<S::Response, S::Error>>;
	type IntoFuture = JoinAll<<MapServiceCall<Zip<Si, Ii>, S, I> as Iterator>::Item>;
	fn into_future(self) -> Self::IntoFuture {
		join_all(self.s_iter.zip(self.i_iter).map_service_call())
	}
}
