mod cache;
mod document;

use std::sync::Arc;

use futures::stream::StreamExt;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

use crate::state::ScreenReaderState;
use atspi_common::events::Event;
use atspi_common::{Role, ScrollType};
use odilia_cache::AccessibleExt;
use odilia_cache::Convertable;
use odilia_common::{
	events::{Direction, ScreenReaderEvent},
	result::OdiliaResult,
};
use ssip_client_async::Priority;

#[tracing::instrument(level = "debug", skip_all, ret, err)]
pub async fn structural_navigation(
	state: &ScreenReaderState,
	dir: Direction,
	role: Role,
) -> OdiliaResult<bool> {
	tracing::debug!("Structural nav call begins!");
	let curr = match state.history_item(0) {
		Some(acc) => acc.into_accessible(state.atspi.connection()).await?,
		None => return Ok(false),
	};
	if let Some(next) = curr.get_next(role, dir == Direction::Backward).await? {
		let comp = next.to_component().await?;
		let texti = next.to_text().await?;
		let curr_prim = curr.try_into()?;
		let _: bool = comp.grab_focus().await?;
		comp.scroll_to(ScrollType::TopLeft).await?;
		state.update_accessible(curr_prim);
		let _: bool = texti.set_caret_offset(0).await?;
		let role = next.get_role().await?;
		let len = texti.character_count().await?;
		let text = texti.get_text(0, len).await?;
		// saying awaits until it is done talking; you may want to spawn a task
		state.say(Priority::Text, format!("{text}, {role}")).await;
		Ok(true)
	} else {
		state.say(Priority::Text, format!("No more {role}s")).await;
		Ok(true)
	}
}

#[tracing::instrument(level = "debug", skip(state), ret, err)]
pub async fn sr_event(
	state: Arc<ScreenReaderState>,
	mut sr_events: Receiver<ScreenReaderEvent>,
	shutdown: CancellationToken,
) -> eyre::Result<()> {
	loop {
		tokio::select! {
			sr_event = sr_events.recv() => {
			    tracing::debug!("SR Event received");
			    match sr_event {
				Some(ScreenReaderEvent::StructuralNavigation(dir, role)) => {
				     if let Err(e) = structural_navigation(&state, dir, role).await {
					tracing::debug!(error = %e, "There was an error with the structural navigation call.");
				    } else {
					    tracing::debug!("Structural navigation successful!");
				    }
				},
				Some(ScreenReaderEvent::StopSpeech) => {
				  tracing::debug!("Stopping speech!");
				  state.stop_speech().await;
				},
				Some(ScreenReaderEvent::ChangeMode(new_sr_mode)) => {
						    tracing::debug!("Changing mode to {:?}", new_sr_mode);
						    if let Ok(mut sr_mode) = state.mode.lock() {
		    *sr_mode = new_sr_mode;
		}
				}
				_ => { continue; }
			    };
			    continue;
			}
			() = shutdown.cancelled() => {
			    tracing::debug!("sr_event cancelled");
			    break;
			}
		    }
	}
	Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn receive(
	state: Arc<ScreenReaderState>,
	tx: Sender<Event>,
	shutdown: CancellationToken,
) {
	let events = state.atspi.event_stream();
	tokio::pin!(events);
	loop {
		tokio::select! {
		    event = events.next() => {
			if let Some(Ok(good_event)) = event {
			    if let Err(e) = tx.send(good_event).await {
				tracing::error!(error = %e, "Error sending atspi event");
			    }
			} else {
			    tracing::debug!("Event is either None or an Error variant.");
			}
			continue;
		    }
		    () = shutdown.cancelled() => {
			tracing::debug!("receive function is done");
			break;
		    }
		}
	}
}

#[cfg(test)]
pub mod dispatch_tests {
	use crate::ScreenReaderState;
	use eyre::Context;
	use odilia_common::settings::ApplicationConfig;
	use tokio::sync::mpsc::channel;

	#[tokio::test]
	async fn test_full_cache() -> eyre::Result<()> {
		let state = generate_state().await?;
		assert_eq!(state.cache.by_id.len(), 14_738);
		Ok(())
	}

	pub async fn generate_state() -> eyre::Result<ScreenReaderState> {
		let (send, _recv) = channel(32);
		let cache = serde_json::from_str(include_str!("wcag_cache_items.json"))
			.context("unable to load cache data from json file")?;
		let state = ScreenReaderState::new(send, ApplicationConfig::default())
			.await
			.context("unable to realise screenreader state")?;
		state.cache
			.add_all(cache)
			.context("unable to add cache to the system")?;
		Ok(state)
	}
}
