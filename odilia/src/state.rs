use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use circular_queue::CircularQueue;
use eyre::WrapErr;
use futures::stream::Stream;
use lazy_static::lazy_static;
use tokio::sync::{Mutex, OnceCell};
use speech_dispatcher::{Connection as SPDConnection, Priority};
use zbus::{fdo::DBusProxy, names::UniqueName, zvariant::ObjectPath, Connection};

use atspi::{
    accessible::AccessibleProxy,
    events::Event,
    cache::CacheProxy,
};
use odilia_common::{modes::ScreenReaderMode, settings::ApplicationConfig};

lazy_static! {
    static ref STATE: OnceCell<ScreenReaderState> = OnceCell::new();
}

pub struct OdiliaCache {
    pub by_id_read: evmap::ReadHandleFactory<u32, (String, String)>,
    pub by_id_write: Arc<Mutex<evmap::WriteHandle<u32, (String, String)>>>,
}

pub struct ScreenReaderState {
    pub atspi: atspi::Connection,
    pub dbus: DBusProxy<'static>,
    pub speaker: Arc<Mutex<SPDConnection>>,
    pub config: ApplicationConfig,
    pub previous_caret_position: AtomicI32,
    pub mode: Arc<Mutex<ScreenReaderMode>>,
    pub accessible_history: Arc<Mutex<CircularQueue<(UniqueName<'static>, ObjectPath<'static>)>>>,
    pub cache: OdiliaCache,
}

pub async fn register_event(event: &str) -> zbus::Result<()> {
    let state = STATE.get().unwrap();
    state.register_event(event).await?;
    Ok(())
}

/// Returns the AT-SPI event stream
pub async fn get_event_stream() -> impl Stream<Item = zbus::Result<Event>> {
    let conn = &STATE.get().unwrap().atspi;
    conn.event_stream()
}

/// Adds a new accessible to the history. We only sotre 16 previous accessibles, but theoretically, it should be lower.
pub async fn update_accessible(sender: UniqueName<'_>, path: ObjectPath<'_>) -> bool {
    let accessible_history_arc = Arc::clone(&STATE.get().unwrap().accessible_history);
    let mut accessible_history = accessible_history_arc.lock().await;
    accessible_history.push((sender.to_owned(), path.to_owned()));
    true
}

/// Initializes state for the screen reader.
/// There are some things, which unfortunately must be held in global state.
/// For example, the mode, currently focused item, a speaker instance (for TTS), etc.
/// This initializes all of it so that the global state may be queried from the rest of the program.
/// @returns bool: true if state has been initialized successfully, false otherwise.
pub async fn init_state() -> eyre::Result<()> {
    let sr_state = ScreenReaderState::new().await.unwrap();
    STATE.set(sr_state).map_err(|_| eyre::eyre!("Could not initialize state"))
}

pub async fn get_connection() -> Connection {
    let c_conn = STATE.get().unwrap().atspi.connection().clone();
    return c_conn;
}

pub async fn say(priority: Priority, text: String) -> bool {
    let state = STATE.get().unwrap();
    let spd = state.speaker.lock().await;
    if text == "" {
        tracing::warn!("blank string, aborting");
        return false;
    }
    spd.say(priority, &text);
    tracing::debug!("Said: {}", text);
    true
}

pub async fn by_id_write() -> Arc<Mutex<evmap::WriteHandle<u32, (String, String)>>> { 
    Arc::clone(&STATE.get().unwrap().cache.by_id_write)
}

pub async fn build_cache<'a>(dest: UniqueName<'a>, path: ObjectPath<'a>) -> zbus::Result<CacheProxy<'a>> {
    CacheProxy::builder(&get_connection().await)
        .destination(dest.to_owned())?
        .path(path.to_owned())?
        .build()
        .await
}

pub async fn get_accessible_history<'a>(index: i32) -> zbus::Result<AccessibleProxy<'a>> {
    let history_arc = Arc::clone(&STATE.get().unwrap().accessible_history);
    let history = history_arc.lock().await;
    let mut history_iter = history.iter();
    for _ in 0..index {
        history_iter.next();
    }
    let history_item = history_iter
        .next()
        .expect("Looking for invalid index in accessible history");
    AccessibleProxy::builder(&get_connection().await)
        .destination(history_item.0.to_owned())?
        .path(history_item.1.to_owned())?
        .build()
        .await
}

pub async fn update_caret_position(new_pos: i32) -> bool {
    let current_position = &STATE.get().unwrap().previous_caret_position;
    current_position.store(new_pos, Ordering::Relaxed);
    true
}
pub async fn get_previous_caret_position() -> i32 {
    STATE
        .get()
        .unwrap()
        .previous_caret_position
        .load(Ordering::Relaxed)
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
        let mode = Arc::new(Mutex::new(ScreenReaderMode {
            name: "CommandMode".to_string(),
        }));
        let speaker = Arc::new(Mutex::new(
            SPDConnection::open(
                env!("CARGO_PKG_NAME"),
                "main",
                "",
                speech_dispatcher::Mode::Threaded,
            )
            .wrap_err("Failed to connect to speech-dispatcher")?,
        ));
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
            ApplicationConfig::new(&config_path)
            .wrap_err("unable to load configuration file")?;
        tracing::debug!("configuration loaded successfully");
        let previous_caret_position = AtomicI32::new(0);
        let accessible_history = Arc::new(Mutex::new(CircularQueue::with_capacity(16)));
        let (rh, wh) = evmap::new();
        let write_handle = Arc::new(Mutex::new(wh));
        let cache = OdiliaCache { by_id_read: rh.factory(), by_id_write: write_handle };
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

    pub async fn add_match_rule(&self, match_rule: &str) -> zbus::Result<()> {
        self.dbus.add_match(match_rule).await?;
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
