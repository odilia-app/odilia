use crate::ScreenReaderState;
use std::{
	sync::Arc,
	task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct StateLayer<S> {
	state: Arc<S>,
}
impl<S> StateLayer<S> {
	pub fn new(state: Arc<S>) -> Self {
		StateLayer { state }
	}
}

pub struct StateService<Srv, Sta> {
	inner: Srv,
	state: Arc<Sta>,
}

impl<I, Srv, Sta> Service<I> for StateService<Srv, Sta>
where
	Srv: Service<(Arc<Sta>, I)>,
{
	type Error = Srv::Error;
	type Response = Srv::Response;
	type Future = Srv::Future;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}
	fn call(&mut self, input: I) -> Self::Future {
		self.inner.call((Arc::clone(&self.state), input))
	}
}

impl<Srv, Sta> Layer<Srv> for StateLayer<Sta> {
	type Service = StateService<Srv, Sta>;
	fn layer(&self, inner: Srv) -> Self::Service {
		StateService { inner, state: Arc::clone(&self.state) }
	}
}
