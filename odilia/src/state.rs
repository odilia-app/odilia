use std::cell::Cell;

use circular_queue::CircularQueue;
use eyre::WrapErr;
use speech_dispatcher::{Connection as SPDConnection, Priority};
use tokio::sync::Mutex;
use zbus::{fdo::DBusProxy, names::UniqueName, zvariant::ObjectPath};

use crate::cache::Cache;
use atspi::{accessible::AccessibleProxy, cache::CacheProxy};
use odilia_common::{modes::ScreenReaderMode, settings::ApplicationConfig};

pub struct ScreenReaderState {
    pub atspi: atspi::Connection,
    pub dbus: DBusProxy<'static>,
    pub speaker: SPDConnection,
    pub config: ApplicationConfig,
    pub previous_caret_position: Cell<i32>,
    pub mode: Mutex<ScreenReaderMode>,
    pub accessible_history: Mutex<CircularQueue<(UniqueName<'static>, ObjectPath<'static>)>>,
    pub cache: Cache,
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

        let mode = Mutex::new(ScreenReaderMode {
            name: "CommandMode".to_string(),
        });

        let speaker = SPDConnection::open(
                env!("CARGO_PKG_NAME"),
                "main",
                "",
                speech_dispatcher::Mode::Threaded,
            )
            .wrap_err("Failed to connect to speech-dispatcher")?;
        tracing::debug!("speech dispatcher initialisation successful");

        let xdg_dirs = xdg::BaseDirectories::with_prefix("odilia").expect(
            "unable to find the odilia config directory according to the xdg dirs specification",
        );
        let config_path = xdg_dirs
            .place_config_file("config.toml")
            .expect("unable to place configuration file. Maybe your system is readonly?")
            .to_str()
            .unwrap()
            .to_owned();
        tracing::debug!(path=%config_path, "loading configuration file");
        let config =
            ApplicationConfig::new(&config_path).wrap_err("unable to load configuration file")?;
        tracing::debug!("configuration loaded successfully");

        let previous_caret_position = Cell::new(0);
        let accessible_history = Mutex::new(CircularQueue::with_capacity(16));
        let cache = Cache::new();
        Ok(Self {
            atspi,
            dbus,
            speaker,
            config,
            previous_caret_position,
            mode,
            accessible_history,
            cache,
        })
    }

    pub async fn register_event(&self, event: &str) -> zbus::Result<()> {
        let match_rule = event_to_match_rule(event);
        self.add_match_rule(&match_rule).await?;
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

    pub async fn add_match_rule(&self, match_rule: &str) -> zbus::fdo::Result<()> {
        self.dbus.add_match(match_rule).await
    }

    pub fn connection(&self) -> &zbus::Connection {
        self.atspi.connection()
    }

pub async fn say(&self, priority: Priority, text: String) -> bool {
    if text.is_empty() {
        tracing::warn!("blank string, aborting");
        return false;
    }
    self.speaker.say(priority, &text);
    tracing::trace!("Said: {}", text);
    true
}

pub async fn history_item(&self, index: usize) -> zbus::Result<Option<AccessibleProxy<'static>>> {
    let history = self.accessible_history.lock().await;
    if history.len() <= index {
      return Ok(None);
    }
    let (dest, path) = history
        .iter()
        .nth(index)
        .expect("Looking for invalid index in accessible history");
    Ok(Some(AccessibleProxy::builder(&self.connection())
        .destination(dest.to_owned())?
        .path(path)?
        .build()
        .await?))
}

/// Adds a new accessible to the history. We only store 16 previous accessibles, but theoretically, it should be lower.
pub async fn update_accessible(&self, sender: UniqueName<'_>, path: ObjectPath<'_>) {
    let mut history = self.accessible_history.lock().await;
    history.push((sender.to_owned(), path.to_owned()));
}

pub async fn build_cache<'a>(
    &self,
    dest: UniqueName<'a>,
    path: ObjectPath<'a>,
) -> zbus::Result<CacheProxy<'a>> {
    CacheProxy::builder(&self.connection())
        .destination(dest)?
        .path(path)?
        .build()
        .await
}
}

/// Converts an at-spi event string ("Object:StateChanged:Focused"), into a DBus match rule ("type='signal',interface='org.a11y.atspi.Event.Object',member='StateChanged'")
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
