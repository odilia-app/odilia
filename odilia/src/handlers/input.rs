use odilia_common::{
	command::{Quit as QuitCommand, TryIntoCommands},
	events::{ChangeMode, Quit, StopSpeech, StructuralNavigation},
};
use ssip::Priority;

use crate::InputEvent;

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
