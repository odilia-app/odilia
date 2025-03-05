use odilia_common::{
	events::{ChangeMode, ScreenReaderEvent, StopSpeech},
	modes::ScreenReaderMode as Mode,
};

fn main() {
	// stop all current speech
	let stop = ScreenReaderEvent::StopSpeech(StopSpeech);

	// change to an arbitrary mode
	let mode_change = ScreenReaderEvent::ChangeMode(ChangeMode(Mode::Browse));

	println!("{}", serde_json::to_string(&stop).unwrap());
	println!("{}", serde_json::to_string(&mode_change).unwrap());
}
