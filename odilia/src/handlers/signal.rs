use odilia_common::command::{Quit as QuitCommand, TryIntoCommands};
use ssip::Priority;

use crate::{signal, state::Signal};

pub async fn sigint_quit(_: Signal<signal::Int>) -> impl TryIntoCommands {
	QuitCommand
}

pub async fn sigusr1_reload_config(_: Signal<signal::Usr1>) -> impl TryIntoCommands {
	(Priority::Message, "Reload config")
}
