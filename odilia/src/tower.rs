#![allow(dead_code)]

use crate::state::CacheProvider;
use crate::ScreenReaderState;
use atspi::AtspiError;
use atspi::Event;
use atspi::EventProperties;
use atspi::EventTypeProperties;
use odilia_common::errors::OdiliaError;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

use futures::future::join;
use futures::future::join3;
use futures::future::join4;
use futures::future::join5;
use futures::future::try_join_all;
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

use odilia_cache::{AccessiblePrimitive, Cache, CacheItem};
use odilia_common::command::{
	CommandType, CommandTypeDynamic, IntoCommands, OdiliaCommand as Command,
	OdiliaCommandDiscriminants as CommandDiscriminants, Speak, TryIntoCommands,
};

#[derive(Debug, Clone)]
pub struct CacheEvent<E: EventProperties + Debug> {
	pub inner: E,
	pub item: CacheItem,
}
impl<E> CacheEvent<E>
where
	E: EventProperties + Debug,
{
	async fn from_event(ev: E, cache: Arc<Cache>) -> Result<Self, Error> {
		let item = cache.from_event(&ev).await?;
		Ok(CacheEvent { inner: ev, item })
	}
}

#[derive(Clone)]
pub struct CacheLayer<I> {
	cache: Arc<Cache>,
	_marker: PhantomData<I>,
}
impl<I> CacheLayer<I> {
	fn new(cache: Arc<Cache>) -> Self {
		CacheLayer { cache, _marker: PhantomData }
	}
}
impl<S, I> Layer<S> for CacheLayer<I>
where
	S: Service<(I, Arc<Cache>)>,
{
	type Service = CacheService<S, I>;
	fn layer(&self, inner: S) -> CacheService<S, I> {
		CacheService { inner, cache: Arc::clone(&self.cache), _marker: PhantomData }
	}
}
pub struct CacheService<S, I> {
	inner: S,
	cache: Arc<Cache>,
	_marker: PhantomData<fn(I)>,
}
impl<I, S> Service<I> for CacheService<S, I>
where
	S: Service<(I, Arc<Cache>)>,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future = S::Future;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}
	fn call(&mut self, req: I) -> Self::Future {
		self.inner.call((req, Arc::clone(&self.cache)))
	}
}

impl<E> AsyncTryFrom<(E, Arc<Cache>)> for CacheEvent<E>
where
	E: EventProperties + Clone + Send + Sync + Debug,
{
	type Error = Error;
	type Future = impl Future<Output = Result<Self, Self::Error>> + Send;
	fn try_from_async((ev, cache): (E, Arc<Cache>)) -> Self::Future {
		CacheEvent::<E>::from_event(ev, cache)
	}
}

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
macro_rules! impl_from_state {
($join_fn:ident, $(($type:ident,$err:ident),)+) => {
    #[allow(non_snake_case)]
    impl<I, $($type, $err,)+> FromState<I> for ($($type,)+)
    where
        $($type: FromState<I, Error = $err>,)+
        $(Error: From<$err>,)+
        $($err: Send,)+
        {
            type Error = Error;
            type Future = impl Future<Output = Result<Self, Self::Error>>;
            fn try_from_state(state: &ScreenReaderState, i: &I) -> Self::Future {
                $join_fn(
                    $(<$type>::try_from_state(state, i),)+
                )
                .map(|($($type,)+)| {
                    Ok((
                        $($type?,)+
                    ))
                })
            }
        }
    }
}
impl_from_state!(join, (T1, E1), (T2, E2),);
impl_from_state!(join3, (T1, E1), (T2, E2), (T3, E3),);
impl_from_state!(join4, (T1, E1), (T2, E2), (T3, E3), (T4, E4),);
impl_from_state!(join5, (T1, E1), (T2, E2), (T3, E3), (T4, E4), (T5, E5),);

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

pub struct AsyncTryIntoService<O, I: AsyncTryInto<O>, S, R, Fut1> {
	inner: S,
	_marker: PhantomData<fn(O, I, Fut1) -> R>,
}
impl<O, E, I: AsyncTryInto<O, Error = E>, S, R, Fut1> AsyncTryIntoService<O, I, S, R, Fut1> {
	fn new(inner: S) -> Self {
		AsyncTryIntoService { inner, _marker: PhantomData }
	}
}
#[derive(Clone)]
pub struct AsyncTryIntoLayer<O, I: AsyncTryInto<O>> {
	_marker: PhantomData<fn(I) -> O>,
}
impl<O, E, I: AsyncTryInto<O, Error = E>> AsyncTryIntoLayer<O, I> {
	fn new() -> Self {
		AsyncTryIntoLayer { _marker: PhantomData }
	}
}

impl<I: AsyncTryInto<O>, O, S, Fut1> Layer<S> for AsyncTryIntoLayer<O, I>
where
	S: Service<O, Future = Fut1>,
{
	type Service = AsyncTryIntoService<O, I, S, <S as Service<O>>::Response, Fut1>;
	fn layer(&self, inner: S) -> Self::Service {
		AsyncTryIntoService::new(inner)
	}
}

impl<O, E, I: AsyncTryInto<O>, S, R, Fut1> Service<I> for AsyncTryIntoService<O, I, S, R, Fut1>
where
	I: AsyncTryInto<O>,
	E: From<<I as AsyncTryInto<O>>::Error>,
	<I as AsyncTryInto<O>>::Error: Send,
	S: Service<O, Response = R, Future = Fut1> + Clone + Send,
	O: Send,
	Fut1: Future<Output = Result<R, E>> + Send,
{
	type Response = R;
	type Future = impl Future<Output = Result<R, E>> + Send;
	type Error = E;
	fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: I) -> Self::Future {
		let clone = self.inner.clone();
		let mut inner = std::mem::replace(&mut self.inner, clone);
		async move {
			match req.try_into_async().await {
				Ok(resp) => inner.call(resp).await,
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
		let inner = std::mem::replace(&mut self.inner, ic);
		SerialServiceFuture { inner, req, results: Vec::with_capacity(len) }
	}
}

pub struct Handlers<S> {
	state: S,
	atspi_handlers:
		HashMap<(&'static str, &'static str), Vec<BoxService<Event, Vec<Command>, Error>>>,
	command_handlers: BTreeMap<CommandDiscriminants, BoxService<Command, (), Error>>,
}

impl<S> Handlers<S>
where
	S: Clone + Send + Sync + CacheProvider + 'static,
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
		H: Handler<T, S, CacheEvent<E>, Response = R> + Send + Sync + 'static,
		E: atspi::BusProperties
			+ EventProperties
			+ TryFrom<Event>
			+ Debug
			+ Send
			+ Sync
			+ 'static
			+ Clone,
		<E as TryFrom<Event>>::Error: Send + Sync + std::fmt::Debug + Into<Error>,
		OdiliaError: From<<E as TryFrom<Event>>::Error>,
		T: Clone + 'static,
		R: TryIntoCommands + Send + Sync + 'static,
	{
		let ie_layer: AsyncTryIntoLayer<CacheEvent<E>, (E, Arc<Cache>)> =
			AsyncTryIntoLayer::new();
		let ch_layer: CacheLayer<E> = CacheLayer::new(Arc::clone(&self.state.cache()));
		let ti_layer: TryIntoLayer<E, Request> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws =
			handler.with_state_and_fn(state, <R as TryIntoCommands>::try_into_commands);
		let ie_serv: AsyncTryIntoService<CacheEvent<E>, (E, Arc<Cache>), _, _, _> =
			ie_layer.layer(ws);
		let ch_serv = ch_layer.layer(ie_serv);
		let serv = ti_layer.layer(ch_serv);
		let dn = (
			<E as atspi::BusProperties>::DBUS_MEMBER,
			<E as atspi::BusProperties>::DBUS_INTERFACE,
		);
		let bs = BoxService::new(serv);
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

macro_rules! impl_handler {
    ($join_fn:ident, $(($type:ident,$err:ident),)+) => {
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
        async move {
          let ($($type,)+) = $join_fn(
            $(<S as AsyncTryInto<$type>>::try_into_async(st.clone()),)+
          )
          .await;
          self(req, $($type?),+).await
        }
      }
    }
}
}
impl_handler!(join, (T1, E1), (T2, E2),);
impl_handler!(join3, (T1, E1), (T2, E2), (T3, E3),);
impl_handler!(join4, (T1, E1), (T2, E2), (T3, E3), (T4, E4),);
impl_handler!(join5, (T1, E1), (T2, E2), (T3, E3), (T4, E4), (T5, E5),);

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
