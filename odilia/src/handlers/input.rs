use odilia_common::{command::TryIntoCommands, events::ChangeMode};
use ssip::Priority;

use crate::InputEvent;

#[tracing::instrument(ret)]
pub async fn change_mode(InputEvent(cm): InputEvent<ChangeMode>) -> impl TryIntoCommands {
	(Priority::Text, format!("{:?} mode", cm.0))
}
