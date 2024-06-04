#![allow(dead_code)]

use crate::ScreenReaderState;
use atspi::AtspiError;
use atspi::Event;
use atspi::EventTypeProperties;
use odilia_common::errors::OdiliaError;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;

use futures::future::join;
use futures::future::join3;
use futures::future::ErrInto;
use futures::future::FutureExt as FatFutureExt;
use futures::future::Map;
use futures::future::MaybeDone;
use futures::future::TryFutureExt;
use futures::future::{err, Either, Ready};
use futures::{Stream, StreamExt};
use futures_lite::FutureExt;
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::task::Context;
use std::task::Poll;

use tower::util::BoxCloneService;
use tower::util::BoxService;
use tower::Layer;
use tower::Service;
use tower::ServiceExt;

use tokio::sync::mpsc::{Receiver, Sender};

use odilia_common::command::{
	CommandType, CommandTypeDynamic, IntoCommands, OdiliaCommand as Command,
	OdiliaCommandDiscriminants as CommandDiscriminants, Speak, TryIntoCommands,
};

pub trait FromState<T>: Sized + Send {
	type Error: Send;
	type Future: Future<Output = Result<Self, Self::Error>> + Send;
	fn try_from_state(state: &ScreenReaderState, t: &T) -> Self::Future;
}
impl<T, U: FromState<T>> AsyncTryFrom<(&ScreenReaderState, &T)> for U
where
	<U as FromState<T>>::Error: Into<OdiliaError>,
{
	type Error = OdiliaError;
	type Future = ErrInto<U::Future, Self::Error>;
	fn try_from_async(state: (&ScreenReaderState, &T)) -> Self::Future {
		U::try_from_state(state.0, state.1).err_into()
	}
}
impl<U, T: FromState<U>> FromState<U> for (T,) {
	type Error = T::Error;
	type Future = impl Future<Output = Result<Self, Self::Error>>;
	fn try_from_state(state: &ScreenReaderState, t: &U) -> Self::Future {
		T::try_from_state(state, t).map_ok(|res| (res,))
	}
}
impl<E1, E2, T1, T2, I> FromState<I> for (T1, T2)
where
	T1: FromState<I, Error = E1>,
	T2: FromState<I, Error = E2>,
	E1: Into<Error> + Send,
	E2: Into<Error> + Send,
{
	type Error = Error;
	type Future = impl Future<Output = Result<Self, Self::Error>>;
	fn try_from_state(state: &ScreenReaderState, i: &I) -> Self::Future {
		join(T1::try_from_state(state, i), T2::try_from_state(state, i)).map(|(r1, r2)| {
			match (r1, r2) {
				(Ok(t1), Ok(t2)) => Ok((t1, t2)),
				(Err(e1), _) => Err(e1.into()),
				(_, Err(e2)) => Err(e2.into()),
			}
		})
	}
}
impl<E1, E2, E3, T1, T2, T3, I> FromState<I> for (T1, T2, T3)
where
	T1: FromState<I, Error = E1>,
	T2: FromState<I, Error = E2>,
	T3: FromState<I, Error = E3>,
	E1: Into<Error> + Send,
	E2: Into<Error> + Send,
	E3: Into<Error> + Send,
{
	type Error = Error;
	type Future = impl Future<Output = Result<Self, Self::Error>>;
	fn try_from_state(state: &ScreenReaderState, i: &I) -> Self::Future {
		join3(
			T1::try_from_state(state, i),
			T2::try_from_state(state, i),
			T3::try_from_state(state, i),
		)
		.map(|(r1, r2, r3)| match (r1, r2, r3) {
			(Ok(t1), Ok(t2), Ok(t3)) => Ok((t1, t2, t3)),
			(Err(e1), _, _) => Err(e1.into()),
			(_, Err(e2), _) => Err(e2.into()),
			(_, _, Err(e3)) => Err(e3.into()),
		})
	}
}

pub trait AsyncTryFrom<T>: Sized + Send {
	type Error: Send;
	type Future: Future<Output = Result<Self, Self::Error>> + Send;

	fn try_from_async(value: T) -> Self::Future;
}
pub trait AsyncTryInto<T>: Sized + Send {
	type Error: Send;
	type Future: Future<Output = Result<T, Self::Error>> + Send;

	fn try_into_async(self) -> Self::Future;
}
impl<T: Send, U: AsyncTryFrom<T> + Send> AsyncTryInto<U> for T {
	type Error = U::Error;
	type Future = U::Future;
	fn try_into_async(self: T) -> Self::Future {
		U::try_from_async(self)
	}
}

/*
impl<T: Send> AsyncTryFrom<T> for T {
    type Error = std::convert::Infallible;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn try_from_async(value: T) -> Self::Future {
	std::future::ready(Ok(value))
    }
}
*/

struct AsyncTryIntoService<S, T, E> {
	inner: S,
	_marker: PhantomData<(T, E)>,
}

impl<S, T, E> AsyncTryIntoService<S, T, E> {
	pub fn new(inner: S) -> Self {
		AsyncTryIntoService { inner, _marker: PhantomData }
	}
}
impl<S, Request, Response, T, E> Service<Request> for AsyncTryIntoService<S, T, E>
where
	S: Service<T, Response = Response> + Send + Clone,
	S::Error: Into<E>,
	E: From<S::Error> + From<<Request as AsyncTryInto<T>>::Error> + Send,
	Request: AsyncTryInto<T>,
	<S as Service<T>>::Future: Send,
	T: Send,
{
	type Response = Response;
	type Error = E;
	type Future = impl Future<Output = Result<Self::Response, Self::Error>> + Send;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx).map_err(|e| e.into())
	}
	fn call(&mut self, req: Request) -> Self::Future {
		let mut this = self.inner.clone();
		async move {
			match req.try_into_async().await {
				Ok(o) => Ok(this.call(o).await?),
				Err(e) => Err(e.into()),
			}
		}
	}
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
	command_handlers: BTreeMap<CommandDiscriminants, BoxService<Command, (), Error>>,
}

impl<S> Handlers<S>
where
	S: Clone + Send + Sync + 'static,
{
	pub fn new(state: S) -> Self {
		Handlers {
			state,
			atspi_handlers: HashMap::new(),
			command_handlers: BTreeMap::new(),
		}
	}
	pub async fn command_handler(mut self, mut commands: Receiver<Command>) {
		while let Some(cmd) = commands.recv().await {
			let dn = cmd.ctype();
			// NOTE: Why not use join_all(...) ?
			// Because this drives the futures concurrently, and we want ordered handlers.
			// Otherwise, we cannot guarentee that the caching functions get run first.
			// we could move caching to a separate, ordered system, then parallelize the other functions,
			// if we determine this is a performance problem.
			match self.command_handlers.get_mut(&dn) {
				Some(hand_fn) => match hand_fn.call(cmd).await {
					Err(e) => tracing::error!("{e:?}"),
					_ => {}
				},
				None => {
					tracing::trace!("There are no associated handler functions for the command '{}'", cmd.ctype());
				}
			}
		}
	}
	pub async fn atspi_handler<R>(mut self, mut events: R, cmds: Sender<Command>)
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
			for res in results {
				match res {
					Ok(oks) => {
						for ok in oks {
							match cmds.send(ok).await {
								Ok(()) => {}
								Err(e) => {
									tracing::error!("Could not send command {:?} over channel! This usually means the channel is full, which is bad!", e);
								}
							}
						}
					}
					Err(e) => {
						tracing::error!("Handler function failed: {e:?}");
					}
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
	pub fn command_listener<H, T, C, R>(mut self, handler: H) -> Self
	where
		H: Handler<T, S, C, Response = R> + Send + Sync + 'static,
		C: CommandType + Send + Sync + 'static,
		Command: TryInto<C>,
		OdiliaError: From<<Command as TryInto<C>>::Error>,
		R: Into<Result<(), Error>> + 'static,
		T: 'static,
	{
		let tflayer: TryIntoLayer<C, Command> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws = handler.with_state_and_fn(state, <R as Into<Result<(), Error>>>::into);
		let tfserv = tflayer.layer(ws);
		let dn = C::CTYPE;
		let bs = BoxService::new(tfserv);
		self.command_handlers.entry(dn).or_insert(bs);
		Self {
			state: self.state,
			atspi_handlers: self.atspi_handlers,
			command_handlers: self.command_handlers,
		}
	}
	pub fn atspi_listener<H, T, E, R>(mut self, handler: H) -> Self
	where
		H: Handler<T, S, E, Response = R> + Send + Sync + 'static,
		E: atspi::BusProperties + TryFrom<Event> + Send + Sync + 'static,
		<E as TryFrom<Event>>::Error: Send + Sync + std::fmt::Debug + Into<Error>,
		OdiliaError: From<<E as TryFrom<Event>>::Error>,
		T: 'static,
		R: TryIntoCommands + Send + Sync + 'static,
	{
		let tflayer: TryIntoLayer<E, Request> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws =
			handler.with_state_and_fn(state, <R as TryIntoCommands>::try_into_commands);
		let tfserv = tflayer.layer(ws);
		let dn = (
			<E as atspi::BusProperties>::DBUS_MEMBER,
			<E as atspi::BusProperties>::DBUS_INTERFACE,
		);
		let bs = BoxService::new(tfserv);
		self.atspi_handlers.entry(dn).or_default().push(bs);
		Self {
			state: self.state,
			atspi_handlers: self.atspi_handlers,
			command_handlers: self.command_handlers,
		}
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
	I: TryInto<O>,
	E: From<<I as TryInto<O>>::Error>,
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
impl<F, Fut, S, T1, E, R> Handler<(Request, T1), S, E> for F
where
	F: FnOnce(E, T1) -> Fut + Clone + Send + 'static,
	Fut: Future<Output = R> + Send + 'static,
	S: Clone + AsyncTryInto<T1> + 'static,
	T1: 'static + Send,
	R: 'static
		+ std::ops::FromResidual<
			std::result::Result<Infallible, <S as AsyncTryInto<T1>>::Error>,
		>,
	E: 'static + Send,
{
	type Future = impl Future<Output = R> + Send;
	type Response = R;
	fn call(self, req: E, state: S) -> Self::Future {
		async move { self(req, state.try_into_async().await?).await }
	}
}
// breakdown of the various type parameters:
//
// F: is a function that takes E, T1, T2 (all of which are generic)
// T1, T2: are type which can be created infallibly from a reference to S, which is generic
// S: is some state type that implements Clone
// Fut: is a future whoes output is Result<Response, Error>, and which is sendable across threads
// statically
impl<F, Fut, S, T1, T2, E, R, E1, E2> Handler<(Request, T1, T2), S, E> for F
where
	F: FnOnce(E, T1, T2) -> Fut + Clone + Send + 'static,
	Fut: Future<Output = R> + Send + 'static,
	S: Clone + AsyncTryInto<T1, Error = E1> + AsyncTryInto<T2, Error = E2> + 'static + Sync,
	T1: From<S> + 'static + Send,
	T2: From<S> + 'static + Send,
	E1: 'static + Send,
	E2: 'static + Send,
	R: 'static
		+ std::ops::FromResidual<std::result::Result<Infallible, E1>>
		+ std::ops::FromResidual<std::result::Result<Infallible, E2>>,
	E: 'static + Send,
{
	type Response = R;
	type Future = impl Future<Output = R> + Send;
	fn call(self, req: E, state: S) -> Self::Future {
		let st = state.clone();
		async move {
			let (t1, t2) = join(
				<S as AsyncTryInto<T1>>::try_into_async(st.clone()),
				<S as AsyncTryInto<T2>>::try_into_async(st.clone()),
			)
			.await;
			self(req, t1?, t2?).await
		}
	}
}

#[derive(Clone)]
pub struct HandlerService<H, T, S, E, R, Er, F> {
	handler: H,
	state: S,
	f: F,
	_marker: PhantomData<fn(E, T) -> Result<R, Er>>,
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
