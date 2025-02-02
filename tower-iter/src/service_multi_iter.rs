use crate::{FutureExt, MapMExt, MapOk, call_iter::MapServiceCall};
use alloc::vec::Vec;
use core::{
  future::{Future,IntoFuture},
  convert::Infallible,
  iter::Zip,
	mem::replace,
	task::{Context, Poll},
  marker::PhantomData,
};
use futures::future::{JoinAll, join_all};
use tower::Service;

/// Useful for running a set of services with the same signature in parallel.
///
/// Note that although calling the [`ServiceMultiIter::call`] function seems to return a
/// `Result<Vec<S::Response, S::Error>, S::Error>`, the outer error is gaurenteed never to be
/// an error.
///
/// Your three options for handling this are:
///
/// 1. Use [`Result::unwrap`] in the inner service.
/// 2. To use the [`crate::UnwrapService`] also provided by this crate. Or,
/// 3. Call [`collect::<Result<Vec<T>, E>>()`] on the result of the future.
#[derive(Clone)]
pub struct ServiceMultiIter<Si, Ii, S, I> {
  s_iter: Si,
  i_iter: Ii,
  _marker: PhantomData<(S, I)>,
}
impl<Si, Ii, S, I> ServiceMultiIter<Si,Ii,S,I> {
	fn new(s_iter: Si, i_iter: Ii) -> Self {
		ServiceMultiIter { s_iter, i_iter, _marker: PhantomData }
	}
}
/*

impl<Si, Ii, S, I> IntoFuture for ServiceMultiIter<Si,Ii,S,I> 
where S: Clone + Service<I>,
     Ii: Iterator<Item = I>,
     Si: Iterator<Item = S> {
    type Output = Vec<Result<S::Response, S::Error>>;
    type IntoFuture = JoinAll<>;
    fn into_future(self) -> Self::IntoFuture {
        join_all(
        self.s_iter.zip(self.i_iter)
            .map_service_call()
            )
    }
}
*/

