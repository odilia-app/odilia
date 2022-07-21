use std::path::Path;
use std::sync::{
    atomic::AtomicI32,
    Arc,
};

use eyre::WrapErr;
use circular_queue::CircularQueue;
use speech_dispatcher::Connection as SPDConnection;
use zbus::{fdo::DBusProxy, names::UniqueName, zvariant::ObjectPath};
use tokio::sync::Mutex;

use atspi::{
  accessible::AccessibleProxy,
  text::TextProxy,
  collection::CollectionProxy,
};

use odilia_common::settings::ApplicationConfig;

const ODILIA_CONFIG_FILE_PATH: &str = "./target/debug/config.toml";

pub struct ScreenReaderState {
    pub atspi: atspi::Connection,
    pub dbus: DBusProxy<'static>,
    pub speaker: SPDConnection,
    pub config: ApplicationConfig,
    pub previous_caret_position: AtomicI32,
    pub accessible_history: Arc<Mutex<CircularQueue<(UniqueName<'static>, ObjectPath<'static>)>>>
}

impl ScreenReaderState {
    #[tracing::instrument]
    pub async fn new() -> eyre::Result<ScreenReaderState> {
        let atspi = atspi::Connection::open()
            .await
            .wrap_err("Could not connect to at-spi bus")?;
        let dbus = DBusProxy::new(atspi.connection())
            .await
            .wrap_err("Failed to create org.freedesktop.DBus proxy")?;
        tracing::debug!("Connecting to speech-dispatcher");
        let speaker = SPDConnection::open(
            env!("CARGO_PKG_NAME"),
            "main",
            "",
            speech_dispatcher::Mode::Threaded,
        )
        .wrap_err("Failed to connect to speech-dispatcher")?;
        tracing::debug!("speech dispatcher initialisation successful");
        tracing::debug!(path=%ODILIA_CONFIG_FILE_PATH, "loading configuration file");
        let config_full_path = Path::new(ODILIA_CONFIG_FILE_PATH);
        let config = ApplicationConfig::new(config_full_path.canonicalize()?.to_str().unwrap())
            .wrap_err("unable to load configuration file")?;
        tracing::debug!("configuration loaded successfully");
        let previous_caret_position = AtomicI32::new(0);
        let accessible_history = Arc::new(Mutex::new(CircularQueue::with_capacity(16)));
        Ok(Self {
            atspi,
            dbus,
            speaker,
            config,
            previous_caret_position,
            accessible_history,
        })
    }

    pub async fn text<'a>(
      &'a self,
      destination: UniqueName<'a>,
      path: ObjectPath<'a>,
    ) -> zbus::Result<TextProxy<'a>> {
      TextProxy::builder(self.atspi.connection())
          .destination(destination)?
          .path(path)?
          .build()
          .await
    }
    pub async fn collection<'a>(
        &'a self,
        destination: UniqueName<'a>,
        path: ObjectPath<'a>,
    ) -> zbus::Result<CollectionProxy<'a>> {
        CollectionProxy::builder(self.atspi.connection())
            .destination(destination)?
            .path(path)?
            .build()
            .await
    }

    pub async fn accessible<'a>(
        &'a self,
        destination: UniqueName<'a>,
        path: ObjectPath<'a>,
    ) -> zbus::Result<AccessibleProxy<'a>> {
        AccessibleProxy::builder(self.atspi.connection())
            .destination(destination)?
            .path(path)?
            .build()
            .await
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
