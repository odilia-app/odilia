use core::task::{Context, Poll};
use tower::{Layer, Service};

pub struct StateLayer<S> {
	state: S,
}
impl<S> StateLayer<S> {
	pub fn new(state: S) -> Self {
		StateLayer { state }
	}
}

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
where Sta: Clone {
	type Service = StateService<Srv, Sta>;
	fn layer(&self, inner: Srv) -> Self::Service {
		StateService { inner, state: self.state.clone() }
	}
}
