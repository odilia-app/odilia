use crate::tower::Command;
use crate::tower::Handler;
use odilia_common::errors::OdiliaError;
use std::future::Future;

type Request = Command;
type Response = ();
type Error = OdiliaError;

impl<F, Fut, S, E> Handler<(Request,), S, E> for F
where
	F: FnOnce(E) -> Fut + Clone + Send,
	Fut: Future<Output = Result<Response, Error>> + Send + 'static,
	S: Clone,
{
	type Response = Response;
	type Future = Fut;
	fn call(self, req: E, _state: S) -> Self::Future {
		self(req)
	}
}

impl<F, Fut, S, E, T1> Handler<(Request, T1), S, E> for F
where
	F: FnOnce(E, T1) -> Fut + Clone + Send,
	Fut: Future<Output = Result<Response, Error>> + Send + 'static,
	S: Clone,
	T1: From<S>,
{
	type Response = Response;
	type Future = Fut;
	fn call(self, req: E, state: S) -> Self::Future {
		self(req, state.into())
	}
}
