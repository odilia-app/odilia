#![allow(clippy::module_name_repetitions)]

use crate::tower::{
    async_try::AsyncTryInto,
};
use atspi::Event;
use std::{
	future::Future,
	marker::PhantomData,
	task::{Context, Poll},
  convert::Infallible,
};
use futures::{
    FutureExt,
    join,
    future::Map,
};
use tower::Service;

type Request = Event;

pub trait Handler<T, S: Clone, E>: Clone {
	type Response;
	type Future: Future<Output = Self::Response> + Send + 'static;
	fn with_state_and_fn<R, Er, F>(
		self,
		state: S,
		f: F,
	) -> HandlerService<Self, T, S, E, R, Er, F>
	where
		F: FnOnce(Self::Response) -> Result<R, Er>,
	{
		HandlerService::new(self, state, f)
	}
	fn call(self, req: E, state: S) -> Self::Future;
}

impl<F, Fut, S, E, R> Handler<((),), S, E> for F
where
	F: FnOnce() -> Fut + Clone + Send,
	Fut: Future<Output = R> + Send + 'static,
	S: Clone,
{
	type Response = R;
	type Future = Fut;
	fn call(self, _req: E, _state: S) -> Self::Future {
		self()
	}
}

impl<F, Fut, S, E, R> Handler<(Request,), S, E> for F
where
	F: FnOnce(E) -> Fut + Clone + Send,
	Fut: Future<Output = R> + Send + 'static,
	S: Clone,
{
	type Response = R;
	type Future = Fut;
	fn call(self, req: E, _state: S) -> Self::Future {
		self(req)
	}
}

macro_rules! impl_handler {
    ($(($type:ident,$err:ident),)+) => {
        #[allow(non_snake_case)]
        impl<F, Fut, S, E, R, $($type,$err,)+> Handler<(Request, $($type,)+), S, E> for F
        where
            F: FnOnce(E, $($type,)+) -> Fut + Clone + Send + 'static,
            Fut: Future<Output = R> + Send + 'static,
            S: Clone + $(AsyncTryInto<$type, Error = $err>+)+ 'static + Sync,
            $($type: From<S> + Send + 'static,)+
            $($err: Send + 'static,)+
            R: 'static + $(std::ops::FromResidual<Result<Infallible, $err>>+)+,
            E: 'static + Send {
      type Response = R;
      type Future = impl Future<Output = R> + Send;
      fn call(self, req: E, state: S) -> Self::Future {
        let st = state.clone();
        $(let $type = <S as AsyncTryInto<$type>>::try_into_async(st.clone());)+
        async move {
          let ($($err,)+) = join!(
            $($type,)+
          );
          self(req, $($err?),+).await
        }
      }
    }
}
}
impl_handler!((T1, E1),);
impl_handler!((T1, E1), (T2, E2),);
impl_handler!((T1, E1), (T2, E2), (T3, E3),);
impl_handler!((T1, E1), (T2, E2), (T3, E3), (T4, E4),);
impl_handler!((T1, E1), (T2, E2), (T3, E3), (T4, E4), (T5, E5),);
impl_handler!((T1, E1), (T2, E2), (T3, E3), (T4, E4), (T5, E5), (T6, E6),);

#[allow(clippy::type_complexity)]
pub struct HandlerService<H, T, S, E, R, Er, F> {
	handler: H,
	state: S,
	f: F,
	_marker: PhantomData<fn(E, T) -> Result<R, Er>>,
}
impl<H, T, S, E, R, Er, F> Clone for HandlerService<H, T, S, E, R, Er, F>
where
	F: Clone,
	S: Clone,
	H: Clone,
{
	fn clone(&self) -> Self {
		HandlerService {
			handler: self.handler.clone(),
			state: self.state.clone(),
			f: self.f.clone(),
			_marker: PhantomData,
		}
	}
}
impl<H, T, S, E, R, Er, F> HandlerService<H, T, S, E, R, Er, F> {
	fn new(handler: H, state: S, f: F) -> Self
	where
		H: Handler<T, S, E>,
		S: Clone,
		F: FnOnce(<H as Handler<T, S, E>>::Response) -> Result<R, Er>,
	{
		HandlerService { handler, state, f, _marker: PhantomData }
	}
}

impl<H, T, S, E, R, Er, O, F> Service<E> for HandlerService<H, T, S, E, R, Er, F>
where
	H: Handler<T, S, E, Response = O>,
	S: Clone,
	F: FnOnce(O) -> Result<R, Er>,
	F: Clone,
{
	type Response = R;
	type Future = Map<<H as Handler<T, S, E>>::Future, F>;
	type Error = Er;

	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: E) -> Self::Future {
		let handler = self.handler.clone();
		let state = self.state.clone();
		handler.call(req, state).map(self.f.clone())
	}
}
