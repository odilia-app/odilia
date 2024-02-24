use odilia_common::{events::ScreenReaderEvent, modes::ScreenReaderMode};

fn main() {
	// a blank event that does nothing
	let noop = ScreenReaderEvent::Noop;

	// stop all current speech
	let stop = ScreenReaderEvent::StopSpeech;

	// change to an arbitrary mode
	let mode_change = ScreenReaderEvent::ChangeMode(ScreenReaderMode::new("browse mode"));

	println!("{}", serde_json::to_string(&noop).unwrap());
	println!("{}", serde_json::to_string(&stop).unwrap());
	println!("{}", serde_json::to_string(&mode_change).unwrap());
}
