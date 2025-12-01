use std::{
	cmp::{max, min},
	fmt::Write,
};

use atspi::{
	events::{
		document::LoadCompleteEvent,
		object::{StateChangedEvent, TextCaretMovedEvent},
	},
	Role, State,
};
use odilia_cache::LabelledBy;
use odilia_common::command::{CaretPos, Focus, OdiliaCommand, SetState, Speak, TryIntoCommands};
use ssip::Priority;

use crate::{
	state::{LastCaretPos, LastFocused},
	tower::{
		state_changed::Focused, ActiveAppEvent, CacheEvent, EventProp, NonContainerEvent,
		PropertyType, RelationSet, Subtree,
	},
};

const OBJECT_REPLACEMENT_CHARACTER: char = '\u{FFFC}';

#[tracing::instrument(ret)]
pub async fn doc_loaded(loaded: ActiveAppEvent<LoadCompleteEvent>) -> impl TryIntoCommands {
	(Priority::Text, "Doc loaded")
}

#[tracing::instrument(ret)]
pub async fn focused(
	state_changed: NonContainerEvent<Focused>,
	EventProp(relation_set): EventProp<RelationSet<LabelledBy>>,
	EventProp(subtree): EventProp<Subtree>,
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
	// This lets us read Fractal messages.
	// But we don't know what the general method should be.
	if role == Role::ListItem {
		// skip root element (`item`)
		for child in subtree.values().skip(1) {
			if let Some(txt) = &child.text {
				let _ = write!(utterance_buffer, "{txt}");
			}
		}
	}
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

#[tracing::instrument(ret)]
pub async fn caret_moved_update_state(
	caret_moved: CacheEvent<TextCaretMovedEvent>,
) -> impl TryIntoCommands {
	[
		CaretPos(
			caret_moved
				.position
				.try_into()
				.expect("Positive starting position for text insertion/deletion"),
		)
		.into(),
		Focus(caret_moved.inner.item.clone().into()).into(),
	]
}

/// Resolve the hypertext of the current item with its children (and recursively until finished)
/// The `subtree` should contain all items in the sub tree, starting with the lement that contains
/// hypertext with object replacement characters, followed by all other elements
///
/// TODO: for now, in order for this function to work correctly, some additional, spec-aditional contracts
/// must be upheld:
///
/// - The number of object replacement characters in the text (U+FFFC) must be equal to the number of children
///   of the element. This function will return [`TextError::InvalidHyperlinkText`] with the string
///   and the number of children if this occurs.
/// - The children must all implement the `org.a11y.atspi.Text` interface. If this fails to uphold,
///   then the [`TextError::NonTextChildren`] with the parent and children respectively will be
///   reported.
fn resolve_hypertext(
	root: odilia_cache::CacheItem,
	subtree: &<Subtree as PropertyType>::Type,
	range: Option<(usize, usize)>,
) -> String {
	// if the object doesn't have any text, then return a blank string
	let Some(text) = root.text else {
		return String::new();
	};
	let mut children: Vec<_> =
		subtree.iter().filter(|(_, ci)| ci.parent == root.object).collect();
	children.sort_by_key(|(_, ci)| ci.index);
	children.reverse();
	let new_str: String = text
		.char_indices()
		.filter_map(|(i, c)| {
			if let Some((start, end)) = range {
				if i >= start && i < end {
					Some(c)
				} else {
					None
				}
			} else {
				Some(c)
			}
		})
		// NOTE: the unicode "replacement character" U+FFFD _IS NOT_
		// the "object replacement character" U+FFFC
		.map(|c| {
			if c == OBJECT_REPLACEMENT_CHARACTER {
				let (_, ch) = children.pop().expect("A child!");
				let text = ch.text.clone().expect("Text child!");
				let visited_text = if ch.states.contains(State::Visited) {
					", visited "
				} else if ch.role == Role::Link {
					", unvisited "
				} else {
					""
				}
				.to_string();
				text + " " + &visited_text + &ch.role.to_string()
			} else {
				c.to_string()
			}
		})
		// why can't I collect multiple strings?
		.reduce(|start, add| start + &add)
		.into_iter()
		.collect();
	new_str
}

#[tracing::instrument(ret)]
pub async fn caret_moved(
	caret_moved: CacheEvent<TextCaretMovedEvent>,
	LastCaretPos(last_pos): LastCaretPos,
	LastFocused(last_focus): LastFocused,
	EventProp(subtree): EventProp<Subtree>,
) -> Option<OdiliaCommand> {
	let pos = caret_moved
		.position
		.try_into()
		.expect("Positive starting position for text insertion/deletion");
	if let Some(ref _text) = caret_moved.item.text {
		if last_focus == caret_moved.item.object {
			let min = min(pos, last_pos);
			let max = max(pos, last_pos);
			if min == 0 && max == 0 {
				return None;
			}
			let text_slice =
				resolve_hypertext(caret_moved.item, &subtree, Some((min, max)));
			if !text_slice.is_empty() {
				return Some(Speak(text_slice, Priority::Text).into());
			}
		} else {
			let text2 = resolve_hypertext(caret_moved.item, &subtree, None);
			return Some(Speak(text2, Priority::Text).into());
		}
	}
	None
}
