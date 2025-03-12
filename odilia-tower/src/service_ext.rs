//! Allow postfix notation for building services from existing ones.

use crate::{
	async_try::{AsyncTryInto, AsyncTryIntoLayer, AsyncTryIntoService},
	state_svc::{StateLayer, StateService},
	sync_try::{TryIntoLayer, TryIntoService},
};
use tower::{Layer, Service};

/// Use postfix notation on your [`tower::Service`]s to produce nested services.
pub trait ServiceExt<Request>: Service<Request> {
	fn request_try_from<I, R, Fut1>(self) -> TryIntoService<Request, I, Self, R, Fut1>
	where
		Self: Sized,
		I: TryInto<Request>,
		Self: Service<Request, Response = R, Future = Fut1>,
	{
		TryIntoLayer::new().layer(self)
	}
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
	/// `(S, P)` where `S` is the state type and `P` is the parameter type.
	/// - The `S` parameter will be cloned upon each invocation to [`Service::call`]. It should be
	/// relatively cheap to clone.
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
