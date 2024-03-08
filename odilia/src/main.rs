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

use std::{process::exit, sync::Arc};

use crate::cli::Args;
use crate::state::ScreenReaderState;
use clap::Parser;
use eyre::WrapErr;
use futures::{future::FutureExt, StreamExt};
use odilia_input::sr_event_receiver;
use odilia_notify::listen_to_dbus_notifications;
use ssip_client_async::Priority;
use tokio::{
	signal::unix::{signal, SignalKind},
	sync::mpsc,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use atspi_common::events::{document, object};
async fn notifications_monitor(
	state: Arc<ScreenReaderState>,
	shutdown: CancellationToken,
) -> eyre::Result<()> {
	let mut stream = listen_to_dbus_notifications().await?;
	loop {
		tokio::select! {
		    Some(notification) = stream.next() => {
		      let notification_message =
			format!("new notification: {}, {}.", notification.title, notification.body);
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
async fn sigterm_signal_watcher(
	token: CancellationToken,
	tracker: TaskTracker,
) -> eyre::Result<()> {
	let mut c = signal(SignalKind::interrupt())?;
	tracing::debug!("Watching for Ctrl+C");
	c.recv().await;
	tracing::debug!("Asking all processes to stop.");
	tracing::debug!("cancelling all tokens");
	token.cancel();
	tracing::debug!("waiting for all tasks to finish");
	tracker.close();
	tracing::debug!("All listeners have stopped.");
	tracing::debug!("Goodbye, Odilia!");
	Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
	let args = Args::parse();
	logging::init();

	//initialize the primary token for task cancelation
	let token = CancellationToken::new();

	//initialize a task tracker, which will allow us to wait for all tasks to finish
	let tracker = TaskTracker::new();

	// Make sure applications with dynamic accessibility supprt do expose their AT-SPI2 interfaces.
	if let Err(e) = atspi_connection::set_session_accessibility(true).await {
		tracing::debug!("Could not set AT-SPI2 IsEnabled property because: {}", e);
	}
	let (sr_event_tx, sr_event_rx) = mpsc::channel(128);
	// this channel must NEVER fill up; it will cause the thread receiving events to deadlock due to a zbus design choice.
	// If you need to make it bigger, then make it bigger, but do NOT let it ever fill up.
	let (atspi_event_tx, atspi_event_rx) = mpsc::channel(128);
	// this is the chanel which handles all SSIP commands. If SSIP is not allowed to operate on a separate task, then wdaiting for the receiving message can block other long-running operations like structural navigation.
	// Although in the future, this may possibly be remidied through a proper cache, I think it still makes sense to separate SSIP's IO operations to a separate task.
	// Like the channel above, it is very important that this is *never* full, since it can cause deadlocking if the other task sending the request is working with zbus.
	let (ssip_req_tx, ssip_req_rx) = mpsc::channel::<ssip_client_async::Request>(128);
	// Initialize state
	let state = Arc::new(ScreenReaderState::new(ssip_req_tx, args.config.as_deref()).await?);
	let ssip = odilia_tts::create_ssip_client().await?;

	if state.say(Priority::Message, "Welcome to Odilia!".to_string()).await {
		tracing::debug!("Welcome message spoken.");
	} else {
		tracing::debug!("Welcome message failed. Odilia is not able to continue in this state. Existing now.");
		let _: bool = state.close_speech().await;
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
