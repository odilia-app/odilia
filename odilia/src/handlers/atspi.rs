use std::{
	cmp::{max, min},
	fmt::Write,
};

use atspi::events::{
	document::LoadCompleteEvent,
	object::{StateChangedEvent, TextCaretMovedEvent},
};
use odilia_cache::LabelledBy;
use odilia_common::{
	command::{CaretPos, Focus, OdiliaCommand, SetState, Speak, TryIntoCommands},
	errors::OdiliaError,
};
use ssip::Priority;

use crate::{
	state::{LastCaretPos, LastFocused},
	tower::{state_changed::Focused, ActiveAppEvent, CacheEvent, EventProp, RelationSet},
};

#[tracing::instrument(ret)]
pub async fn doc_loaded(loaded: ActiveAppEvent<LoadCompleteEvent>) -> impl TryIntoCommands {
	(Priority::Text, "Doc loaded")
}

#[tracing::instrument(ret)]
pub async fn focused(
	state_changed: CacheEvent<Focused>,
	EventProp(relation_set): EventProp<RelationSet<LabelledBy>>,
) -> impl TryIntoCommands {
	//because the current command implementation doesn't allow for multiple speak commands without interrupting the previous utterance, this is more or less an accumulating buffer for that utterance
	let mut utterance_buffer = String::new();
	let item = state_changed.item;
	//does this have a text or a name?
	// in order for the borrow checker to not scream that we move ownership of item.text, therefore making item partially moved, we only take a reference here, because in truth the only thing that we need to know is if the string is empty, because the extending of the buffer will imply a clone anyway
	if let Some(text) = item.text {
		//then just append to the buffer and be done with it
		utterance_buffer += &text;
	} else {
		//then the label can either be the accessible name, the description, or the relations set, aka labeled by another object
		//unfortunately, the or_else function of result doesn't accept async cloasures or cloasures with async blocks, so we can't use lazy loading here at the moment. The performance penalty is minimal however, because this should be in cache anyway
		let label = if let Some(n) = item.name.as_deref() {
			n.to_string()
		} else if let Some(d) = item.description.as_deref() {
			d.to_string()
		//otherwise, if this is empty too, we try to use the relations set to find the element labeling this one
		} else {
			relation_set.into_iter().filter_map(|this| this.text).collect()
		};
		utterance_buffer += &label;
	}
	let role = item.role;
	//there has to be a space between the accessible name of an object and its role, so insert it now
	write!(utterance_buffer, " {}", role.name()).expect("Able to write to string");
	Ok(vec![Focus(item.object).into(), Speak(utterance_buffer, Priority::Text).into()])
}

#[tracing::instrument(ret)]
pub async fn state_set(state_changed: CacheEvent<StateChangedEvent>) -> impl TryIntoCommands {
	SetState {
		item: state_changed.item.object.clone(),
		state: state_changed.state,
		enabled: state_changed.enabled,
	}
}

#[tracing::instrument(ret, err)]
pub async fn caret_moved(
	caret_moved: CacheEvent<TextCaretMovedEvent>,
	LastCaretPos(last_pos): LastCaretPos,
	LastFocused(last_focus): LastFocused,
) -> Result<Vec<OdiliaCommand>, OdiliaError> {
	let pos = caret_moved
		.position
		.try_into()
		.expect("Positive starting position for text insertion/deletion");
	let new_caret_pos = CaretPos(pos).into();
	let new_focus = Focus(caret_moved.inner.item.clone().into()).into();
	if let Some(ref text) = caret_moved.item.text {
		if last_focus == caret_moved.item.object {
			let min = min(pos, last_pos);
			let max = max(pos, last_pos);
			if min == 0 && max == 0 {
				return Ok(vec![new_focus, new_caret_pos]);
			}
			if let Some(text_slice) = text.get(min..max) {
				Ok(vec![
					new_focus,
					new_caret_pos,
					Speak(text_slice.to_string(), Priority::Text).into(),
				])
			} else {
				tracing::error!(
					"Can't read the text {text} at indecies {min}..{max}"
				);
				Ok(vec![new_focus, new_caret_pos])
			}
		} else {
			Ok(vec![
				new_focus,
				new_caret_pos,
				Speak(text.to_string(), Priority::Text).into(),
			])
		}
	} else {
		Ok(vec![new_focus, new_caret_pos])
	}
}
