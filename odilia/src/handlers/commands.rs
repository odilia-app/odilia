use odilia_common::{
	command::{CaretPos, Focus, Speak, TryIntoCommands},
	errors::OdiliaError,
	events::{StopSpeech, StructuralNavigation},
};
use ssip::{Priority, Request};

use crate::state::{AccessibleHistory, Command, CurrentCaretPos, InputEvent, Speech};

#[tracing::instrument(ret, err, level = "debug")]
pub async fn speak(
	Command(Speak(text, priority)): Command<Speak>,
	Speech(ssip): Speech,
) -> Result<(), odilia_common::errors::OdiliaError> {
	ssip.send(Request::SetPriority(priority)).await?;
	ssip.send(Request::Speak).await?;
	ssip.send(Request::SendLines(Vec::from([text]))).await?;
	Ok(())
}

// TODO: move all cache logic behind the cache
//#[tracing::instrument(ret, err)]
//pub async fn set_state(
//	Command(SetState { item, state, enabled }): Command<SetState>,
//	Cache(cache): Cache,
//) -> Result<(), OdiliaError> {
//	cache.modify_item(&item, |it| {
//		if enabled {
//			it.states.insert(state);
//		} else {
//			it.states.remove(state);
//		}
//	})?;
//	Ok(())
//}

#[tracing::instrument(ret, err)]
pub async fn new_focused_item(
	Command(Focus(new_focus)): Command<Focus>,
	AccessibleHistory(old_focus): AccessibleHistory,
) -> Result<(), OdiliaError> {
	let _ = old_focus.lock()?.push(new_focus);
	Ok(())
}

#[tracing::instrument(ret, err)]
pub async fn new_caret_pos(
	Command(CaretPos(new_pos)): Command<CaretPos>,
	CurrentCaretPos(pos): CurrentCaretPos,
) -> Result<(), OdiliaError> {
	pos.store(new_pos, core::sync::atomic::Ordering::Relaxed);
	Ok(())
}

#[tracing::instrument(ret)]
pub async fn stop_speech(InputEvent(_): InputEvent<StopSpeech>) -> impl TryIntoCommands {
	(Priority::Text, "Stop speech")
}

#[tracing::instrument(ret)]
pub async fn structural_nav(
	InputEvent(sn): InputEvent<StructuralNavigation>,
) -> impl TryIntoCommands {
	(Priority::Text, format!("Navigate to {}, {:?}", sn.1, sn.0))
}
