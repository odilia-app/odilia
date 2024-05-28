#![allow(dead_code)]

use atspi::AtspiError;
use atspi::Event;
use atspi::EventTypeProperties;
use odilia_common::errors::OdiliaError;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use futures::future::MaybeDone;
use futures::future::{err, Either, Ready};
use futures::{Stream, StreamExt};
use futures_lite::FutureExt;
use std::collections::HashMap;
use std::task::Context;
use std::task::Poll;

use tower::util::BoxCloneService;
use tower::util::BoxService;
use tower::Layer;
use tower::Service;

use enum_dispatch::enum_dispatch;

trait CommandType {
	const CTYPE: &'static str;
}
#[enum_dispatch]
trait CommandTypeDynamic {
	fn ctype(&self) -> &'static str;
}
impl<T: CommandType> CommandTypeDynamic for T {
	fn ctype(&self) -> &'static str {
		T::CTYPE
	}
}

#[derive(Debug)]
pub struct Speak(String);
impl CommandType for Speak {
	const CTYPE: &'static str = "speak";
}

#[derive(Debug)]
#[enum_dispatch(CommandTypeDynamic)]
pub enum Command {
	Speak(Speak),
}

type Response = Vec<Command>;
type Request = Event;
type Error = OdiliaError;

/// `SerialFuture` is a way to link a variable number of dependent futures.
/// You can race!() two `SerialFuture`s, which will cause the two, non-dependent chains of futures to poll concurrently, while completing the individual serial futures, well, serially.
///
/// Why not just use `.await`? like so:
///
/// ```rust,norun
/// return (fut1.await, fut2.await, ...);
/// ```
///
/// Because you may have a variable number of functions that need to run in series.
/// Think of handlers for an event, if you want the results to be deterministic, you will need to run the event listeners in series, even if multiple events could come in and each trigger its own set of listeners to execute syhncronously, the set of all even listeners can technically be run concorrently.
pub struct SerialFutures<F>
where
	F: Future,
{
	// TODO: look into MaybeDone
	inner: Pin<Box<[MaybeDone<F>]>>,
}
fn serial_futures<I>(iter: I) -> SerialFutures<I::Item>
where
	I: IntoIterator,
	I::Item: futures::TryFuture,
{
	SerialFutures {
		inner: iter.into_iter().map(MaybeDone::Future).collect::<Box<[_]>>().into(),
	}
}
impl<F> Unpin for SerialFutures<F> where F: Future {}

impl<F> Future for SerialFutures<F>
where
	F: futures::TryFuture + Unpin,
{
	type Output = Result<Vec<F::Output>, F::Error>;
	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		for mfut in self.inner.as_mut().get_mut() {
			match mfut {
				MaybeDone::Future(fut) => match fut.poll(cx) {
					Poll::Pending => return Poll::Pending,
					_ => {
						continue;
					}
				},
				_ => {
					continue;
				}
			}
		}
		let result = self
			.inner
			.as_mut()
			.get_mut()
			.iter_mut()
			.map(|f| Pin::new(f))
			.map(|e| e.take_output().unwrap())
			.collect();
		Poll::Ready(Ok(result))
	}
}

pub struct SerialHandlers<I, O, E> {
	inner: Vec<BoxCloneService<I, O, E>>,
}

#[pin_project::pin_project]
pub struct SerialServiceFuture<I, O, E> {
	req: I,
	inner: Vec<BoxCloneService<I, O, E>>,
	results: Vec<Result<O, E>>,
}

impl<I, O, E> Future for SerialServiceFuture<I, O, E>
where
	I: Clone,
	O: Clone,
	E: Clone,
	// Assuming YourInnerType implements a call function.
{
	type Output = Result<Vec<Result<O, E>>, E>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.as_mut().project();
		let rc = this.req.clone();
		loop {
			if let Some(s) = this.inner.into_iter().next() {
				match s.call(rc.clone()).poll(cx) {
					Poll::Pending => return Poll::Pending,
					Poll::Ready(result) => {
						this.results.push(result);
					}
				}
			} else {
				break;
			}
		}
		return Poll::Ready(Ok(this.results.to_vec()));
	}
}

impl<I, O, E> Service<I> for SerialHandlers<I, O, E>
where
	I: Clone + Send + Sync,
	O: Send + Clone,
	E: Send + Clone,
{
	type Response = Vec<Result<O, E>>;
	type Error = E;
	type Future = SerialServiceFuture<I, O, E>;
	fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), E>> {
		for service in &mut self.inner {
			let _ = service.poll_ready(ctx)?;
		}
		Poll::Ready(Ok(()))
	}

	fn call(&mut self, req: I) -> Self::Future {
		let len = self.inner.len();
		let ic = self.inner.clone();
		SerialServiceFuture { inner: ic, req, results: Vec::with_capacity(len) }
	}
}

pub struct Handlers<S> {
	state: S,
	atspi_handlers:
		HashMap<(&'static str, &'static str), Vec<BoxService<Event, Response, Error>>>,
	command_handlers:
		HashMap<&'static str, Vec<BoxService<Command, (), Error>>>,
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
			let dn = (ev.member(), ev.interface());
			// NOTE: Why not use join_all(...) ?
			// Because this drives the futures concurrently, and we want ordered handlers.
			// Otherwise, we cannot guarentee that the caching functions get run first.
			// we could move caching to a separate, ordered system, then parallelize the other functions,
			// if we determine this is a performance problem.
			let mut results = vec![];
			match self.atspi_handlers.get_mut(&dn) {
				Some(hands) => {
					for hand in hands {
						results.push(hand.call(ev.clone()).await);
					}
				}
				None => {
					tracing::trace!("There are no associated handler functions for {}:{}", ev.interface(), ev.member());
				}
			}
		}
	}
	async fn call_event_listeners<E>(&mut self, ev: E) -> Vec<Result<Response, Error>>
	where
		E: atspi::BusProperties + Into<Event> + Send + Sync,
	{
		let dn = (
			<E as atspi::BusProperties>::DBUS_MEMBER.into(),
			<E as atspi::BusProperties>::DBUS_INTERFACE.into(),
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
	pub fn command_listener<H, T, C>(mut self, handler: H) -> Self 
	where
		H: Handler<T, S, C, Response = ()> + Send + Sync + 'static,
		C: CommandType + TryFrom<Command> + Send + Sync + 'static,
		<C as TryFrom<Command>>::Error: Send + Sync + Into<Error>,
		OdiliaError: From<<C as TryFrom<Command>>::Error>,
		T: 'static,
	{
		let tflayer: TryIntoLayer<C, Command> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws = handler.with_state(state);
		let tfserv = tflayer.layer(ws);
		let dn = C::CTYPE;
		let bs = BoxService::new(tfserv);
		self.command_handlers.entry(dn).or_default().push(bs);
		Self { state: self.state, atspi_handlers: self.atspi_handlers, command_handlers: self.command_handlers }
	}
	pub fn atspi_listener<H, T, E>(mut self, handler: H) -> Self
	where
		H: Handler<T, S, E, Response = Vec<Command>> + Send + Sync + 'static,
		E: atspi::BusProperties + TryFrom<Event> + Send + Sync + 'static,
		<E as TryFrom<Event>>::Error: Send + Sync + std::fmt::Debug + Into<Error>,
		OdiliaError: From<<E as TryFrom<Event>>::Error>,
		T: 'static,
	{
		let tflayer: TryIntoLayer<E, Request> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws = handler.with_state(state);
		let tfserv = tflayer.layer(ws);
		let dn = (
			<E as atspi::BusProperties>::DBUS_MEMBER,
			<E as atspi::BusProperties>::DBUS_INTERFACE,
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
	type Response;
	type Future: Future<Output = Result<Self::Response, Error>> + Send + 'static;
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
	type Response = Response;
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
	type Response = Response;
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
	type Response = Response;
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
	type Response = Response;
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
{
	type Response = <H as Handler<T, S, E>>::Response;
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

