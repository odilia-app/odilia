#![allow(dead_code)]

use crate::state::CacheProvider;
use crate::tower::{
	async_try::{AsyncTryFrom, AsyncTryIntoLayer, AsyncTryIntoService},
	cache::CacheLayer,
	sync_try::TryIntoLayer,
	Handler,
};
use atspi::AtspiError;
use atspi::Event;
use atspi::EventProperties;
use atspi::EventTypeProperties;
use odilia_common::errors::OdiliaError;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use std::collections::{BTreeMap, HashMap};

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
	E: EventProperties + Debug,
{
	type Error = Error;
	type Future = impl Future<Output = Result<Self, Self::Error>>;
	fn try_from_async((ev, cache): (E, Arc<Cache>)) -> Self::Future {
		CacheEvent::<E>::from_event(ev, cache)
	}
}

type Response = Vec<Command>;
type Request = Event;
type Error = OdiliaError;

type AtspiHandler = BoxService<Event, Vec<Command>, Error>;
type CommandHandler = BoxService<Command, (), Error>;

pub struct Handlers<S> {
	state: S,
	atspi: HashMap<(&'static str, &'static str), Vec<AtspiHandler>>,
	command: BTreeMap<CommandDiscriminants, CommandHandler>,
}

impl<S> Handlers<S>
where
	S: Clone + Send + Sync + CacheProvider + 'static,
{
	pub fn new(state: S) -> Self {
		Handlers { state, atspi: HashMap::new(), command: BTreeMap::new() }
	}
	pub async fn command_handler(mut self, mut commands: Receiver<Command>) {
		while let Some(cmd) = commands.recv().await {
			let dn = cmd.ctype();
			// NOTE: Why not use join_all(...) ?
			// Because this drives the futures concurrently, and we want ordered handlers.
			// Otherwise, we cannot guarentee that the caching functions get run first.
			// we could move caching to a separate, ordered system, then parallelize the other functions,
			// if we determine this is a performance problem.
			if let Some(hand_fn) = self.command.get_mut(&dn) {
				if let Err(e) = hand_fn.call(cmd).await {
					tracing::error!("{e:?}");
				}
			} else {
				tracing::trace!("There are no associated handler functions for the command '{}'", cmd.ctype());
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
			match self.atspi.get_mut(&dn) {
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
			<E as atspi::BusProperties>::DBUS_MEMBER,
			<E as atspi::BusProperties>::DBUS_INTERFACE,
		);
		let input = ev.into();
		// NOTE: Why not use join_all(...) ?
		// Because this drives the futures concurrently, and we want ordered handlers.
		// Otherwise, we cannot guarentee that the caching functions get run first.
		// we could move caching to a separate, ordered system, then parallelize the other functions,
		// if we determine this is a performance problem.
		let mut results = vec![];
		for hand in self.atspi.entry(dn).or_default() {
			results.push(hand.call(input.clone()).await);
		}
		results
	}
	pub fn command_listener<H, T, C, R>(mut self, handler: H) -> Self
	where
		H: Handler<T, C, Response = R> + Send + Clone + 'static,
		<H as Handler<T, C>>::Future: Send,
		C: CommandType + Send + 'static,
		Command: TryInto<C>,
		OdiliaError: From<<Command as TryInto<C>>::Error>,
		R: Into<Result<(), Error>> + 'static,
		T: 'static,
	{
		let tflayer: TryIntoLayer<C, Command> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws = handler.into_service();
		let tfserv = tflayer.layer(ws);
		let dn = C::CTYPE;
		let bs = BoxService::new(tfserv);
		self.command.entry(dn).or_insert(bs);
		Self { state: self.state, atspi: self.atspi, command: self.command }
	}
	pub fn atspi_listener<H, T, E, R>(mut self, handler: H) -> Self
	where
		H: Handler<T, CacheEvent<E>, Response = R> + Send + Clone + 'static,
		<H as Handler<T, CacheEvent<E>>>::Future: Send,
		E: EventProperties
			+ TryFrom<Event>
			+ Debug
			+ atspi::BusProperties
			+ Send
			+ Sync
			+ 'static,
		Event: TryInto<E>,
		OdiliaError: From<<Event as TryInto<E>>::Error>,
		R: TryIntoCommands + 'static,
		T: 'static,
	{
		let ie_layer: AsyncTryIntoLayer<CacheEvent<E>, (E, Arc<Cache>)> =
			AsyncTryIntoLayer::new();
		let ch_layer: CacheLayer<E> = CacheLayer::new(Arc::clone(&self.state.cache()));
		let ti_layer: TryIntoLayer<E, Request> = TryIntoLayer::new();
		let state = self.state.clone();
		let ws = handler.into_service();
		let ie_serv: AsyncTryIntoService<CacheEvent<E>, (E, Arc<Cache>), _, _, _> =
			ie_layer.layer(ws);
		let ch_serv = ch_layer.layer(ie_serv);
		let serv = ti_layer.layer(ch_serv);
		let dn = (
			<E as atspi::BusProperties>::DBUS_MEMBER,
			<E as atspi::BusProperties>::DBUS_INTERFACE,
		);
		let bs = BoxService::new(serv);
		self.atspi.entry(dn).or_default().push(bs);
		Self { state: self.state, atspi: self.atspi, command: self.command }
	}
}
