use std::{process::Child, time::Duration};

use odilia_common::{
	command::{CaretPos, Focus, Quit, Speak},
	errors::OdiliaError,
};
use ssip::Request;

use crate::state::{
	AccessibleHistory, ChildrenPids, Command, CurrentCaretPos, ShutdownToken, Speech,
};

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
pub async fn quit_cmd(
	Command(Quit): Command<Quit>,
	ShutdownToken(token): ShutdownToken,
	ChildrenPids(pids): ChildrenPids,
) -> Result<(), OdiliaError> {
	let timeout_duration = Duration::from_secs(5); //todo: perhaps take this from the configuration file at some point
	tracing::debug!("Asking all processes to stop.");
	pids.lock()
		.expect("Unable to lock mutex!")
		.iter_mut()
		.try_for_each(Child::kill)
		.expect("Unable to kill child processes");
	tracing::debug!("cancelling all tokens");
	token.cancel();
	tracing::debug!(?timeout_duration, "waiting for all tasks to finish");
	Ok(())
}
