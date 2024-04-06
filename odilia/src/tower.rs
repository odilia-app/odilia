#![allow(dead_code)]

use atspi::events::{document::DocumentEvents, object::ObjectEvents};
use atspi::AtspiError;
use atspi::Event;
use atspi::GenericEvent;
use odilia_common::errors::OdiliaError;
use std::future::Future;
use std::marker::PhantomData;

use futures::future::{err, join_all, ok, Either, FutureExt, JoinAll, Ready};
use futures::{Stream, StreamExt};
use std::collections::HashMap;
use std::task::Context;
use std::task::Poll;

use tower::util::BoxService;
use tower::Layer;
use tower::Service;

#[derive(Debug)]
pub enum Command {
    Speak(String),
}

type Response = Vec<Command>;
type Request = Event;
type Error = OdiliaError;

pub struct Handlers<S> {
	state: S,
	atspi_handlers: HashMap<(String, String), Vec<BoxService<Event, Response, Error>>>,
  command_handlers: HashMap<Command, Vec<BoxService<Command, (), Error>>>,
}
impl<S> Handlers<S>
where
	S: Clone + Send + Sync + 'static,
{
	pub fn new(state: S) -> Self {
		Handlers { state, atspi_handlers: HashMap::new(), command_handlers: HashMap::new() }
	}
	pub async fn atspi_handler<R>(mut self, mut events: R)
	where
		R: Stream<Item = Result<Event, AtspiError>> + Unpin,
	{
		std::pin::pin!(&mut events);
		while let Some(Ok(ev)) = events.next().await {
			let r = match ev {
				Event::Object(ObjectEvents::StateChanged(e)) => {
					self.call_event_listeners(e).await
				}
				Event::Object(ObjectEvents::TextCaretMoved(e)) => {
					self.call_event_listeners(e).await
				}
				Event::Object(ObjectEvents::ChildrenChanged(e)) => {
					self.call_event_listeners(e).await
				}
				Event::Object(ObjectEvents::TextChanged(e)) => {
					self.call_event_listeners(e).await
				}
				Event::Document(DocumentEvents::LoadComplete(e)) => {
					self.call_event_listeners(e).await
				}
				_ => {
					println!("Not implemented yet....");
					vec![Ok(vec![])]
				}
			};
			for res in r {
				if let Ok(resp) = res {
				} else if let Err(err) = res {
					println!("ERR: {:?}", err);
				}
			}
		}
	}
	async fn call_event_listeners<'a, E>(&mut self, ev: E) -> Vec<Result<Response, Error>>
	where
		E: atspi::GenericEvent<'a> + Into<Event> + Send + Sync + 'a,
	{
		let dn = (
			<E as atspi::GenericEvent<'a>>::DBUS_MEMBER.into(),
			<E as atspi::GenericEvent<'a>>::DBUS_INTERFACE.into(),
		);
		let input = ev.into();
    // NOTE: Why not use join_all(...) ?
    // Because this drives the futures concurrently, and we want ordered handlers.
    // Otherwise, we cannot guarentee that the caching functions get run first.
    // we could move caching to a separate, ordered system, then parallelize the other functions,
    // if we determine this is a performance problem.
    let mut results = vec![];
		for hand in self.atspi_handlers.entry(dn).or_default() {
			results.push(hand.call(input.clone()).await);
		}
		results
	}
	pub fn add_listener<'a, H, T, E>(mut self, handler: H) -> Self
	where
		H: Handler<T, S, E> + Send + Sync + 'static,
		E: atspi::GenericEvent<'a> + TryFrom<Event> + Send + Sync + 'static,
		<E as TryFrom<Event>>::Error: Send + Sync + std::fmt::Debug + Into<Error>,
		OdiliaError: From<<E as TryFrom<Event>>::Error>,
		T: 'static,
	{
		let tflayer: TryIntoLayer<E, Request> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws = handler.with_state(state);
		let tfserv = tflayer.layer(ws);
		let dn = (
			<E as atspi::GenericEvent<'a>>::DBUS_MEMBER.into(),
			<E as atspi::GenericEvent<'a>>::DBUS_INTERFACE.into(),
		);
		let bs = BoxService::new(tfserv);
		self.atspi_handlers.entry(dn).or_default().push(bs);
		Self { state: self.state, atspi_handlers: self.atspi_handlers, command_handlers: self.command_handlers }
	}
}

pub struct TryIntoService<O, I: TryInto<O>, S, R, Fut1> {
	inner: S,
	_marker: PhantomData<fn(O, I, Fut1) -> R>,
}
impl<O, E, I: TryInto<O, Error = E>, S, R, Fut1> TryIntoService<O, I, S, R, Fut1> {
	fn new(inner: S) -> Self {
		TryIntoService { inner, _marker: PhantomData }
	}
}
pub struct TryIntoLayer<O, I: TryInto<O>> {
	_marker: PhantomData<fn(I) -> O>,
}
impl<O, E, I: TryInto<O, Error = E>> TryIntoLayer<O, I> {
	fn new() -> Self {
		TryIntoLayer { _marker: PhantomData }
	}
}

impl<I: TryInto<O>, O, S, Fut1> Layer<S> for TryIntoLayer<O, I>
where
	S: Service<O, Future = Fut1>,
{
	type Service = TryIntoService<O, I, S, <S as Service<O>>::Response, Fut1>;
	fn layer(&self, inner: S) -> Self::Service {
		TryIntoService::new(inner)
	}
}

impl<O, E, I: TryInto<O>, S, R, Fut1> Service<I> for TryIntoService<O, I, S, R, Fut1>
where
	O: TryFrom<I>,
	E: From<<O as TryFrom<I>>::Error> + From<<I as TryInto<O>>::Error>,
	S: Service<O, Response = R, Future = Fut1>,
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
			Err(e) => Either::Right(err(e.into())),
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

impl<F, Fut, S, E> Handler<((),), S, E> for F
where
	F: FnOnce() -> Fut + Clone + Send,
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
	F: FnOnce(E) -> Fut + Clone + Send,
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
	F: FnOnce(E, T1) -> Fut + Clone + Send,
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
	F: FnOnce(E, T1, T2) -> Fut + Clone + Send,
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
