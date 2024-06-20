use futures::FutureExt;
use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tower::Service;

#[allow(clippy::type_complexity)]
pub struct UnwrapService<S, Req, Res, R, E, F> {
	inner: S,
	f: F,
	_marker: PhantomData<fn(F, Req, Res) -> Result<R, E>>,
}
impl<S, Req, Res, R, E, F> UnwrapService<S, Req, Res, R, E, F>
where
	S: Service<Req, Response = Res, Error = Infallible>,
{
	pub fn new(inner: S, f: F) -> Self {
		UnwrapService { inner, f, _marker: PhantomData }
	}
}
impl<S, Req, Res, R, E, F> Clone for UnwrapService<S, Req, Res, R, E, F>
where
	S: Clone,
	F: Clone,
{
	fn clone(&self) -> Self {
		UnwrapService { inner: self.inner.clone(), f: self.f.clone(), _marker: PhantomData }
	}
}

impl<S, Req, Res, R, E, F> Service<Req> for UnwrapService<S, Req, Res, R, E, F>
where
	S: Service<Req, Response = Res, Error = Infallible>,
	E: From<Infallible>,
	F: FnOnce(S::Response) -> Result<R, E> + Clone,
{
	type Error = E;
	type Response = R;
	type Future = impl Future<Output = Result<R, E>>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx).map_err(Into::into)
	}
	fn call(&mut self, req: Req) -> Self::Future {
		self.inner.call(req).map(|res| res.unwrap()).map(self.f.clone())
	}
}
