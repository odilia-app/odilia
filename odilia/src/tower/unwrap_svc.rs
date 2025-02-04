use crate::TryIntoCommands;
use futures::{
	future::{ErrInto, OkInto},
	FutureExt, TryFutureExt,
};
use std::convert::Infallible;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tower::Service;

#[allow(clippy::type_complexity)]
pub struct MapErrIntoService<S, Req, E1, R, E, T> {
	inner: S,
	_marker: PhantomData<fn(Req, E1, T) -> Result<R, E>>,
}
impl<S, Req, E1, R, E, T> MapErrIntoService<S, Req, E1, R, E, T>
where
	S: Service<Req, Response = R, Error = E1>,
	E: From<E1>,
{
	pub fn new(inner: S) -> Self {
		MapErrIntoService { inner, _marker: PhantomData }
	}
}
impl<S, Req, Res, R, E, T> Clone for MapErrIntoService<S, Req, Res, R, E, T>
where
	S: Clone,
{
	fn clone(&self) -> Self {
		MapErrIntoService { inner: self.inner.clone(), _marker: PhantomData }
	}
}

impl<S, Req, E1, R, E, T> Service<Req> for MapErrIntoService<S, Req, E1, R, E, T>
where
	S: Service<Req, Response = R, Error = E1>,
	E: From<E1>,
{
	type Error = E;
	type Response = R;
	type Future = ErrInto<S::Future, E>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx).map_err(Into::into)
	}
	fn call(&mut self, req: Req) -> Self::Future {
		self.inner.call(req).err_into()
	}
}

#[allow(clippy::type_complexity)]
pub struct MapResponseTryIntoCommandsService<S, Req> {
	inner: S,
	_marker: PhantomData<Req>,
}
impl<S, Req> MapResponseTryIntoCommandsService<S, Req>
where
	S: Service<Req, Error = Infallible>,
	S::Response: TryIntoCommands,
{
	pub fn new(inner: S) -> Self {
		MapResponseTryIntoCommandsService { inner, _marker: PhantomData }
	}
}
impl<S, Req> Clone for MapResponseTryIntoCommandsService<S, Req>
where
	S: Clone,
{
	fn clone(&self) -> Self {
		MapResponseTryIntoCommandsService {
			inner: self.inner.clone(),
			_marker: PhantomData,
		}
	}
}

impl<S, Req> Service<Req> for MapResponseTryIntoCommandsService<S, Req>
where
	S: Service<Req, Error = Infallible>,
	S::Response: TryIntoCommands,
{
	type Error = crate::OdiliaError;
	type Response = <S::Response as TryIntoCommands>::Iter;
	type Future = TryIntoCommandFut<
		UnwrapFut<S::Future, S::Response, Infallible>,
		S::Response,
		S::Error,
	>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		let Poll::Ready(ready) = self.inner.poll_ready(cx) else {
			return Poll::Pending;
		};
		Poll::Ready(Ok(ready.expect("An infallible poll_ready!")))
	}
	fn call(&mut self, req: Req) -> Self::Future {
		self.inner.call(req).unwrap_fut().ok_try_into_command()
	}
}

#[allow(clippy::type_complexity)]
pub struct MapResponseIntoService<S, Req, Res, R, E> {
	inner: S,
	_marker: PhantomData<fn(Req, Res) -> Result<R, E>>,
}
impl<S, Req, Res, R, E> MapResponseIntoService<S, Req, Res, R, E>
where
	S: Service<Req, Error = Infallible>,
	S::Response: Into<Result<R, E>>,
{
	pub fn new(inner: S) -> Self {
		MapResponseIntoService { inner, _marker: PhantomData }
	}
}
impl<S, Req, Res, R, E> Clone for MapResponseIntoService<S, Req, Res, R, E>
where
	S: Clone,
{
	fn clone(&self) -> Self {
		MapResponseIntoService { inner: self.inner.clone(), _marker: PhantomData }
	}
}

impl<S, Req, Res, R, E> Service<Req> for MapResponseIntoService<S, Req, Res, R, E>
where
	S: Service<Req, Error = Infallible>,
	S::Response: Into<Result<R, E>>,
{
	type Error = E;
	type Response = R;
	type Future = FlattenFutResult<OkInto<S::Future, Result<R, E>>, R, E>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		let Poll::Ready(ready) = self.inner.poll_ready(cx) else {
			return Poll::Pending;
		};
		Poll::Ready(Ok(ready.expect("An infallible poll_ready!")))
	}
	fn call(&mut self, req: Req) -> Self::Future {
		self.inner.call(req).ok_into().flatten_fut_res()
	}
}

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
		self.inner.call(req).unwrap_fut().map(self.f.clone())
	}
}

use std::pin::Pin;

#[pin_project::pin_project]
pub struct TryIntoCommandFut<F, Ic, E> {
	#[pin]
	f: F,
	_marker: PhantomData<(Ic, E)>,
}
impl<F, Ic, E> Future for TryIntoCommandFut<F, Ic, E>
where
	F: Future<Output = Ic>,
	Ic: TryIntoCommands,
{
	type Output = Result<Ic::Iter, crate::OdiliaError>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.project();
		match this.f.poll(cx) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(ic) => Poll::Ready(ic.try_into_commands()),
		}
	}
}

#[pin_project::pin_project]
pub struct FlattenFutResult<F, O, E1> {
	#[pin]
	fut: F,
	_marker: PhantomData<(O, E1)>,
}
impl<F, O, E1> Future for FlattenFutResult<F, O, E1>
where
	F: Future<Output = Result<Result<O, E1>, Infallible>>,
{
	type Output = Result<O, E1>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.project();
		let Poll::Ready(output) = this.fut.poll(cx) else {
			return Poll::Pending;
		};
		let mapped = match output {
            Ok(Ok(o)) => Ok(o),
            Ok(Err(e)) => Err(e),
            Err(_) => panic!("Not possible to construct this future unless the error value in Infallible"),
        };
		Poll::Ready(mapped)
	}
}

#[pin_project::pin_project]
pub struct UnwrapFut<F, O, E> {
	#[pin]
	fut: F,
	_marker: PhantomData<(O, E)>,
}
impl<F, O, Infallible> Future for UnwrapFut<F, O, Infallible>
where
	F: Future<Output = Result<O, Infallible>>,
{
	type Output = O;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		let this = self.project();
		match this.fut.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(o)) => Poll::Ready(o),
            Poll::Ready(Err(_)) => panic!("This future may only be called with futures whose error type is Infallible"),
        }
	}
}

trait UnwrapFutExt: Future {
	fn unwrap_fut<O, E>(self) -> UnwrapFut<Self, O, E>
	where
		Self: Sized,
	{
		UnwrapFut { fut: self, _marker: PhantomData }
	}
	fn flatten_fut_res<O, E1>(self) -> FlattenFutResult<Self, O, E1>
	where
		Self: Sized,
	{
		FlattenFutResult { fut: self, _marker: PhantomData }
	}
	fn ok_try_into_command<O, E>(self) -> TryIntoCommandFut<Self, O, E>
	where
		Self: Sized,
	{
		TryIntoCommandFut { f: self, _marker: PhantomData }
	}
}
impl<F> UnwrapFutExt for F where F: Future {}
