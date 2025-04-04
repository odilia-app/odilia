//! Proxy and interface for implementing the Odilia control DBus interface.

use odilia_common::events::{
    StopSpeech,
    Enable,
    Disable,
    ChangeMode,
    StructuralNavigation
};
use zbus::{proxy, interface, object_server::SignalEmitter, zvariant::Type};

#[proxy(
	interface = "app.odilia.Control",
	default_service = "app.odilia.Controller",
	default_path = "/app/odilia/Controller",
)]
trait Controller {
	#[zbus(signal)]
  fn speech_stopped(&self) -> zbus::Result<StopSpeech>;
	#[zbus(signal)]
  fn mode_changed(&self, new_mode: ChangeMode) -> zbus::Result<StopSpeech>;
}

pub struct ControllerInterface;

#[interface(
    name = "app.odilia.Control"
)]
impl ControllerInterface {
    #[zbus(signal)]
    async fn stop_speech(signal_emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
    #[zbus(signal)]
    async fn change_mode(signal_emitted: &SignalEmitter<'_>, new_mode: ChangeMode) -> zbus::Result<()>;
}

