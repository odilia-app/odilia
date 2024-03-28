#![allow(dead_code)]

use odilia_common::errors::OdiliaError;
use std::future::Future;
use std::marker::PhantomData;

use futures::future::{err, ok, Ready, ready, Either};
use std::collections::HashMap;
use std::task::Context;
use std::task::Poll;

use tower::Layer;
use tower::ServiceExt;
use tower::util::BoxService;
use tower::filter::Filter;
use tower::Service;

type Response = ();
type Request = atspi::Event;
type Error = OdiliaError;

pub struct Handlers<S, U, E> {
    state: S,
    atspi_handlers: HashMap<(String, String), BoxService<atspi::Event, U, E>>,
}
impl<S, U, E> Handlers<S, U, E> 
where S: Clone + Send + Sync + 'static {
    fn add_listener<'a, H, T>(&mut self, handler: H) 
    where H: Handler<T, S, E> + Send + Sync + 'static,
          E: atspi::GenericEvent<'a> + TryFrom<atspi::Event> + Send + Sync + 'static,
          <E as TryFrom<atspi::Event>>::Error: Send + Sync + std::fmt::Debug {
        let tfl: TryIntoLayer<E, Request> = TryIntoLayer::new();
        let ws = handler.with_state(self.state.clone());
        let state = self.state.clone();
        let tfs = tfl.layer(ws);
        let bs = BoxService::new(tfs);
        //self.atspi_handlers.insert(("".into(),"".into()), bs);
        /*
        self.atspi_handlers.insert(
            (<E as atspi::GenericEvent>::DBUS_MEMBER.into(),
            <E as atspi::GenericEvent>::DBUS_INTERFACE.into()),
            BoxService::new(
                atspi_event_handler(
                    handler.with_state(self.state.clone())
                )
            )
        );
        */
    }
}

pub struct TryIntoService<O, I: TryInto<O>, S, R, Fut1> {
    inner: S,
    _marker: PhantomData<fn(O, I, Fut1) -> R>,
}
impl<O, E, I: TryInto<O, Error=E>, S, R, Fut1> TryIntoService<O, I, S, R, Fut1> {
    fn new(inner: S) -> Self {
        TryIntoService {
            inner,
            _marker: PhantomData,
        }
    }
}
pub struct TryIntoLayer<O, I: TryInto<O>> {
    _marker: PhantomData<fn(I) -> O>,
}
impl<O, E, I: TryInto<O, Error=E>> TryIntoLayer<O, I> {
    fn new() -> Self {
        TryIntoLayer {
            _marker: PhantomData,
        }
    }
}

impl<I: TryInto<O>, O, S, Fut1> Layer<S> for TryIntoLayer<O, I>
where
    S: Service<O, Future=Fut1> {
    type Service = TryIntoService<O, I, S, <S as Service<O>>::Response, Fut1>;
    fn layer(&self, inner: S) -> Self::Service {
        TryIntoService::new(inner)
    }
}

impl<O, E, I: TryInto<O, Error=E>, S, R, Fut1> Service<I> for TryIntoService<O, I, S, R, Fut1>
where
    O: TryFrom<I>,
    <O as TryFrom<I>>::Error: Into<E>,
    S: Service<O, Response=R, Error=E, Future=Fut1>,
    Fut1: Future<Output = Result<R, E>>,
{
    type Response = R;
    type Future = Either<Fut1, Ready<Result<R, E>>>;
    type Error = E;
    fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: I) -> Self::Future {
       match req.try_into() {
           Ok(o) => Either::Left(self.inner.call(o)),
           Err(e) => {
               Either::Right(err::<R, E>(e.into()))
           }
       }
    }
}

pub trait Handler<T, S, E>: Clone {
	type Future: Future<Output = Result<Response, Error>> + Send + 'static;
	fn with_state(self, state: S) -> HandlerService<Self, T, S, E> {
		HandlerService { handler: self, state, _marker: PhantomData }
	}
	fn call(self, req: E, state: S) -> Self::Future;
}

fn atspi_event_handler<H, T, S, E>(h: HandlerService<H, T, S, E>) -> impl Service<Request>
where
	S: Clone,
	E: TryFrom<Request>,
  <E as TryFrom<Request>>::Error: Send + Sync + std::error::Error + 'static,
	H: Handler<T, S, E>,
{
	Filter::new(h, <E as TryFrom<Request>>::try_from)
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

#[derive(Clone)]
pub struct HandlerService<H, T, S, E> {
	handler: H,
	state: S,
	_marker: PhantomData<fn(E) -> T>,
}

impl<H, T, S, E> Service<E> for HandlerService<H, T, S, E>
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
	fn call(&mut self, req: E) -> Self::Future {
		let handler = self.handler.clone();
		let state = self.state.clone();
		handler.call(req, state)
	}
}
