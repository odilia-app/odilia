use odilia_common::errors::OdiliaError;
use std::future::Future;
use std::marker::PhantomData;

use std::task::Context;
use std::task::Poll;

use tower::Service;

type Response = ();
type Request = atspi::Event;
type Error = OdiliaError;

pub trait Handler<T, S, E>: Clone {
	type Future: Future<Output = Result<Response, Error>> + Send + 'static;
	fn with_state(self, state: S) -> HandlerService<Self, T, S, E> {
		HandlerService { handler: self, state, _marker: PhantomData }
	}
	fn call(self, req: E, state: S) -> Self::Future;
}

impl<F, Fut, S, E> Handler<((),), S, E> for F
where
	F: FnOnce() -> Fut + Clone + Send + 'static,
	Fut: Future<Output = Result<Response, Error>> + Send + 'static,
	S: Clone,
{
	type Future = Fut;
	fn call(self, _req: E, _state: S) -> Self::Future {
		self()
	}
}

impl<F, Fut, S, E> Handler<(Request,), S, E> for F
where
	F: FnOnce(E) -> Fut + Clone + Send + 'static,
	Fut: Future<Output = Result<Response, Error>> + Send + 'static,
	S: Clone,
{
	type Future = Fut;
	fn call(self, req: E, _state: S) -> Self::Future {
		self(req)
	}
}
impl<F, Fut, S, T1, E> Handler<(Request, T1), S, E> for F
where
	F: FnOnce(E, T1) -> Fut + Clone + Send + 'static,
	Fut: Future<Output = Result<Response, Error>> + Send + 'static,
	S: Clone,
	T1: From<S>,
{
	type Future = Fut;
	fn call(self, req: E, state: S) -> Self::Future {
		self(req, (state.clone()).into())
	}
}
// breakdown of the various type parameters:
//
// F: is a function that takes E, T1, T2 (all of which are generic)
// T1, T2: are type which can be created infallibly from a reference to S, which is generic
// S: is some state type that implements Clone
// Fut: is a future whoes output is Result<Response, Error>, and which is sendable across threads
// statically
impl<F, Fut, S, T1, T2, E> Handler<(Request, T1, T2), S, E> for F
where
	F: FnOnce(E, T1, T2) -> Fut + Clone + Send + 'static,
	Fut: Future<Output = Result<Response, Error>> + Send + 'static,
	S: Clone,
	T1: From<S>,
	T2: From<S>,
{
	type Future = Fut;
	fn call(self, req: E, state: S) -> Self::Future {
		self(req, state.clone().into(), state.clone().into())
	}
}

pub struct HandlerService<H, T, S, E> {
	handler: H,
	state: S,
	_marker: PhantomData<fn(E) -> T>,
}

impl<H, T, S, E> Service<Request> for HandlerService<H, T, S, E>
where
	H: Handler<T, S, E>,
	S: Clone,
	E: TryFrom<Request>,
	<E as TryFrom<Request>>::Error: std::fmt::Debug,
{
	type Response = Response;
	type Future = <H as Handler<T, S, E>>::Future;
	type Error = Error;

	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: Request) -> Self::Future {
		let handler = self.handler.clone();
		let state = self.state.clone();
		handler.call(req.try_into().expect("Must be converted from a certain type"), state)
	}
}