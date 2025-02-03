#![allow(dead_code)]

use crate::state::ScreenReaderState;
use crate::tower::{
	choice::{ChoiceService, ChooserStatic},
	from_state::TryFromState,
	service_set::ServiceSet,
	Handler, ServiceExt as OdiliaServiceExt,
};
use atspi::AtspiError;
use atspi::BusProperties;
use atspi::Event;
use atspi::EventProperties;
use atspi::EventTypeProperties;
use odilia_common::command::{OdiliaCommand, OdiliaCommandDiscriminants};
use odilia_common::errors::OdiliaError;
use std::fmt::Debug;
use std::sync::Arc;

use futures::{Stream, StreamExt};

use tower::util::BoxCloneService;
use tower::Service;
use tower::ServiceExt;

use tokio::sync::mpsc::Receiver;

use odilia_common::command::{
	CommandType, OdiliaCommand as Command, OdiliaCommandDiscriminants as CommandDiscriminants,
	TryIntoCommands,
};

type Response = Vec<Command>;
type Request = Event;
type Error = OdiliaError;

type AtspiHandler = BoxCloneService<Event, (), Error>;
type CommandHandler = BoxCloneService<Command, (), Error>;

pub struct Handlers {
	state: Arc<ScreenReaderState>,
	atspi: ChoiceService<(&'static str, &'static str), ServiceSet<AtspiHandler>, Event>,
	command: ChoiceService<CommandDiscriminants, ServiceSet<CommandHandler>, Command>,
}

impl Handlers {
	pub fn new(state: Arc<ScreenReaderState>) -> Self {
		Handlers { state, atspi: ChoiceService::new(), command: ChoiceService::new() }
	}
	pub async fn command_handler(mut self, mut commands: Receiver<Command>) {
		loop {
			let maybe_cmd = commands.recv().await;
			let Some(cmd) = maybe_cmd else {
				tracing::error!("Error cmd: {maybe_cmd:?}");
				continue;
			};
			// NOTE: Why not use join_all(...) ?
			// Because this drives the futures concurrently, and we want ordered handlers.
			// Otherwise, we cannot guarentee that the caching functions get run first.
			// we could move caching to a separate, ordered system, then parallelize the other functions,
			// if we determine this is a performance problem.
			if let Err(e) = self.command.call(cmd).await {
				tracing::error!("{e:?}");
			}
		}
	}
	#[tracing::instrument(skip_all)]
	pub async fn atspi_handler<R>(mut self, mut events: R)
	where
		R: Stream<Item = Result<Event, AtspiError>> + Unpin,
	{
		std::pin::pin!(&mut events);
		loop {
			let maybe_ev = events.next().await;
			let Some(Ok(ev)) = maybe_ev else {
				tracing::error!("Error in processing {maybe_ev:?}");
				continue;
			};
			if let Err(e) = self.atspi.call(ev).await {
				tracing::error!("{e:?}");
			}
		}
	}
	pub fn command_listener<H, T, C, R>(mut self, handler: H) -> Self
	where
		H: Handler<T, Response = R> + Send + Clone + 'static,
		<H as Handler<T>>::Future: Send,
		C: CommandType + ChooserStatic<CommandDiscriminants> + Send + 'static,
		Command: TryInto<C>,
		OdiliaError: From<<Command as TryInto<C>>::Error>
			+ From<<T as TryFromState<Arc<ScreenReaderState>, C>>::Error>,
		R: Into<Result<(), Error>> + Send + 'static,
		T: TryFromState<Arc<ScreenReaderState>, C> + Send + 'static,
		<T as TryFromState<Arc<ScreenReaderState>, C>>::Future: Send,
		<T as TryFromState<Arc<ScreenReaderState>, C>>::Error: Send,
	{
		let bs = handler
			.into_service()
			.map_response_into::<R, (), OdiliaError, T>()
			.request_async_try_from()
			.with_state(Arc::clone(&self.state))
			.request_try_from()
			.boxed_clone();
		self.command.entry(C::identifier()).or_default().push(bs);
		Self { state: self.state, atspi: self.atspi, command: self.command }
	}
	pub fn atspi_listener<H, T, R, E>(mut self, handler: H) -> Self
	where
		H: Handler<T, Response = R> + Send + Clone + 'static,
		<H as Handler<T>>::Future: Send,
		E: EventTypeProperties
			+ Debug
			+ BusProperties
			+ TryFrom<Event>
			+ EventProperties
			+ ChooserStatic<(&'static str, &'static str)>
			+ Clone
			+ Send
			+ 'static,
		OdiliaError: From<<Event as TryInto<E>>::Error>
			+ From<<T as TryFromState<Arc<ScreenReaderState>, E>>::Error>,
		R: TryIntoCommands + Send + 'static,
		T: TryFromState<Arc<ScreenReaderState>, E> + Send + 'static,
		<T as TryFromState<Arc<ScreenReaderState>, E>>::Error: Send + 'static,
		<T as TryFromState<Arc<ScreenReaderState>, E>>::Future: Send,
	{
		let bs = handler
			.into_service()
			.unwrap_map(TryIntoCommands::try_into_commands)
			.request_async_try_from()
			.with_state(Arc::clone(&self.state))
			.request_try_from()
			.iter_into(self.command.clone())
			.map_result(
				|res: Result<
					Vec<Result<Vec<Result<(), OdiliaError>>, OdiliaError>>,
					OdiliaError,
				>| {
					res?.into_iter()
						.flatten()
						.flatten()
						.collect::<Result<(), OdiliaError>>()
				},
			)
			.boxed_clone();
		self.atspi.entry(E::identifier()).or_default().push(bs);
		Self { state: self.state, atspi: self.atspi, command: self.command }
	}
}
