use crate::tower::{
	async_try::{AsyncTryInto, AsyncTryIntoLayer, AsyncTryIntoService},
	iter_svc::IterService,
	state_svc::{StateLayer, StateService},
	sync_try::{TryIntoLayer, TryIntoService},
	unwrap_svc::{MapErrIntoService, MapResponseIntoService, UnwrapService},
};
use std::{convert::Infallible, sync::Arc};
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

	fn map_response_into<Res, R, E, T>(
		self,
	) -> MapResponseIntoService<Self, Request, Res, R, E, T>
	where
		Self: Service<Request, Response = Res, Error = Infallible> + Sized,
		Res: Into<Result<R, E>>,
	{
		MapResponseIntoService::new(self)
	}
	fn map_err_into<Req, E1, R, E, T>(self) -> MapErrIntoService<Self, Req, E1, R, E, T>
	where
		Self: Service<Req, Response = R, Error = E1> + Sized,
		E: From<E1>,
	{
		MapErrIntoService::new(self)
	}
	fn iter_into<S, Iter, I, E>(self, s: S) -> IterService<Self, Request, Iter, I, S, E>
	where
		Self: Service<Request, Response = Iter> + Clone + Sized,
		Iter: Iterator<Item = I>,
		S: Service<I> + ServiceExt<I> + Clone,
		E: From<<Self as Service<Request>>::Error> + From<S::Error>,
		//TODO erase:
		Request: Clone,
	{
		IterService::new(self, s)
	}
}

impl<T: ?Sized, Request> ServiceExt<Request> for T where T: Service<Request> {}
