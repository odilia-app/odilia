mod cache;
mod objects;

use std::{sync::Arc};

use futures::stream::StreamExt;
use tokio::sync::{
	broadcast,
	mpsc::{Receiver, Sender},
};

use crate::state::ScreenReaderState;
use crate::traits::{
	IntoOdiliaCommands, Command, IntoStateView, IntoMutableStateView,
};

use atspi_common::events::{Event, CacheEvents};

use odilia_common::{
	errors::OdiliaError,
	events::{ScreenReaderEvent},
	commands::OdiliaCommand,
};


/*
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
*/

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
//			    Some(ScreenReaderEvent::StructuralNavigation(dir, role)) => {
//				 if let Err(e) = structural_navigation(&state, dir, role).await {
//				    tracing::debug!(error = %e, "There was an error with the structural navigation call.");
//				} else {
//					tracing::debug!("Structural navigation successful!");
//				}
//			    },
			    Some(ScreenReaderEvent::StopSpeech) => {
			      tracing::debug!("Stopping speech!");
			      let _: bool = state.stop_speech().await;
			    },
			    Some(ScreenReaderEvent::ChangeMode(new_sr_mode)) => {
						tracing::debug!("Changing mode to {:?}", new_sr_mode);
						let mut sr_mode = state.mode.write().await;
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
	let mut event_id: u64 = 0;
	loop {
		tokio::select! {
			event = rx.recv() => {
			    match event {
				Some(good_event) => {
		let state_arc = Arc::clone(&state);
		tokio::task::spawn(
		  dispatch_wrapper(state_arc, good_event, event_id)
		);
		event_id += 1;
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

async fn dispatch_wrapper(state: Arc<ScreenReaderState>, good_event: Event, event_id: u64) {
  match dispatch(&state, good_event, event_id).await {
    Err(e) => {
      tracing::error!(error = %e, "Could not handle event");
    },
    Ok(events) => {
      let _ = state.apply_all(events).await;
      tracing::debug!("Event handled without error");
    },
	}
}

async fn handle_command<C: IntoMutableStateView + Command + std::fmt::Debug>(command: C, state: &ScreenReaderState, event_id: u64) -> Result<(), OdiliaError> {
	tracing::trace!("Command received ({event_id}): {:?}", command);
	let mutable_state = command.create_view(&state).await?;
	tracing::trace!("Mutable state grab successful ({event_id})");
	Ok(command.execute(mutable_state).await?)
}

async fn handle_event<E: IntoStateView + IntoOdiliaCommands + std::fmt::Debug>(event: E, state: &ScreenReaderState, event_id: u64) -> Result<Vec<OdiliaCommand>, OdiliaError> {
	tracing::trace!("Event received ({event_id}): {:?}", event);
	let state_view = <E as IntoStateView>::create_view(&event, state).await?;
	tracing::trace!("Read-only state grab successful ({event_id})");
	Ok(<E as IntoOdiliaCommands>::commands(&event, &state_view).await?)
}

async fn dispatch(state: &ScreenReaderState, event: Event, event_id: u64) -> eyre::Result<Vec<OdiliaCommand>> {
	// Dispatch based on interface
	Ok(match event {
		Event::Cache(CacheEvents::Add(add)) => {
			let commands = add.commands(&()).await?;
			commands
		},
		other_event => {
			tracing::debug!(
				"Ignoring event with unknown interface: {:#?}",
				other_event
			);
			vec![]
		}
	})
	//let accessible_id = state.new_accessible(&interface).await?.path().try_into()?;
	//state.update_accessible(accessible_id).await;
	//state.event_history_update(event).await;
	//Ok(events)
}

#[cfg(test)]
pub mod dispatch_tests {
	use crate::ScreenReaderState;
	use tokio::sync::mpsc::channel;

	#[tokio::test]
	async fn test_full_cache() {
		let state = generate_state().await;
		assert_eq!(state.cache.by_id.read().await.len(), 14_738);
	}

	pub async fn generate_state() -> ScreenReaderState {
		let (send, _recv) = channel(32);
		let cache = serde_json::from_str(include_str!("wcag_cache_items.json")).unwrap();
		let state = ScreenReaderState::new(send.into()).await.unwrap();
		state.cache.add_all(cache).await.unwrap();
		state
	}
}
