use std::{fs, sync::atomic::AtomicI32};

use circular_queue::CircularQueue;
use eyre::WrapErr;
use ssip_client_async::{MessageScope, Priority, Request as SSIPRequest};
use parking_lot::{RwLock};
use tokio::sync::{mpsc::Sender};

use zbus::{fdo::DBusProxy, MatchRule, MessageType};

use atspi_client::{accessible_ext::AccessibleExt, convertable::Convertable};
use atspi_common::{
	events::{HasMatchRule, HasRegistryEventString},
	Event,
};
use atspi_connection::AccessibilityConnection;
use atspi_proxies::{accessible::AccessibleProxy};
use odilia_cache::Cache as InnerCache;
use odilia_common::{
	errors::{ConfigError},
	modes::ScreenReaderMode,
	settings::ApplicationConfig,
	types::TextSelectionArea,
  events::ScreenReaderEvent,
	cache::{AccessiblePrimitive, CacheItem},
	Result as OdiliaResult,
};
use std::sync::Arc;

/// All the types used by the state.
/// These are all guarenteed to be able to be accessed from multiple threads safely.
/// They are all wrapped in an [`std::sync::Arc`] to allow efficient access across threads.
mod types {
	use std::sync::Arc;
	use std::sync::atomic::AtomicI32;
	use static_assertions::assert_impl_all;
	use odilia_common::{modes::ScreenReaderMode, settings::ApplicationConfig, cache::AccessiblePrimitive};
	use odilia_cache::Cache as OdiliaCache;
	use atspi_connection::AccessibilityConnection;
	use atspi_common::events::Event;
	use ssip_client_async::types::Request as SSIPRequest;
	use tokio::sync::mpsc::Sender;
	use parking_lot::RwLock;
	use circular_queue::CircularQueue;

	/// The type only requires read access.
	pub type AtspiConnection = Arc<AccessibilityConnection>;
	assert_impl_all!(AtspiConnection: Send, Sync);

	/// The channel only needs read access to perform sends, and awaiting for closure.
	pub type SsipSendChannel = Arc<Sender<SSIPRequest>>;
	assert_impl_all!(SsipSendChannel: Send, Sync);

	/// The DBus proxy structure that handles match rules.
	pub type Proxy<'a> = Arc<zbus::fdo::DBusProxy<'a>>;
	assert_impl_all!(Proxy: Send, Sync);

	/// The current mode of the screen reader.
	pub type Mode = Arc<RwLock<ScreenReaderMode>>;
	assert_impl_all!(Mode: Send, Sync);

	/// Odilia's configuration
	pub type Config = Arc<RwLock<ApplicationConfig>>;
	assert_impl_all!(Config: Send, Sync);

	/// Last caret position.
	pub type CaretPosition = Arc<AtomicI32>;
	assert_impl_all!(CaretPosition: Send, Sync);

	/// The history of previously focused items. This may contain duplicates.
	pub type FocusHistory = Arc<RwLock<CircularQueue<AccessiblePrimitive>>>;
	assert_impl_all!(FocusHistory: Send, Sync);

	/// The history of atspi events.
	pub type EventHistory = Arc<RwLock<CircularQueue<Event>>>;
	assert_impl_all!(EventHistory: Send, Sync);

	/// The cache.
	pub type Cache = Arc<OdiliaCache>;
	assert_impl_all!(Cache: Send, Sync);
}
use types::*;

/// The global state of the screen reader.
/// This is never interaced with directly, instead, you can use the associated types and [`crate::traits::IntoStatePieces`] to *references* of *pieces* of state, then open up the smallest possible window for modification within the [`crate::traits::IntoStatePieces::execute
/// When modifying the types here, please change the type aliases in the code directly above the structure, as these types are used throughout the codebase.
#[allow(clippy::module_name_repetitions)]
pub struct ScreenReaderState {
	pub atspi: AtspiConnection,
	pub dbus: Proxy<'static>,
	pub ssip: SsipSendChannel,
	pub config: Config,
	pub caret_position: CaretPosition,
	pub mode: Mode,
	pub focus_history: FocusHistory,
	pub event_history: EventHistory,
	pub cache: Cache,
}

impl ScreenReaderState {
	#[tracing::instrument]
	pub async fn new(ssip: Arc<Sender<SSIPRequest>>) -> eyre::Result<ScreenReaderState> {
		let atspi: AtspiConnection = AccessibilityConnection::open()
			.await
			.wrap_err("Could not connect to at-spi bus")?
			.into();
		let dbus: Proxy = DBusProxy::new(atspi.connection())
			.await
			.wrap_err("Failed to create org.freedesktop.DBus proxy")?
			.into();

		let mode: Mode = RwLock::new(ScreenReaderMode { name: "CommandMode".to_string() }).into();

		tracing::debug!("Reading configuration");
		let xdg_dirs = xdg::BaseDirectories::with_prefix("odilia").expect(
            "unable to find the odilia config directory according to the xdg dirs specification",
        );
		let config_path = xdg_dirs.place_config_file("config.toml").expect(
			"unable to place configuration file. Maybe your system is readonly?",
		);
		if !config_path.exists() {
			fs::write(&config_path, include_str!("../config.toml"))
				.expect("Unable to copy default config file.");
		}
		let config_path = config_path.to_str().ok_or(ConfigError::PathNotFound)?.to_owned();
		tracing::debug!(path=%config_path, "loading configuration file");
		let config: Config = RwLock::new(ApplicationConfig::new(&config_path)
			.wrap_err("unable to load configuration file")?)
			.into();
		tracing::debug!("configuration loaded successfully");

		let caret_position: CaretPosition = AtomicI32::new(0).into();
		let focus_history: FocusHistory = RwLock::new(CircularQueue::with_capacity(16)).into();
		let event_history: EventHistory = RwLock::new(CircularQueue::with_capacity(16)).into();
		let cache: Cache = InnerCache::new(atspi.connection().clone()).into();

		Ok(Self {
			atspi,
			dbus,
			ssip,
			config,
			caret_position,
			mode,
			focus_history,
			event_history,
			cache,
		})
	}
  
  pub async fn apply_all(&self, _events: Vec<ScreenReaderEvent>) -> OdiliaResult<bool> {
    Ok(true)
  }

	//pub async fn get_or_create_atspi_cache_item_to_cache(
	//	&self,
	//	atspi_cache_item: atspi_common::CacheItem,
	//) -> OdiliaResult<CacheItem> {
	//	let prim = atspi_cache_item.object.clone().try_into()?;
	//	if self.cache.get(&prim).is_none() {
	//		self.cache.add(CacheItem::from_atspi_cache_item(
	//			atspi_cache_item,
	//			//Arc::downgrade(&Arc::clone(&self.cache)),
	//			self.atspi.connection(),
	//		)
	//		.await?)?;
	//	}
	//	self.cache.get(&prim).ok_or(CacheError::NoItem.into())
	//}
	//pub async fn get_or_create_event_object_to_cache<'a, T: GenericEvent<'a> + Sync>(
	//	&self,
	//	event: &T,
	//) -> OdiliaResult<CacheItem> {
	//	let prim = AccessiblePrimitive::from_event(event)?;
	//	if self.cache.get(&prim).is_none() {
	//		self.cache.add(CacheItem::from_atspi_event(
	//			event,
	//			//Arc::downgrade(&Arc::clone(&self.cache)),
	//			self.atspi.connection(),
	//		)
	//		.await?)?;
	//	}
	//	self.cache.get(&prim).ok_or(CacheError::NoItem.into())
	//}

	// TODO: use cache; this will uplift performance MASSIVELY, also TODO: use this function instad of manually generating speech every time.
	#[allow(dead_code)]
	pub async fn generate_speech_string(
		&self,
		acc: AccessibleProxy<'_>,
		select: TextSelectionArea,
	) -> OdiliaResult<String> {
		let acc_text = acc.to_text().await?;
		let _acc_hyper = acc.to_hyperlink().await?;
		//let _full_text = acc_text.get_text_ext().await?;
		let (mut text_selection, start, end) = match select {
			TextSelectionArea::Granular(granular) => {
				acc_text.get_string_at_offset(granular.index, granular.granularity)
					.await?
			}
			TextSelectionArea::Index(indexed) => (
				acc_text.get_text(indexed.start, indexed.end).await?,
				indexed.start,
				indexed.end,
			),
		};
		// TODO: Use streaming filters, or create custom function
		let children = acc.get_children_ext().await?;
		let mut children_in_range = Vec::new();
		for child in children {
			let child_hyper = child.to_hyperlink().await?;
			let index = child_hyper.start_index().await?;
			if index >= start && index <= end {
				children_in_range.push(child);
			}
		}
		for child in children_in_range {
			let child_hyper = child.to_hyperlink().await?;
			let child_start = usize::try_from(child_hyper.start_index().await?)?;
			let child_end = usize::try_from(child_hyper.end_index().await?)?;
			let child_text = format!(
				"{}, {}",
				child.name().await?,
				child.get_role_name().await?
			);
			text_selection.replace_range(
				child_start + (usize::try_from(start)?)
					..child_end + (usize::try_from(start)?),
				&child_text,
			);
		}
		// TODO: add logic for punctuation
		Ok(text_selection)
	}

	pub async fn register_event<E: HasRegistryEventString + HasMatchRule>(
		&self,
	) -> OdiliaResult<()> {
		Ok(self.atspi.register_event::<E>().await?)
	}

	#[allow(dead_code)]
	pub async fn deregister_event<E: HasRegistryEventString + HasMatchRule>(
		&self,
	) -> OdiliaResult<()> {
		Ok(self.atspi.deregister_event::<E>().await?)
	}

#[allow(dead_code)]
	pub fn connection(&self) -> &zbus::Connection {
		self.atspi.connection()
	}

	pub async fn stop_speech(&self) -> bool {
		self.ssip.send(SSIPRequest::Cancel(MessageScope::All)).await.is_ok()
	}

	pub async fn close_speech(&self) -> bool {
		self.ssip.send(SSIPRequest::Quit).await.is_ok()
	}

	pub async fn say(&self, priority: Priority, text: String) -> bool {
		if self.ssip.send(SSIPRequest::SetPriority(priority)).await.is_err() {
			return false;
		}
		if self.ssip.send(SSIPRequest::Speak).await.is_err() {
			return false;
		}
		// this crashed ssip-client because the connection is automatically stopped when invalid text is sent; since the period character on a line by itself is the stop character, there's not much we can do except filter it out explicitly.
		if text == *"." {
			return false;
		}
		if self.ssip
			.send(SSIPRequest::SendLines(Vec::from([text])))
			.await
			.is_err()
		{
			return false;
		}
		true
	}

	#[allow(dead_code)]
	pub async fn event_history_item(&self, index: usize) -> Option<Event> {
		let history = self.event_history.read();
		history.iter().nth(index).cloned()
	}

	//pub async fn event_history_update(&self, event: Event) {
	//	let mut history = self.event_history.lock();
	//	history.push(event);
	//}

	//pub async fn history_item<'a>(&self, index: usize) -> Option<AccessiblePrimitive> {
	//	let history = self.accessible_history.lock();
	//	history.iter().nth(index).cloned()
	//}

	/// Adds a new accessible to the history. We only store 16 previous accessibles, but theoretically, it should be lower.
	//pub async fn update_accessible(&self, new_a11y: AccessiblePrimitive) {
	//	let mut history = self.accessible_history.lock();
	//	history.push(new_a11y);
	//}
	//pub async fn build_cache<'a>(&self, dest: UniqueName<'a>) -> OdiliaResult<CacheProxy<'a>> {
	//	debug!("CACHE SENDER: {dest}");
	//	Ok(CacheProxy::builder(self.connection())
	//		.destination(dest)?
	//		.path(ObjectPath::from_static_str("/org/a11y/atspi/cache")?)?
	//		.build()
	//		.await?)
	//}
	#[allow(dead_code)]
	pub async fn get_or_create_cache_item(
		&self,
		accessible: AccessiblePrimitive,
	) -> OdiliaResult<CacheItem> {
		let accessible_proxy = AccessibleProxy::builder(self.atspi.connection())
			.destination(accessible.sender.as_str())?
			.path(accessible.id.to_string())?
			.build()
			.await?;
		self.cache
			.get_or_create(&accessible_proxy, Arc::downgrade(&self.cache))
			.await
	}
	//pub async fn new_accessible<'a, T: GenericEvent<'a>>(
	//	&self,
	//	event: &T,
	//) -> OdiliaResult<AccessibleProxy<'_>> {
	//	let sender = event.sender().to_owned();
	//	let path = event.path().to_owned();
	//	Ok(AccessibleProxy::builder(self.connection())
	//		.cache_properties(zbus::CacheProperties::No)
	//		.destination(sender)?
	//		.path(path)?
	//		.build()
	//		.await?)
	//}
	pub async fn add_cache_match_rule(&self) -> OdiliaResult<()> {
		let cache_rule = MatchRule::builder()
			.msg_type(MessageType::Signal)
			.interface("org.a11y.atspi.Cache")?
			.build();
		self.dbus.add_match_rule(cache_rule).await?;
		Ok(())
	}
}
