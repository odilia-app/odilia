#![deny(
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	clippy::map_unwrap_or,
	clippy::unwrap_used,
	unsafe_code,
	clippy::print_stdout,
	clippy::print_stderr
)]

mod cli;
mod handlers;
mod logging;
mod state;
mod tower;
use std::{
	env,
	path::{Path, PathBuf},
	process::{exit, Child, Command as ProcCommand},
	sync::Arc,
	time::Duration,
};

use async_channel::bounded;
use async_executor::StaticExecutor;
use async_signal::{Signal, Signals};
use atspi::events::{document, object};
use futures_concurrency::future::{Join, TryJoin};
use futures_lite::{
	future::{block_on, FutureExt},
	stream::StreamExt,
};
use futures_util::FutureExt as FatExt;
use handlers::{
	caret_moved, caret_moved_update_state, change_mode, doc_loaded, focused, new_caret_pos,
	new_focused_item, speak, state_set, stop_speech, structural_nav,
};
use odilia_cache::{cache_handler_task, Cache, CacheActor};
use odilia_common::{
	command::TryIntoCommands,
	errors::OdiliaError,
	events::ScreenReaderEvent,
	settings::{ApplicationConfig, InputMethod},
};
use odilia_notify::listen_to_dbus_notifications;
use smol_cancellation_token::CancellationToken;
use ssip::Priority;
use tracing::Instrument;

use crate::{
	cli::Args,
	state::{InputEvent, ScreenReaderState},
	tower::Handlers,
};

fn find_it<P>(exe_name: P) -> Option<PathBuf>
where
	P: AsRef<Path>,
{
	find_it_in(exe_name, [""].into_iter())
}
fn find_it_in<P, I, Item>(exe_name: P, paths: I) -> Option<PathBuf>
where
	P: AsRef<Path>,
	I: Iterator<Item = Item>,
	Item: Into<std::ffi::OsString>,
{
	let os_strings = paths
		.map(Into::into)
		.chain([env::var_os("PATH")?])
		.collect::<Vec<std::ffi::OsString>>();
	os_strings.iter().find_map(|paths| {
		env::split_paths(&paths).find_map(|dir| {
			let full_path = dir.join(&exe_name);
			if full_path.is_file() {
				Some(full_path)
			} else {
				None
			}
		})
	})
}

async fn or_cancel<F>(f: F, token: &CancellationToken) -> Result<F::Output, std::io::Error>
where
	F: std::future::Future,
{
	token.cancelled()
		.map(|()| Err(std::io::ErrorKind::TimedOut.into()))
		.or(f.map(Ok))
		.await
}

/// Try to spawn the `odilia-input-server-*` binary.
#[tracing::instrument(skip(input), err)]
fn try_spawn_input_server(input: &InputMethod) -> Result<Child, OdiliaError> {
	let bin_name = format!(
		"{}-{}",
		"odilia-input-server",
		match input {
			InputMethod::Keyboard => "keyboard",
			InputMethod::Custom(s) => &s,
		}
	);
	if find_it(&bin_name).is_none() {
		tracing::info!("Unable to find {bin_name} in $PATH; trying hardcoded paths.");
	}
	let found = find_it_in(
		bin_name.clone(),
		["./target/debug/", "./target/release/", "../target/debug/", "../target/release/"]
			.into_iter(),
	);
	let child = match found {
		None => {
			return Err(format!("Unable to find {bin_name} in $PATH or any hardcoded paths (for development). This means Odilia is uncontrollable by any mechanism!").into());
		}
		Some(path) => {
			tracing::info!("Input server path: {:?}", path);
			ProcCommand::new(path).spawn()?
		}
	};
	Ok(child)
}

#[tracing::instrument(skip_all, err)]
async fn notifications_monitor(
	state: Arc<ScreenReaderState>,
	shutdown: CancellationToken,
) -> Result<(), OdiliaError> {
	let mut stream = listen_to_dbus_notifications()
		.instrument(tracing::info_span!("creating notification listener"))
		.await?;
	loop {
		let maybe_timeout = or_cancel(stream.next(), &shutdown).await;
		let Ok(maybe_notification) = maybe_timeout else {
			tracing::debug!("Shutting down notification task.");
			break;
		};
		let Some(notification) = maybe_notification else {
			continue;
		};
		let notification_message = format!(
			"new notification: {}, {}, {}.",
			notification.app_name, notification.title, notification.body
		);
		state.say(Priority::Important, notification_message).await;
	}
	Ok(())
}
#[tracing::instrument(skip_all, err)]
async fn sigterm_signal_watcher(
	token: CancellationToken,
	state: Arc<ScreenReaderState>,
) -> Result<(), OdiliaError> {
	let timeout_duration = Duration::from_secs(5); //todo: perhaps take this from the configuration file at some point
	let mut signals = Signals::new([Signal::Int])?;
	signals.next()
		.instrument(tracing::debug_span!("Watching for Ctrl+C"))
		.await;
	tracing::debug!("Asking all processes to stop.");
	(*state.children_pids.lock().expect("Unable to lock mutex!"))
		.iter_mut()
		.try_for_each(Child::kill)
		.expect("Able to kill child processes");
	tracing::debug!("cancelling all tokens");
	token.cancel();
	tracing::debug!(?timeout_duration, "waiting for all tasks to finish");
	Ok(())
}

static EXECUTOR: StaticExecutor = StaticExecutor::new();

fn main() -> Result<(), OdiliaError> {
	block_on(EXECUTOR.run(async_main()))
}

async fn async_main() -> Result<(), OdiliaError> {
	let ex = &EXECUTOR;
	let args = Args::from_cli_args()?;

	//initialize the primary token for task cancelation
	let token = CancellationToken::new();

	//initializing configuration
	let config = load_configuration(args.config)?;
	//initialize logging, with the provided config
	logging::init(&config)?;

	tracing::info!(?config, "this configuration was used to prepair odilia");

	// Make sure applications with dynamic accessibility support do expose their AT-SPI2 interfaces.
	if let Err(e) = atspi::connection::set_session_accessibility(true)
		.instrument(tracing::info_span!("setting accessibility enabled flag"))
		.await
	{
		tracing::error!("Could not set AT-SPI2 IsEnabled property because: {}", e);
	}
	// this is the channel which handles all SSIP commands. If SSIP is not allowed to operate on a separate task, then waiting for the receiving message can block other long-running operations like structural navigation.
	// Although in the future, this may possibly be resolved through a proper cache, I think it still makes sense to separate SSIP's IO operations to a separate task.
	//  it is very important that this is *never* full, since it can cause deadlocking if the other task sending the request is working with zbus.
	let (ssip_req_tx, ssip_req_rx) = bounded::<ssip_client_async::Request>(128);
	let (ev_tx, ev_rx) = bounded::<Result<atspi::Event, atspi::AtspiError>>(10_000);
	let (input_tx, input_rx) = bounded::<ScreenReaderEvent>(255);
	// Initialize state
	// lots of space for caching just in case...
	let (cache_tx, cache_rx) = bounded(4096);
	let cache = CacheActor::new(cache_tx);
	let state = Arc::new(ScreenReaderState::new(ssip_req_tx, config, cache).await?);
	let ssip = odilia_tts::create_ssip_client().await?;

	if state.say(Priority::Message, "Welcome to Odilia!".to_string()).await {
		tracing::debug!("Welcome message spoken.");
	} else {
		tracing::error!("Welcome message failed. Odilia is not able to continue in this state. Exiting now.");
		state.close_speech().await;
		exit(1);
	}

	// Register events
	(
		state.register_event::<object::StateChangedEvent>(),
		state.register_event::<object::TextCaretMovedEvent>(),
		state.register_event::<document::LoadCompleteEvent>(),
		//TODO: we don't handle these yet!
		// state.add_cache_match_rule(),
	)
		.try_join()
		.await?;

	// load handlers
	let handlers = Handlers::new(state.clone())
		.command_listener(speak)
		.command_listener(new_focused_item)
		.command_listener(new_caret_pos)
		//.command_listener(set_state)
		.atspi_listener(doc_loaded)
		.atspi_listener(caret_moved_update_state)
		.atspi_listener(caret_moved)
		.atspi_listener(focused)
		.atspi_listener(state_set)
		.input_listener(stop_speech)
		.input_listener(change_mode)
		.input_listener(structural_nav);

	let ssip_event_receiver =
		odilia_tts::handle_ssip_commands(ssip, ssip_req_rx, token.clone());
	let notification_task = notifications_monitor(Arc::clone(&state), token.clone());
	let mut stream = state.atspi.event_stream();
	// There is a reason we are not reading from the event stream directly.
	// This `MessageStream` can only store 64 events in its buffer.
	// And, even if it could store more (it can via options), `zbus` specifically states that:
	// You must ensure a MessageStream is continuously polled or you will experience hangs.
	// So, we continually poll it here, then receive it on the other end.
	// Additioanlly, since sending is not async, but simply errors when there is an issue, this will
	// help us avoid hangs.
	let token_clone = token.clone();
	let event_send_task = async move {
		std::pin::pin!(&mut stream);
		loop {
			let maybe = or_cancel(stream.next(), &token_clone).await;
			let Ok(maybe_ev) = maybe else {
				return;
			};
			if let Some(ev) = maybe_ev {
				if let Err(e) = ev_tx.try_send(ev) {
					tracing::error!(
						"Error sending event across channel! {e:?}"
					);
				}
			}
		}
	};
	let atspi_handlers_task = handlers.clone().atspi_handler(ev_rx, token.clone());
	let listener = odilia_input::setup_input_server()
		.await
		.expect("We should be able to set up input server; without it, Odilia cannot be controlled via input methods");
	let input_task = odilia_input::sr_event_receiver(listener, input_tx, token.clone())
		.for_each(|fut| {
			ex.spawn(fut).detach();
		});
	let input_handler = handlers.input_handler(input_rx, token.clone());
	let child = try_spawn_input_server(&state.config.input.method)?;
	state.add_child_proc(child).expect("Able to add child to process!");

	let cache = Cache::new(state.connection().clone());
	let ct = token.clone();
	let cache_handler = blocking::unblock(|| block_on(cache_handler_task(cache_rx, ct, cache)));

	let joined_tasks = (
		ssip_event_receiver,
		notification_task,
		atspi_handlers_task,
		event_send_task,
		input_task,
		input_handler,
		cache_handler,
	)
		.join();
	ex.spawn(sigterm_signal_watcher(token, Arc::clone(&state))).detach();
	let _ = joined_tasks.await;
	tracing::debug!("All listeners have stopped.");
	tracing::debug!("Goodbye, Odilia!");

	Ok(())
}

fn load_configuration(cli_overide: Option<PathBuf>) -> Result<ApplicationConfig, OdiliaError> {
	// In order, do  a configuration file specified via cli, XDG_CONFIG_HOME, the usual location for system wide configuration(/etc/odilia/config.toml)
	// If XDG_CONFIG_HOME based configuration wasn't found, create one by combining default values with the system provided ones, if available, for the user to alter, for the next run of odilia
	//default configuration first, because that doesn't affect the priority outlined above
	let xdg_dirs = xdg::BaseDirectories::with_prefix("odilia").expect(
			"unable to find the odilia config directory according to the xdg dirs specification",
		);

	let config_path = xdg_dirs
		.place_config_file("config.toml")
		.expect("unable to place configuration file. Maybe your system is readonly?");

	let mut config = config::Config::builder()
		.add_source(config::Config::try_from(&ApplicationConfig::default())?)
		// env vars
		.add_source(config::Environment::with_prefix("ODILIA"))
		//next, the configuration system wide, in /etc/odilia/config.toml
		.add_source(config::File::with_name("/etc/odilia/config"))
		//finally, the xdg configuration
		.add_source(config::File::with_name(
			config_path.to_str().expect("Valid UTF-8 path"),
		));
	if let Some(path) = cli_overide {
		// if a path overide was given, use that
		config = config.add_source(config::File::with_name(
			path.to_str().expect("Valid UTF-8 path"),
		));
	}
	Ok(config.build()?.try_deserialize()?)
}
