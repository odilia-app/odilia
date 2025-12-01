use odilia_common::{
	command::{Focus, Move, Quit as QuitCommand, TryIntoCommands},
	events::{ChangeMode, Navigate, Quit, StopSpeech, StructuralNavigation},
};
use ssip::Priority;

use crate::{state::NavigateTo, InputEvent};

#[tracing::instrument(ret)]
pub async fn change_mode(InputEvent(cm): InputEvent<ChangeMode>) -> impl TryIntoCommands {
	(Priority::Text, format!("{:?} mode", cm.0))
}

#[tracing::instrument(ret)]
pub async fn quit_input(InputEvent(cm): InputEvent<Quit>) -> impl TryIntoCommands {
	QuitCommand
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

#[tracing::instrument(ret)]
pub async fn navigate(
	InputEvent(nav): InputEvent<Navigate>,
	NavigateTo(maybe_item): NavigateTo,
) -> impl TryIntoCommands {
	if let Some(item) = maybe_item {
		Ok(vec![Move(item.object).into()])
	} else {
		Ok(vec![(Priority::Text, "No item found!").into()])
	}
}
