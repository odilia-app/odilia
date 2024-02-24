mod cache;
mod document;
mod object;

use std::{collections::HashMap, sync::Arc};

use futures::stream::StreamExt;
use tokio::sync::{
	broadcast,
	mpsc::{Receiver, Sender},
};

use crate::state::ScreenReaderState;
use atspi_client::{accessible_ext::AccessibleExt, convertable::Convertable};
use atspi_common::events::Event;
use atspi_common::{InterfaceSet, MatchType, MatcherArgs, Role, ScrollType};
use odilia_common::{
	events::{Direction, ScreenReaderEvent},
	result::OdiliaResult,
};
use ssip_client_async::Priority;

pub async fn structural_navigation(
	state: &ScreenReaderState,
	dir: Direction,
	role: Role,
) -> OdiliaResult<bool> {
	tracing::debug!("Structural nav call begins!");
	let curr = match state.history_item(0).await {
		Some(acc) => acc.into_accessible(state.atspi.connection()).await?,
		None => return Ok(false),
	};
	let roles = vec![role];
	let attributes = HashMap::new();
	let interfaces = InterfaceSet::empty();
	let mt: MatcherArgs = (
		roles,
		MatchType::Invalid,
		attributes,
		MatchType::Invalid,
		interfaces,
		MatchType::Invalid,
	);
	if let Some(next) = curr
		.get_next(&mt, dir == Direction::Backward, &mut Vec::new())
		.await?
	{
		let comp = next.to_component().await?;
		let texti = next.to_text().await?;
		let curr_prim = curr.try_into()?;
		let _: bool = comp.grab_focus().await?;
		comp.scroll_to(ScrollType::TopLeft).await?;
		state.update_accessible(curr_prim).await;
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

pub async fn sr_event(
	state: Arc<ScreenReaderState>,
	sr_events: &mut Receiver<ScreenReaderEvent>,
	shutdown_rx: &mut broadcast::Receiver<i32>,
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
			      let _: bool = state.stop_speech().await;
			    },
			    Some(ScreenReaderEvent::ChangeMode(new_sr_mode)) => {
						tracing::debug!("Changing mode to {:?}", new_sr_mode);
						let mut sr_mode = state.mode.lock().await;
						*sr_mode = new_sr_mode;
			    }
			    _ => { continue; }
			};
			continue;
		    }
		    _ = shutdown_rx.recv() => {
			tracing::debug!("sr_event cancelled");
			break;
		    }
		}
	}
	Ok(())
}

//#[tracing::instrument(level = "debug"i, skip(state))]
pub async fn receive(
	state: Arc<ScreenReaderState>,
	tx: Sender<Event>,
	shutdown_rx: &mut broadcast::Receiver<i32>,
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
		    _ = shutdown_rx.recv() => {
			tracing::debug!("receive function is done");
			break;
		    }
		}
	}
}

//#[tracing::instrument(level = "debug")]
pub async fn process(
	state: Arc<ScreenReaderState>,
	rx: &mut Receiver<Event>,
	shutdown_rx: &mut broadcast::Receiver<i32>,
) {
	loop {
		tokio::select! {
			event = rx.recv() => {
			    match event {
				Some(good_event) => {
		let state_arc = Arc::clone(&state);
		tokio::task::spawn(
		  dispatch_wrapper(state_arc, good_event)
		);
				},
				None => {
				    tracing::debug!("Event was none.");
				}
			    };
			    continue;
			}
			_ = shutdown_rx.recv() => {
			    tracing::debug!("process function is done");
			    break;
			}
		    }
	}
}

async fn dispatch_wrapper(state: Arc<ScreenReaderState>, good_event: Event) {
	if let Err(e) = dispatch(&state, good_event).await {
		tracing::error!(error = %e, "Could not handle event");
	} else {
		tracing::debug!("Event handled without error");
	}
}

async fn dispatch(state: &ScreenReaderState, event: Event) -> eyre::Result<()> {
	// Dispatch based on interface
	match &event {
		Event::Object(object_event) => {
			object::dispatch(state, object_event).await?;
		}
		Event::Document(document_event) => {
			document::dispatch(state, document_event).await?;
		}
		Event::Cache(cache_event) => cache::dispatch(state, cache_event).await?,
		other_event => {
			tracing::debug!(
				"Ignoring event with unknown interface: {:#?}",
				other_event
			);
		}
	}
	//let accessible_id = state.new_accessible(&interface).await?.path().try_into()?;
	//state.update_accessible(accessible_id).await;
	state.event_history_update(event).await;
	Ok(())
}

#[cfg(test)]
pub mod dispatch_tests {
	use crate::ScreenReaderState;
	use eyre::Context;
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
		let state = ScreenReaderState::new(send, None)
			.await
			.context("unable to realise screenreader state")?;
		state.cache
			.add_all(cache)
			.context("unable to add cache to the system")?;
		Ok(state)
	}
}
