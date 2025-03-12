//! Allow postfix notation for building services from existing ones.

use crate::{
	async_try::{AsyncTryInto, AsyncTryIntoLayer, AsyncTryIntoService},
	state_svc::{StateLayer, StateService},
	sync_try::{TryIntoLayer, TryIntoService},
};
use tower::{Layer, Service};

/// Use postfix notation on your [`tower::Service`]s to produce nested services.
pub trait ServiceExt<Request>: Service<Request> {
	/// Map a new input type into your service's input using a [`TryInto`] impl.
	///
	/// ```
	/// use assert_matches::assert_matches;
	/// use odilia_tower::service_ext::ServiceExt;
	/// use tower::{service_fn, Service};
	/// use futures::executor::block_on;
	/// use std::num::TryFromIntError;
	/// // NOTE: the associated error type must implement From<E>, where E is the error in converting
	/// // from the new input type (in this example, u64) to the inner one (u8).
	/// #[derive(Debug, PartialEq, Eq)]
	/// pub enum Error {
	///   IntConv(TryFromIntError)
	/// }
	/// impl From<TryFromIntError> for Error {
	///   fn from(ie: TryFromIntError) -> Error {
	///       Error::IntConv(ie)
	///   }
	/// }
	///
	/// async fn double(input: u8) -> Result<u8, Error> {
	///   Ok(input * 2)
	/// }
	/// let mut service = service_fn(double)
	///     // try to convert u64 to u8 before passing to the service function
	///     .request_try_from();
	/// assert_eq!(block_on(service.call(100u64)), Ok(200u8));
	/// // can not be successfully converted to u8!
	/// assert_matches!(block_on(service.call(300u64)), Err(_));
	/// ```
	fn request_try_from<I, R, Fut1>(self) -> TryIntoService<Request, I, Self, R, Fut1>
	where
		Self: Sized,
		I: TryInto<Request>,
		Self: Service<Request, Response = R, Future = Fut1>,
	{
		TryIntoLayer::new().layer(self)
	}
	/// Map a new input type into your service's input using a custom [`AsyncTryInto`] impl.
	/// This is mostly used in conjunction with state extraction, but doesn't inherently have to.
	///
	/// ```
	/// use assert_matches::assert_matches;
	/// use odilia_tower::{
	///   service_ext::ServiceExt,
	///   async_try::AsyncTryFrom,
	/// };
	/// use tower::{service_fn, Service};
	/// use futures::{
	///   executor::block_on,
	///   future::{ready, Ready},
	/// };
	/// use std::num::TryFromIntError;
	/// // Used to get around "foreign traits on foreign types" rule.
	/// #[derive(Debug, PartialEq, Eq)]
	/// struct U8(u8);
	/// impl From<u8> for U8 {
	///   fn from(inner: u8) -> U8 {
	///     U8(inner)
	///   }
	/// }
	/// #[derive(Debug, PartialEq, Eq)]
	/// struct U64(u64);
	/// impl From<u64> for U64 {
	///   fn from(inner: u64) -> U64 {
	///     U64(inner)
	///   }
	/// }
	///
	/// impl AsyncTryFrom<U64> for U8 {
	///   type Error = TryFromIntError;
	///   type Future = Ready<Result<U8, Self::Error>>;
	///   fn try_from_async(big: U64) -> Self::Future {
	///       ready(big.0.try_into().map(U8))
	///   }
	/// }
	/// // NOTE: the associated error type must implement From<E>, where E is the error in converting
	/// // from the new input type (in this example, u64) to the inner one (u8).
	/// #[derive(Debug, PartialEq, Eq)]
	/// pub enum Error {
	///   IntConv(TryFromIntError)
	/// }
	/// impl From<TryFromIntError> for Error {
	///   fn from(ie: TryFromIntError) -> Error {
	///       Error::IntConv(ie)
	///   }
	/// }
	///
	/// async fn double(input: U8) -> Result<U8, Error> {
	///   Ok((input.0 * 2).into())
	/// }
	/// let mut service = service_fn(double)
	///     // try to convert u64 to u8 before passing to the service function
	///     // this time, use the async function we described above
	///     .request_async_try_from::<U64, U8, _>();
	/// assert_eq!(block_on(service.call(100u64.into())), Ok(200u8.into()));
	/// // can not be successfully converted to u8!
	/// assert_matches!(block_on(service.call(300u64.into())), Err(_));
	/// ```
	fn request_async_try_from<I, R, Fut1>(
		self,
	) -> AsyncTryIntoService<Request, I, Self, R, Fut1>
	where
		I: AsyncTryInto<Request>,
		Self: Service<Request, Response = R, Future = Fut1> + Clone,
	{
		AsyncTryIntoLayer::new().layer(self)
	}
	/// Inject a clonable state into each invocation of the inner service.
	/// NOTE:
	///
	/// - since [`tower::Service`] only accepts functions with one parameter, this is passed as
	///   `(S, P)` where `S` is the state type and `P` is the parameter type.
	/// - The `S` parameter will be cloned upon each invocation to [`Service::call`]. It should be
	///   relatively cheap to clone.
	///
	/// ```
	/// use odilia_tower::service_ext::ServiceExt;
	/// use tower::{service_fn, Service};
	/// use futures::executor::block_on;
	/// use std::{convert::Infallible, sync::Arc};
	/// // a stand in for some comlpex type, don't actually do this.
	/// type State = Arc<usize>;
	///
	/// // NOTE: `tower` does not allow multiple parameters to services.
	/// async fn state_and_param((state, param): (Arc<usize>, u8)) -> Result<Vec<u8>, Infallible> {
	///     let mut vec = Vec::with_capacity(*state);
	///     for _ in 0..*state {
	///         vec.push(param);
	///     }
	///     Ok(vec)
	/// }
	/// let mut service = service_fn(state_and_param)
	///     .with_state(Arc::new(5));
	/// assert_eq!(block_on(service.call(2)), Ok(vec![2u8, 2, 2, 2, 2]));
	/// ```
	fn with_state<S>(self, s: S) -> StateService<Self, S>
	where
		Self: Sized,
		S: Clone,
	{
		StateLayer::new(s).layer(self)
	}
}

impl<T: ?Sized, Request> ServiceExt<Request> for T where T: Service<Request> {}
