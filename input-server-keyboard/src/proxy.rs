//! Proxy and interface for implementing the Odilia control DBus interface.

use odilia_common::command::{Speak, Focus, CaretPos};
use zbus::{proxy, interface, Result as zResult, object_server::SignalEmitter};

#[proxy(
	interface = "app.odilia.Control",
	default_service = "app.odilia.Controller",
	default_path = "/app/odilia/Controller",
)]
trait ControllerInterface {
	#[zbus(signal)]
  fn spoke(&self) -> zResult<Speak>;
	#[zbus(signal)]
  fn focused(&self) -> zResult<Focus>;
	#[zbus(signal)]
  fn caret_pos_moved(&self) -> zResult<CaretPos>;
}
