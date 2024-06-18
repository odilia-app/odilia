use crate::tower::{
	async_try::{AsyncTryInto, AsyncTryIntoLayer, AsyncTryIntoService},
	iter_svc::IterService,
	state_svc::{StateLayer, StateService},
	sync_try::{TryIntoLayer, TryIntoService},
	unwrap_svc::UnwrapService,
};
use std::{convert::Infallible, future::Future, sync::Arc};
use tower::{util::MapResult, Layer, Service, ServiceExt as TowerServiceExt};

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
	fn with_state<S>(self, s: Arc<S>) -> StateService<Self, S>
	where
		Self: Sized,
	{
		StateLayer::new(s).layer(self)
	}
	fn unwrap_map<R, E, F>(self, f: F) -> UnwrapService<Self, Request, Self::Response, R, E, F>
	where
		Self: Service<Request, Error = Infallible> + Sized,
		F: FnOnce(<Self as Service<Request>>::Response) -> Result<R, E>,
	{
		UnwrapService::new(self, f)
	}
}

impl<T: ?Sized, Request> ServiceExt<Request> for T where T: Service<Request> {}
