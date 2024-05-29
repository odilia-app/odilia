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
mod events;
mod logging;
mod state;

use std::{fs, path::PathBuf, process::exit, sync::Arc, time::Duration};

use crate::cli::Args;
use crate::state::ScreenReaderState;
use clap::Parser;
use eyre::WrapErr;
use figment::{
	providers::{Format, Serialized, Toml},
	Figment,
};
use futures::{future::FutureExt, StreamExt};
use odilia_common::settings::ApplicationConfig;
use odilia_input::sr_event_receiver;
use odilia_notify::listen_to_dbus_notifications;
use ssip_client_async::Priority;
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

#[tracing::instrument]
#[tokio::main(flavor = "current_thread")]
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
	let (sr_event_tx, sr_event_rx) = mpsc::channel(128);
	// this channel must NEVER fill up; it will cause the thread receiving events to deadlock due to a zbus design choice.
	// If you need to make it bigger, then make it bigger, but do NOT let it ever fill up.
	let (atspi_event_tx, atspi_event_rx) = mpsc::channel(128);
	// this is the channel which handles all SSIP commands. If SSIP is not allowed to operate on a separate task, then waiting for the receiving message can block other long-running operations like structural navigation.
	// Although in the future, this may possibly be resolved through a proper cache, I think it still makes sense to separate SSIP's IO operations to a separate task.
	// Like the channel above, it is very important that this is *never* full, since it can cause deadlocking if the other task sending the request is working with zbus.
	let (ssip_req_tx, ssip_req_rx) = mpsc::channel::<ssip_client_async::Request>(128);
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

	let ssip_event_receiver =
		odilia_tts::handle_ssip_commands(ssip, ssip_req_rx, token.clone())
			.map(|r| r.wrap_err("Could no process SSIP request"));
	let atspi_event_receiver =
		events::receive(Arc::clone(&state), atspi_event_tx, token.clone())
			.map(|()| Ok::<_, eyre::Report>(()));
	let atspi_event_processor =
		events::process(Arc::clone(&state), atspi_event_rx, token.clone())
			.map(|()| Ok::<_, eyre::Report>(()));
	let odilia_event_receiver = sr_event_receiver(sr_event_tx, token.clone())
		.map(|r| r.wrap_err("Could not process Odilia events"));
	let odilia_event_processor =
		events::sr_event(Arc::clone(&state), sr_event_rx, token.clone())
			.map(|r| r.wrap_err("Could not process Odilia event"));
	let notification_task = notifications_monitor(Arc::clone(&state), token.clone())
		.map(|r| r.wrap_err("Could not process signal shutdown."));

	tracker.spawn(atspi_event_receiver);
	tracker.spawn(atspi_event_processor);
	tracker.spawn(odilia_event_receiver);
	tracker.spawn(odilia_event_processor);
	tracker.spawn(ssip_event_receiver);
	tracker.spawn(notification_task);
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
