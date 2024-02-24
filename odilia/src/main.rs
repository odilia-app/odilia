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

use clap::Parser;
use eyre::WrapErr;
use futures::{future::FutureExt, StreamExt};
use tokio::{
	signal::unix::{signal, SignalKind},
	sync::broadcast::{self, error::SendError},
	sync::mpsc,
};

use crate::cli::Args;
use crate::state::ScreenReaderState;
use odilia_input::sr_event_receiver;
use odilia_notify::listen_to_dbus_notifications;
use ssip_client_async::Priority;

use atspi_common::events::{document, object};
async fn notifications_monitor(state: Arc<ScreenReaderState>) -> eyre::Result<()> {
	let mut stream = listen_to_dbus_notifications().await?;
	while let Some(notification) = stream.next().await {
		let notification_message =
			format!("new notification: {}, {}.", notification.title, notification.body);
		state.say(Priority::Important, notification_message).await;
	}
	Ok(())
}
async fn sigterm_signal_watcher(shutdown_tx: broadcast::Sender<i32>) -> eyre::Result<()> {
	let mut c = signal(SignalKind::interrupt())?;
	tracing::debug!("Watching for Ctrl+C");
	c.recv().await;
	tracing::debug!("Asking all processes to stop.");
	let _: Result<usize, SendError<i32>> = shutdown_tx.send(0);
	Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
	let args = Args::parse();

	logging::init();
	// Make sure applications with dynamic accessibility supprt do expose their AT-SPI2 interfaces.
	if let Err(e) = atspi_connection::set_session_accessibility(true).await {
		tracing::debug!("Could not set AT-SPI2 IsEnabled property because: {}", e);
	}
	let (shutdown_tx, _) = broadcast::channel(1);
	let (sr_event_tx, mut sr_event_rx) = mpsc::channel(128);
	// this channel must NEVER fill up; it will cause the thread receiving events to deadlock due to a zbus design choice.
	// If you need to make it bigger, then make it bigger, but do NOT let it ever fill up.
	let (atspi_event_tx, mut atspi_event_rx) = mpsc::channel(128);
	// this is the chanel which handles all SSIP commands. If SSIP is not allowed to operate on a separate task, then wdaiting for the receiving message can block other long-running operations like structural navigation.
	// Although in the future, this may possibly be remidied through a proper cache, I think it still makes sense to separate SSIP's IO operations to a separate task.
	// Like the channel above, it is very important that this is *never* full, since it can cause deadlocking if the other task sending the request is working with zbus.
	let (ssip_req_tx, ssip_req_rx) = mpsc::channel::<ssip_client_async::tokio::Request>(128);
	// Initialize state
	let state = Arc::new(ScreenReaderState::new(ssip_req_tx, args.config.as_deref()).await?);
	let mut ssip = odilia_tts::create_ssip_client().await?;

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

	let mut shutdown_rx_ssip_recv = shutdown_tx.subscribe();
	let ssip_event_receiver = odilia_tts::handle_ssip_commands(
		&mut ssip,
		ssip_req_rx,
		&mut shutdown_rx_ssip_recv,
	)
	.map(|r| r.wrap_err("Could no process SSIP request"));
	let mut shutdown_rx_atspi_recv = shutdown_tx.subscribe();
	let atspi_event_receiver =
		events::receive(Arc::clone(&state), atspi_event_tx, &mut shutdown_rx_atspi_recv)
			.map(|()| Ok::<_, eyre::Report>(()));
	let mut shutdown_rx_atspi_proc_recv = shutdown_tx.subscribe();
	let atspi_event_processor = events::process(
		Arc::clone(&state),
		&mut atspi_event_rx,
		&mut shutdown_rx_atspi_proc_recv,
	)
	.map(|()| Ok::<_, eyre::Report>(()));
	let mut shutdown_rx_odilia_recv = shutdown_tx.subscribe();
	let odilia_event_receiver = sr_event_receiver(sr_event_tx, &mut shutdown_rx_odilia_recv)
		.map(|r| r.wrap_err("Could not process Odilia events"));
	let mut shutdown_rx_odilia_proc_recv = shutdown_tx.subscribe();
	let odilia_event_processor = events::sr_event(
		Arc::clone(&state),
		&mut sr_event_rx,
		&mut shutdown_rx_odilia_proc_recv,
	)
	.map(|r| r.wrap_err("Could not process Odilia event"));
	let signal_receiver = sigterm_signal_watcher(shutdown_tx)
		.map(|r| r.wrap_err("Could not process signal shutdown."));
	let notification_task = notifications_monitor(Arc::clone(&state))
		.map(|r| r.wrap_err("Could not process signal shutdown."));

	tokio::try_join!(
		signal_receiver,
		atspi_event_receiver,
		atspi_event_processor,
		odilia_event_receiver,
		odilia_event_processor,
		ssip_event_receiver,
		notification_task,
	)?;
	tracing::debug!("All listeners have stopped.");
	tracing::debug!("Goodbye, Odilia!");
	Ok(())
}
