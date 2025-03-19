//! Proxy and interface for implementing the Odilia control DBus interface.

use odilia_common::command::OdiliaCommand;
use zbus::{proxy, interface, Result as zResult, object_server::SignalEmitter};

#[proxy(
	interface = "app.odilia.Control",
	default_service = "app.odilia.Controller",
	default_path = "/app/odilia/Controller",
)]
trait ControllerInterface {
	#[zbus(signal)]
	fn control_event(&self) -> zResult<OdiliaCommand>;
}

struct ControllerInterface;

#[interface(name = "app.odilia.Control")]
impl ControllerInterface {
	#[zbus(signal)]
	async fn send_event(sig_em: &SignalEmitter<'_>, ev: OdiliaCommand) -> zResult<()> {
		todo!()
	}
}


