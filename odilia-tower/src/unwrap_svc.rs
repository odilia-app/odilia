//! A service built specifically to run `.unwrap()` on computed futures.
//!
//! Why not just call `.await.unwrap()`?
//! Because when chaining together async functions on stable Rust, if the type is returned from a
//! trait, it must be named (see: unnamable types).
//!
//! This module also contains a few related "unwrap-then-..." cases that are useful in Odilia.

use core::{
	convert::Infallible,
	future::Future,
	marker::PhantomData,
	task::{Context, Poll},
};
use futures::{future::OkInto, TryFutureExt};
use odilia_common::command::TryIntoCommands;
use tower::Service;

/// Maps a response value of `Result<T, E>` to `[<T as TryIntoCommands>::into`]
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
		Poll::Ready(Ok(ready?))
	}
	fn call(&mut self, req: Req) -> Self::Future {
		self.inner.call(req).unwrap_fut().ok_try_into_command()
	}
}

pub struct MapResponseIntoService<S, Req, Res, R, E> {
	inner: S,
	_marker: PhantomData<(Res, Req, R, E)>,
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
		let Poll::Ready(_ready) = self.inner.poll_ready(cx) else {
			return Poll::Pending;
		};
		// allowed due to Error = Infallible
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: Req) -> Self::Future {
		self.inner.call(req).ok_into().flatten_fut_res()
	}
}

use core::pin::Pin;

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
		Poll::Ready(output.expect("An infallible future!"))
	}
}

/// A future which unwraps the future's [`Future::Output`] value if it is a [`Result<T,
/// Infallible>`] and converts it into [`T`].
///
/// This is useful in the context of [`tower`] where all services must return `Result<T, E>`, even
/// if `Err(E)` will never occur.
/// To ensure safety, this is only possible to use when the `E` parameter is
/// [`std::convert::Infallible`].
///
/// ```
/// # use futures::executor::block_on;
/// # use std::convert::Infallible;
/// # use odilia_tower::unwrap_svc::UnwrapFutExt;
/// async fn first_four_bits(x: u8) -> Result<u8, Infallible> {
///     Ok(x & 0xF)
/// }
/// assert_eq!(
///     block_on(first_four_bits(0xFA).unwrap_fut()),
///     0xA
/// );
/// ```
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
