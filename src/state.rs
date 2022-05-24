use eyre::WrapErr;
use zbus::fdo::DBusProxy;

pub struct ScreenReaderState {
    pub atspi: atspi::Connection,
    pub dbus: DBusProxy<'static>,
}

impl ScreenReaderState {
    pub async fn new() -> eyre::Result<Self> {
        let atspi = atspi::Connection::open()
            .await
            .wrap_err("Could not connect to at-spi bus")?;
        let dbus = DBusProxy::new(atspi.connection()).await.wrap_err("Failed to create org.freedesktop.DBus proxy")?;
        Ok(Self { atspi, dbus })
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
let interface = components.next().expect("Event should consist of 3 components separated by ':'");
let member = components.next().expect("Event should consist of 3 components separated by ':'");
format!("type='signal',interface='org.a11y.atspi.Event.{interface}',member='{member}'")
}
