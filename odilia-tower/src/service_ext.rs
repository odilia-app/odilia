use crate::{
	async_try::{AsyncTryInto, AsyncTryIntoLayer, AsyncTryIntoService},
	state_svc::{StateLayer, StateService},
	sync_try::{TryIntoLayer, TryIntoService},
};
use tower::{Layer, Service};

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
	fn with_state<S>(self, s: S) -> StateService<Self, S>
	where
		Self: Sized,
		S: Clone,
	{
		StateLayer::new(s).layer(self)
	}
}

impl<T: ?Sized, Request> ServiceExt<Request> for T where T: Service<Request> {}
