use std::{convert::Infallible, future::Future, sync::Arc};

use tower::{Layer, Service};

use crate::{
	tower::{
		async_try::{AsyncTryInto, AsyncTryIntoLayer, AsyncTryIntoService},
		iter_svc::IterService,
		state_svc::{StateLayer, StateService},
		sync_try::{TryIntoLayer, TryIntoService},
		unwrap_svc::{MapResponseIntoService, MapResponseTryIntoCommandsService},
	},
	TryIntoCommands,
};

pub trait ServiceExt<Request>: Service<Request> {
	fn request_try_from<I, R, Fut1, E>(self) -> TryIntoService<Request, I, Self, R, Fut1>
	where
		Self: Service<Request, Response = R, Future = Fut1> + Sized,
		I: TryInto<Request>,
		E: From<<I as TryInto<Request>>::Error>,
		Fut1: Future<Output = Result<R, E>>,
	{
		TryIntoLayer::new().layer(self)
	}
	fn request_async_try_from<I, R, Fut1, E, E2>(
		self,
	) -> AsyncTryIntoService<Request, I, Self, R, Fut1>
	where
		Self: Service<Request, Response = R, Future = Fut1, Error = E> + Clone + Sized,
		I: AsyncTryInto<Request, Error = E2>,
		E: From<E2>,
		Fut1: std::future::Future<Output = Result<R, E>>,
	{
		AsyncTryIntoLayer::new().layer(self)
	}
	fn with_state<S>(self, s: Arc<S>) -> StateService<Self, Arc<S>>
	where
		Self: Service<Request> + Clone + Sized,
		StateLayer<Arc<S>>: Layer<Self, Service = StateService<Self, Arc<S>>>,
	{
		StateLayer::new(s).layer(self)
	}

	fn map_response_into<Res, R, E>(self) -> MapResponseIntoService<Self, Request, Res, R, E>
	where
		Self: Service<Request, Response = Res, Error = Infallible> + Sized,
		Res: Into<Result<R, E>>,
	{
		MapResponseIntoService::new(self)
	}
	fn map_response_try_into_command<R>(
		self,
	) -> MapResponseTryIntoCommandsService<Self, Request>
	where
		Self: Service<Request, Error = Infallible, Response = R> + Sized,
		R: TryIntoCommands,
	{
		MapResponseTryIntoCommandsService::new(self)
	}
	//fn map_err_into<Req, E1, R, E, T>(self) -> MapErrIntoService<Self, Req, E1, R, E, T>
	//where
	//	Self: Service<Req, Response = R, Error = E1> + Sized,
	//	E: From<E1>,
	//{
	//	MapErrIntoService::new(self)
	//}
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
