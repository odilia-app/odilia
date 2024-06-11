use crate::ScreenReaderState;
use std::{
	sync::Arc,
	task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct StateLayer {
	state: Arc<ScreenReaderState>,
}

pub struct StateService<S> {
	inner: S,
	state: Arc<ScreenReaderState>,
}

impl<I, S> Service<I> for StateService<S>
where
	S: Service<(Arc<ScreenReaderState>, I)>,
{
	type Error = S::Error;
	type Response = S::Response;
	type Future = S::Future;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}
	fn call(&mut self, input: I) -> Self::Future {
		self.inner.call((Arc::clone(&self.state), input))
	}
}

impl<S> Layer<S> for StateLayer {
	type Service = StateService<S>;
	fn layer(&self, inner: S) -> Self::Service {
		StateService { inner, state: Arc::clone(&self.state) }
	}
}
