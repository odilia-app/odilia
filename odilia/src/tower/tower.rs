#![allow(dead_code)]

use crate::state::CacheProvider;
use crate::tower::{
	async_try::{AsyncTryFrom, AsyncTryInto, AsyncTryIntoLayer, AsyncTryIntoService},
	cache::CacheLayer,
	sync_try::TryIntoLayer,
};
use atspi::AtspiError;
use atspi::Event;
use atspi::EventProperties;
use atspi::EventTypeProperties;
use odilia_common::errors::OdiliaError;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use futures::future::FutureExt as FatFutureExt;
use futures::future::Map;
use futures::join;
use futures::{Stream, StreamExt};
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::task::Context;
use std::task::Poll;

use tower::util::BoxService;
use tower::Layer;
use tower::Service;

use tokio::sync::mpsc::{Receiver, Sender};

use odilia_cache::{Cache, CacheItem};
use odilia_common::command::{
	CommandType, CommandTypeDynamic, OdiliaCommand as Command,
	OdiliaCommandDiscriminants as CommandDiscriminants, TryIntoCommands,
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

type Response = Vec<Command>;
type Request = Event;
type Error = OdiliaError;

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
