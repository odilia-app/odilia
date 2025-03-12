//! A generic state provider service.
//! This clones the state upon every invocation of [`Service::call`], so make sure it's relatively cheap to do so.

use core::task::{Context, Poll};
use tower::{Layer, Service};

/// A [`tower::Layer`] which stores state `S`.
pub struct StateLayer<S> {
	state: S,
}
impl<S> StateLayer<S> {
	/// Create a new [`StateLayer`] with service of type `S`.
	pub fn new(state: S) -> Self {
		StateLayer { state }
	}
}

/// A service which clones state [`Sta`] into the [`Service::call`] method of the given service
/// [`Srv`].
pub struct StateService<Srv, Sta> {
	inner: Srv,
	state: Sta,
}
impl<Srv, Sta> Clone for StateService<Srv, Sta>
where
	Srv: Clone,
	Sta: Clone,
{
	fn clone(&self) -> Self {
		StateService { inner: self.inner.clone(), state: self.state.clone() }
	}
}

impl<I, Srv, Sta> Service<I> for StateService<Srv, Sta>
where
	Srv: Service<(Sta, I)>,
	Sta: Clone,
{
	type Error = Srv::Error;
	type Response = Srv::Response;
	type Future = Srv::Future;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}
	fn call(&mut self, input: I) -> Self::Future {
		self.inner.call((self.state.clone(), input))
	}
}

impl<Srv, Sta> Layer<Srv> for StateLayer<Sta>
where
	Sta: Clone,
{
	type Service = StateService<Srv, Sta>;
	fn layer(&self, inner: Srv) -> Self::Service {
		StateService { inner, state: self.state.clone() }
	}
}
