use std::{
	future::Future,
	marker::PhantomData,
	task::{Context, Poll},
};
use tower::{Layer, Service};

pub trait AsyncTryFrom<T>: Sized + Send {
	type Error: Send;
	type Future: Future<Output = Result<Self, Self::Error>> + Send;

	fn try_from_async(value: T) -> Self::Future;
}
pub trait AsyncTryInto<T>: Sized + Send {
	type Error: Send;
	type Future: Future<Output = Result<T, Self::Error>> + Send;

	fn try_into_async(self) -> Self::Future;
}
impl<T: Send, U: AsyncTryFrom<T> + Send> AsyncTryInto<U> for T {
	type Error = U::Error;
	type Future = U::Future;
	fn try_into_async(self: T) -> Self::Future {
		U::try_from_async(self)
	}
}

pub struct AsyncTryIntoService<O, I: AsyncTryInto<O>, S, R, Fut1> {
	inner: S,
	_marker: PhantomData<fn(O, I, Fut1) -> R>,
}
impl<O, E, I: AsyncTryInto<O, Error = E>, S, R, Fut1> AsyncTryIntoService<O, I, S, R, Fut1> {
	pub fn new(inner: S) -> Self {
		AsyncTryIntoService { inner, _marker: PhantomData }
	}
}
#[derive(Clone)]
pub struct AsyncTryIntoLayer<O, I: AsyncTryInto<O>> {
	_marker: PhantomData<fn(I) -> O>,
}
impl<O, E, I: AsyncTryInto<O, Error = E>> AsyncTryIntoLayer<O, I> {
	pub fn new() -> Self {
		AsyncTryIntoLayer { _marker: PhantomData }
	}
}

impl<I: AsyncTryInto<O>, O, S, Fut1> Layer<S> for AsyncTryIntoLayer<O, I>
where
	S: Service<O, Future = Fut1>,
{
	type Service = AsyncTryIntoService<O, I, S, <S as Service<O>>::Response, Fut1>;
	fn layer(&self, inner: S) -> Self::Service {
		AsyncTryIntoService::new(inner)
	}
}

impl<O, E, I: AsyncTryInto<O>, S, R, Fut1> Service<I> for AsyncTryIntoService<O, I, S, R, Fut1>
where
	I: AsyncTryInto<O>,
	E: From<<I as AsyncTryInto<O>>::Error>,
	<I as AsyncTryInto<O>>::Error: Send,
	S: Service<O, Response = R, Future = Fut1> + Clone + Send,
	O: Send,
	Fut1: Future<Output = Result<R, E>> + Send,
{
	type Response = R;
	type Future = impl Future<Output = Result<R, E>> + Send;
	type Error = E;
	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: I) -> Self::Future {
		let clone = self.inner.clone();
		let mut inner = std::mem::replace(&mut self.inner, clone);
		async move {
			match req.try_into_async().await {
				Ok(resp) => inner.call(resp).await,
				Err(e) => Err(e.into()),
			}
		}
	}
}
