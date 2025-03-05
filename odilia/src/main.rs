#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code
)]
#![allow(clippy::multiple_crate_versions)]

mod cli;
mod logging;
mod state;
mod tower;

use std::{fmt::Write, fs, path::PathBuf, process::exit, sync::Arc, time::Duration};

use crate::cli::Args;
use crate::state::AccessibleHistory;
use crate::state::Command;
use crate::state::CurrentCaretPos;
use crate::state::InputEvent;
use crate::state::LastCaretPos;
use crate::state::LastFocused;
use crate::state::ScreenReaderState;
use crate::state::Speech;
use crate::tower::Handlers;
use crate::tower::{ActiveAppEvent, CacheEvent, Description, EventProp, Name, RelationSet};
use atspi::RelationType;
use clap::Parser;
use eyre::WrapErr;
use figment::{
	providers::{Format, Serialized, Toml},
	Figment,
};
use futures::{future::FutureExt, StreamExt};
use odilia_common::{
	command::{CaretPos, Focus, IntoCommands, OdiliaCommand, Speak, TryIntoCommands},
	errors::OdiliaError,
	events::{ScreenReaderEvent, StopSpeech},
	settings::ApplicationConfig,
};

use odilia_notify::listen_to_dbus_notifications;
use ssip::Priority;
use ssip::Request as SSIPRequest;
use tokio::{
	signal::unix::{signal, SignalKind},
	sync::mpsc,
	time::timeout,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use atspi_common::events::{document, object};
use tracing::Instrument;
#[tracing::instrument(skip(state, shutdown))]
async fn notifications_monitor(
	state: Arc<ScreenReaderState>,
	shutdown: CancellationToken,
) -> eyre::Result<()> {
	let mut stream = listen_to_dbus_notifications()
		.instrument(tracing::info_span!("creating notification listener"))
		.await?;
	loop {
		tokio::select! {
		    Some(notification) = stream.next() => {
		      let notification_message =
			format!("new notification: {}, {}, {}.", notification.app_name, notification.title, notification.body);
		      state.say(Priority::Important, notification_message).await;
		    },
		    () = shutdown.cancelled() => {
		      tracing::debug!("Shutting down notification task.");
		      break;
		    },
		}
	}
	Ok(())
}
#[tracing::instrument]
async fn sigterm_signal_watcher(
	token: CancellationToken,
	tracker: TaskTracker,
) -> eyre::Result<()> {
	let timeout_duration = Duration::from_millis(500); //todo: perhaps take this from the configuration file at some point
	let mut c = signal(SignalKind::interrupt())?;
	c.recv().instrument(tracing::debug_span!("Watching for Ctrl+C")).await;
	tracing::debug!("Asking all processes to stop.");
	tracing::debug!("cancelling all tokens");
	token.cancel();
	tracing::debug!(?timeout_duration, "waiting for all tasks to finish");
	timeout(timeout_duration, tracker.wait()).await?;
	tracing::debug!("All listeners have stopped.");
	tracing::debug!("Goodbye, Odilia!");
	Ok(())
}

use atspi::events::document::LoadCompleteEvent;
use atspi::events::object::TextCaretMovedEvent;
use atspi::Granularity;
use std::cmp::{max, min};

#[tracing::instrument(ret, err)]
async fn speak(
	Command(Speak(text, priority)): Command<Speak>,
	Speech(ssip): Speech,
) -> Result<(), odilia_common::errors::OdiliaError> {
	ssip.send(SSIPRequest::SetPriority(priority)).await?;
	ssip.send(SSIPRequest::Speak).await?;
	ssip.send(SSIPRequest::SendLines(Vec::from([text]))).await?;
	Ok(())
}

#[tracing::instrument(ret)]
async fn doc_loaded(loaded: ActiveAppEvent<LoadCompleteEvent>) -> impl TryIntoCommands {
	(Priority::Text, "Doc loaded")
}

use crate::tower::state_changed::{Focused, Unfocused};

#[tracing::instrument(ret)]
async fn focused(
	state_changed: CacheEvent<Focused>,
	EventProp(name): EventProp<Name>,
	EventProp(description): EventProp<Description>,
	EventProp(relation_set): EventProp<RelationSet>,
) -> impl TryIntoCommands {
	//because the current command implementation doesn't allow for multiple speak commands without interrupting the previous utterance, this is more or less an accumulating buffer for that utterance
	let mut utterance_buffer = String::new();
	//does this have a text or a name?
	// in order for the borrow checker to not scream that we move ownership of item.text, therefore making item partially moved, we only take a reference here, because in truth the only thing that we need to know is if the string is empty, because the extending of the buffer will imply a clone anyway
	let text = &state_changed.item.text;
	if text.is_empty() {
		//then the label can either be the accessible name, the description, or the relations set, aka labeled by another object
		//unfortunately, the or_else function of result doesn't accept async cloasures or cloasures with async blocks, so we can't use lazy loading here at the moment. The performance penalty is minimal however, because this should be in cache anyway
		let label = if let Some(n) = name.as_deref() {
			n.to_string()
		} else if let Some(d) = description.as_deref() {
			d.to_string()
		//otherwise, if this is empty too, we try to use the relations set to find the element labeling this one
		} else {
			relation_set
				.iter()
				// we only need entries which contain the wanted relationship, only labeled by for now
				.filter(|elem| elem.0 == RelationType::LabelledBy)
				.cloned()
				// we have to remove the first item of the entries, because it's constant now that we filtered by it
				//furthermore, after doing this, we'd have something like Vec<Vec<Item>>, which is not what we need, we need something like Vec<Item>, so we do both the flattening of the structure and the mapping in one function call
				.flat_map(|this| this.1)
				// from a collection of items, to a collection of strings, in this case the text of those labels, because yes, technically there can be more than one
				.map(|this| this.text)
				// gather all that into a string, separated by newlines or spaces I think
				.collect()
		};
		utterance_buffer += &label;
	} else {
		//then just append to the buffer and be done with it
		utterance_buffer += text;
	}
	let role = state_changed.item.role;
	//there has to be a space between the accessible name of an object and its role, so insert it now
	write!(utterance_buffer, " {}", role.name()).expect("Able to write to string");
	Ok(vec![
		Focus(state_changed.item.object).into(),
		Speak(utterance_buffer, Priority::Text).into(),
	])
}

#[tracing::instrument(ret)]
async fn unfocused(state_changed: CacheEvent<Unfocused>) -> impl TryIntoCommands {
	// TODO: set focused state on item to be false
	Ok::<_, OdiliaError>(())
}

#[tracing::instrument(ret, err)]
async fn new_focused_item(
	Command(Focus(new_focus)): Command<Focus>,
	AccessibleHistory(old_focus): AccessibleHistory,
) -> Result<(), OdiliaError> {
	let _ = old_focus.lock()?.push(new_focus);
	Ok(())
}

#[tracing::instrument(ret, err)]
async fn new_caret_pos(
	Command(CaretPos(new_pos)): Command<CaretPos>,
	CurrentCaretPos(pos): CurrentCaretPos,
) -> Result<(), OdiliaError> {
	pos.store(new_pos, core::sync::atomic::Ordering::Relaxed);
	Ok(())
}

#[tracing::instrument(ret)]
async fn stop_speech(InputEvent(_): InputEvent<StopSpeech>) -> impl TryIntoCommands {
	(Priority::Text, "Stop talking, eh!")
}

#[tracing::instrument(ret, err)]
async fn caret_moved(
	caret_moved: CacheEvent<TextCaretMovedEvent>,
	LastCaretPos(last_pos): LastCaretPos,
	LastFocused(last_focus): LastFocused,
) -> Result<Vec<OdiliaCommand>, OdiliaError> {
	let mut commands: Vec<OdiliaCommand> =
		vec![CaretPos(caret_moved.inner.position.try_into()?).into()];

	if last_focus == caret_moved.item.object {
		let start = min(caret_moved.inner.position.try_into()?, last_pos);
		let end = max(caret_moved.inner.position.try_into()?, last_pos);
		if let Some(text) = caret_moved.item.text.get(start..end) {
			commands.extend((Priority::Text, text.to_string()).into_commands());
		} else {
			return Err(OdiliaError::Generic(format!(
				"Slide {}..{} could not be created from {}",
				start, end, caret_moved.item.text
			)));
		}
	} else {
		let (text, _, _) = caret_moved
			.item
			.get_string_at_offset(
				caret_moved.inner.position.try_into()?,
				Granularity::Line,
			)
			.await?;
		commands.extend((Priority::Text, text).into_commands());
	}
	Ok(commands)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
	let args = Args::parse();

	//initialize the primary token for task cancelation
	let token = CancellationToken::new();

	//initialize a task tracker, which will allow us to wait for all tasks to finish
	let tracker = TaskTracker::new();

	//initializing configuration
	let config = load_configuration(args.config)?;
	//initialize logging, with the provided config
	logging::init(&config)?;

	tracing::info!(?config, "this configuration was used to prepair odilia");

	// Make sure applications with dynamic accessibility support do expose their AT-SPI2 interfaces.
	if let Err(e) = atspi_connection::set_session_accessibility(true)
		.instrument(tracing::info_span!("setting accessibility enabled flag"))
		.await
	{
		tracing::error!("Could not set AT-SPI2 IsEnabled property because: {}", e);
	}
	// this is the channel which handles all SSIP commands. If SSIP is not allowed to operate on a separate task, then waiting for the receiving message can block other long-running operations like structural navigation.
	// Although in the future, this may possibly be resolved through a proper cache, I think it still makes sense to separate SSIP's IO operations to a separate task.
	//  it is very important that this is *never* full, since it can cause deadlocking if the other task sending the request is working with zbus.
	let (ssip_req_tx, ssip_req_rx) = mpsc::channel::<ssip_client_async::Request>(128);
	let (mut ev_tx, ev_rx) =
		futures::channel::mpsc::channel::<Result<atspi::Event, atspi::AtspiError>>(10_000);
	let (input_tx, input_rx) = mpsc::channel::<ScreenReaderEvent>(255);
	// Initialize state
	let state = Arc::new(ScreenReaderState::new(ssip_req_tx, config).await?);
	let ssip = odilia_tts::create_ssip_client().await?;

	if state.say(Priority::Message, "Welcome to Odilia!".to_string()).await {
		tracing::debug!("Welcome message spoken.");
	} else {
		tracing::error!("Welcome message failed. Odilia is not able to continue in this state. Exiting now.");
		state.close_speech().await;
		exit(1);
	}

	// Register events
	tokio::try_join!(
		state.register_event::<object::StateChangedEvent>(),
		state.register_event::<object::TextCaretMovedEvent>(),
		state.register_event::<object::ChildrenChangedEvent>(),
		state.register_event::<object::TextChangedEvent>(),
		state.register_event::<document::LoadCompleteEvent>(),
		state.add_cache_match_rule(),
	)?;

	// load handlers
	let handlers = Handlers::new(state.clone())
		.command_listener(speak)
		.command_listener(new_focused_item)
		.command_listener(new_caret_pos)
		.atspi_listener(doc_loaded)
		.atspi_listener(caret_moved)
		.atspi_listener(focused)
		.atspi_listener(unfocused)
		.input_listener(stop_speech);

	let ssip_event_receiver =
		odilia_tts::handle_ssip_commands(ssip, ssip_req_rx, token.clone())
			.map(|r| r.wrap_err("Could no process SSIP request"));
	let notification_task = notifications_monitor(Arc::clone(&state), token.clone())
		.map(|r| r.wrap_err("Could not process signal shutdown."));
	let mut stream = state.atspi.event_stream();
	// There is a reason we are not reading from the event stream directly.
	// This `MessageStream` can only store 64 events in its buffer.
	// And, even if it could store more (it can via options), `zbus` specifically states that:
	// You must ensure a MessageStream is continuously polled or you will experience hangs.
	// So, we continually poll it here, then receive it on the other end.
	// Additioanlly, since sending is not async, but simply errors when there is an issue, this will
	// help us avoid hangs.
	let event_send_task = async move {
		std::pin::pin!(&mut stream);
		while let Some(ev) = stream.next().await {
			if let Err(e) = ev_tx.try_send(ev) {
				tracing::error!("Error sending event across channel! {e:?}");
			}
		}
	};
	let atspi_handlers_task = handlers.clone().atspi_handler(ev_rx);
	let input_task = odilia_input::sr_event_receiver(input_tx, token.clone());
	let input_handler = handlers.input_handler(input_rx);

	tracker.spawn(ssip_event_receiver);
	tracker.spawn(notification_task);
	tracker.spawn(atspi_handlers_task);
	tracker.spawn(event_send_task);
	tracker.spawn(input_task);
	tracker.spawn(input_handler);
	tracker.close();
	let _ = sigterm_signal_watcher(token, tracker)
		.await
		.wrap_err("can not process interrupt signal");
	Ok(())
}

fn load_configuration(cli_overide: Option<PathBuf>) -> Result<ApplicationConfig, eyre::Report> {
	// In order, do  a configuration file specified via cli, XDG_CONFIG_HOME, the usual location for system wide configuration(/etc/odilia/config.toml)
	// If XDG_CONFIG_HOME based configuration wasn't found, create one by combining default values with the system provided ones, if available, for the user to alter, for the next run of odilia
	//default configuration first, because that doesn't affect the priority outlined above
	let figment = Figment::from(Serialized::defaults(ApplicationConfig::default()));
	//cli override, if applicable
	let figment =
		if let Some(path) = cli_overide { figment.join(Toml::file(path)) } else { figment };
	//create a config.toml file in `XDG_CONFIG_HOME`, to make it possible for the user to edit the default values, if it doesn't exist already
	let xdg_dirs = xdg::BaseDirectories::with_prefix("odilia").expect(
			"unable to find the odilia config directory according to the xdg dirs specification",
		);

	let config_path = xdg_dirs
		.place_config_file("config.toml")
		.expect("unable to place configuration file. Maybe your system is readonly?");

	let figment = figment
		//next, the configuration system wide, in /etc/odilia/config.toml
		.admerge(Toml::file("/etc/odilia/config.toml"))
		//finally, the xdg configuration
		.admerge(Toml::file(&config_path));
	//realise the configuration and freeze it into place
	let config: ApplicationConfig = figment.extract()?;
	if !config_path.exists() {
		let toml = toml::to_string(&config)?;
		fs::write(&config_path, toml).expect("Unable to create default config file.");
	}
	Ok(config)
}
