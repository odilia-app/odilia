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
	/// Wrap an inner service.
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

/// Map the `Ok` variant of [`Service::call`] into a new type.
/// Where the new type `R` implements `From<Res>`.
pub struct MapResponseIntoService<S, Req, Res, R, E> {
	inner: S,
	_marker: PhantomData<(Res, Req, R, E)>,
}
impl<S, Req, Res, R, E> MapResponseIntoService<S, Req, Res, R, E>
where
	S: Service<Req, Error = Infallible>,
	S::Response: Into<Result<R, E>>,
{
	/// Wrap an inner service.
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

/// Map a future result into return of [`TryIntoCommands::try_into_commands`].
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

/// A future which flattens a future's nested results when the outer result in
/// [`std::convert::Infallible`].
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
/// Infallible>`] and converts it into `T`.
///
/// This is useful in the context of [`tower`] where all services must return `Result<T, E>`, even
/// if `Err(E)` will never occur.
/// To ensure safety, this is only possible to use when the `E` parameter is
/// [`std::convert::Infallible`].
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

/// Add postfix notation for [`TryIntoCommandFut`].
pub trait UnwrapFutExt: Future {
	///
	/// ```
	/// use futures::executor::block_on;
	/// use std::convert::Infallible;
	/// use odilia_tower::unwrap_svc::UnwrapFutExt;
	/// async fn first_four_bits(x: u8) -> Result<u8, Infallible> {
	///     Ok(x & 0xF)
	/// }
	/// assert_eq!(
	///     block_on(first_four_bits(0xFA).unwrap_fut()),
	///     0xA
	/// );
	/// ```
	fn unwrap_fut<O, E>(self) -> UnwrapFut<Self, O, E>
	where
		Self: Sized,
	{
		UnwrapFut { fut: self, _marker: PhantomData }
	}
	/// ```
	/// use futures::executor::block_on;
	/// use futures::future::TryFutureExt;
	/// use odilia_tower::unwrap_svc::UnwrapFutExt;
	/// use std::convert::Infallible;
	/// #[derive(Debug, PartialEq)]
	/// pub struct Error;
	/// async fn inner(x: u8) -> Result<u8, Infallible> {
	///   Ok(x+2)
	/// }
	/// fn outer(x: u8) -> Result<u8, Error> {
	///     Ok(x-3)
	/// }
	/// let fut = inner(10)
	///     .map_ok(outer)
	///     .flatten_fut_res();
	/// // Note Ok(8) instead of Ok(Ok(9))!
	/// assert_eq!(block_on(fut), Ok(9));
	/// ```
	fn flatten_fut_res<O, E1>(self) -> FlattenFutResult<Self, O, E1>
	where
		Self: Sized,
	{
		FlattenFutResult { fut: self, _marker: PhantomData }
	}
	/// Map's a future into it's corresponding [`TryIntoCommands::try_into_commands`] output.
	/// This type is only for being able to name it.
	/// The same effect can be achieved with [`futures::future::FutureExt::map`] if you do not need to name the type.
	///
	/// ```
	/// use ssip::Priority;
	/// use odilia_common::command::{OdiliaCommand, Speak, TryIntoCommands};
	/// use futures::executor::block_on;
	/// use futures::future::TryFutureExt;
	/// use odilia_tower::unwrap_svc::{UnwrapFutExt, TryIntoCommandFut};
	/// use std::convert::Infallible;
	/// async fn commands() -> (Priority, &'static str) {
	///   (Priority::Text, "This should convert into a speak command!")
	/// }
	/// let fut = commands()
	///     .ok_try_into_command::<_, Infallible>();
	/// let mut iter = block_on(fut).expect("Conversion success");
	/// assert_eq!(iter.next().expect("First item"),
	///     OdiliaCommand::Speak(Speak(
	///         "This should convert into a speak command!".to_string(),
	///         Priority::Text,
	///     ))
	/// );
	/// ```
	fn ok_try_into_command<O, E>(self) -> TryIntoCommandFut<Self, O, E>
	where
		Self: Sized,
	{
		TryIntoCommandFut { f: self, _marker: PhantomData }
	}
}
impl<F> UnwrapFutExt for F where F: Future {}
