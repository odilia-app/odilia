use eyre::WrapErr;
use speech_dispatcher::Connection as SPDConnection;
use zbus::{fdo::DBusProxy, names::UniqueName, zvariant::ObjectPath};

use atspi::accessible::AccessibleProxy;

pub struct ScreenReaderState {
    pub atspi: atspi::Connection,
    pub dbus: DBusProxy<'static>,
    pub speaker: SPDConnection,
}

impl ScreenReaderState {
    #[tracing::instrument]
    pub async fn new() -> eyre::Result<Self> {
        let atspi = atspi::Connection::open()
            .await
            .wrap_err("Could not connect to at-spi bus")?;
        let dbus = DBusProxy::new(atspi.connection())
            .await
            .wrap_err("Failed to create org.freedesktop.DBus proxy")?;
        tracing::debug!("Connecting to speech-dispatcher");
        let speaker = SPDConnection::open(env!("CARGO_PKG_NAME"), "main", "", speech_dispatcher::Mode::Threaded).wrap_err("Failed to connect to speech-dispatcher")?;
        Ok(Self { atspi, dbus, speaker })
    }

pub async fn accessible<'a>(&'a self, destination: UniqueName<'a>, path: ObjectPath<'a>) -> zbus::Result<AccessibleProxy<'a>> {
    AccessibleProxy::builder(self.atspi.connection()).destination(destination)?.path(path)?.build().await
}

    #[allow(dead_code)]
    pub async fn register_event(&self, event: &str) -> zbus::Result<()> {
        let match_rule = event_to_match_rule(event);
        self.dbus.add_match(&match_rule).await?;
        self.atspi.register_event(event).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn deregister_event(&self, event: &str) -> zbus::Result<()> {
        let match_rule = event_to_match_rule(event);
        self.atspi.deregister_event(event).await?;
        self.dbus.remove_match(&match_rule).await?;
        Ok(())
    }
}

fn event_to_match_rule(event: &str) -> String {
    let mut components = event.split(':');
    let interface = components
        .next()
        .expect("Event should consist of 3 components separated by ':'");
    let member = components
        .next()
        .expect("Event should consist of 3 components separated by ':'");
    format!("type='signal',interface='org.a11y.atspi.Event.{interface}',member='{member}'")
}
