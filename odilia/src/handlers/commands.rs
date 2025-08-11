use std::{process::Child, time::Duration};

use atspi::{
	proxy::{accessible::AccessibleProxy, proxy_ext::ProxyExt},
	ScrollType,
};
use odilia_common::{
	command::{CaretPos, Focus, Move, Quit, Speak},
	errors::OdiliaError,
};
use ssip::Request;
use tracing::{error, trace};
use zbus::proxy::CacheProperties;

use crate::state::{
	AccessibleHistory, ChildrenPids, Command, Connection, CurrentCaretPos, ShutdownToken,
	Speech,
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

// NOTE: DO NOT UPDATE STATE HERE!
//
// This is for when Odilia has requested a change of focus via AT-SPI.
// All state management, speaking, etc. will happen once the focus event makes its way back through
// the handlers.
#[tracing::instrument(ret, err)]
pub async fn move_focus(
	Command(Move(new_focus)): Command<Move>,
	Connection(con): Connection,
) -> Result<(), OdiliaError> {
	let proxies = AccessibleProxy::builder(&con)
		.cache_properties(CacheProperties::No)
		.path(new_focus.id.clone())?
		.destination(new_focus.sender.clone())?
		.build()
		.await?
		.proxies()
		.await?;
	let live_item = proxies.component().await?;
	if !live_item.grab_focus().await? {
		error!("Unable to get focus on new item: {new_focus:?}");
	}
	if !live_item.scroll_to(ScrollType::TopLeft).await? {
		error!("Unable to scroll to new item; this is usually because the item is invalid!");
	}
	let Ok(text_item) = proxies.text().await else {
		trace!("New focus is not a text item");
		return Ok(());
	};
	text_item.set_caret_offset(0).await?;
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
