use std::{fmt::Debug, sync::atomic::AtomicUsize};

use crate::tower::from_state::TryFromState;
use circular_queue::CircularQueue;
use eyre::WrapErr;
use futures::future::err;
use futures::future::ok;
use futures::future::Ready;
use ssip_client_async::{MessageScope, Priority, PunctuationMode, Request as SSIPRequest};
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tracing::{debug, Instrument, Level};
use zbus::{
	fdo::DBusProxy, message::Type as MessageType, names::BusName, zvariant::ObjectPath,
	MatchRule,
};

use atspi_common::{
	events::{EventProperties, HasMatchRule, HasRegistryEventString},
	Event,
};
use atspi_connection::AccessibilityConnection;
use atspi_proxies::{accessible::AccessibleProxy, cache::CacheProxy};
use odilia_cache::Convertable;
use odilia_cache::{AccessibleExt, Cache, CacheItem};
use odilia_common::{
	cache::AccessiblePrimitive,
	command::CommandType,
	errors::{CacheError, OdiliaError},
	events::EventType,
	settings::{speech::PunctuationSpellingMode, ApplicationConfig},
	types::TextSelectionArea,
	Result as OdiliaResult,
};
use std::sync::Arc;

#[allow(clippy::module_name_repetitions)]
pub(crate) struct ScreenReaderState {
	pub atspi: AccessibilityConnection,
	pub dbus: DBusProxy<'static>,
	pub ssip: Sender<SSIPRequest>,
	pub previous_caret_position: Arc<AtomicUsize>,
	pub accessible_history: Arc<Mutex<CircularQueue<AccessiblePrimitive>>>,
	pub event_history: Mutex<CircularQueue<Event>>,
	pub cache: Arc<Cache>,
	pub config: Arc<ApplicationConfig>,
}
#[derive(Debug, Clone)]
pub struct AccessibleHistory(pub Arc<Mutex<CircularQueue<AccessiblePrimitive>>>);

impl<C> TryFromState<Arc<ScreenReaderState>, C> for AccessibleHistory {
	type Error = OdiliaError;
	type Future = Ready<Result<Self, Self::Error>>;
	fn try_from_state(state: Arc<ScreenReaderState>, _cmd: C) -> Self::Future {
		ok(AccessibleHistory(Arc::clone(&state.accessible_history)))
	}
}
impl<C> TryFromState<Arc<ScreenReaderState>, C> for CurrentCaretPos {
	type Error = OdiliaError;
	type Future = Ready<Result<Self, Self::Error>>;
	fn try_from_state(state: Arc<ScreenReaderState>, _cmd: C) -> Self::Future {
		ok(CurrentCaretPos(Arc::clone(&state.previous_caret_position)))
	}
}

#[derive(Debug, Clone)]
pub struct LastFocused(pub AccessiblePrimitive);
#[derive(Debug)]
pub struct CurrentCaretPos(pub Arc<AtomicUsize>);
#[derive(Debug, Clone)]
pub struct LastCaretPos(pub usize);
pub struct Speech(pub Sender<SSIPRequest>);
#[derive(Debug)]
pub struct Command<T>(pub T)
where
	T: CommandType;

#[derive(Debug)]
pub struct InputEvent<T>(pub T)
where
	T: EventType;

impl<E> TryFromState<Arc<ScreenReaderState>, E> for InputEvent<E>
where
	E: EventType + Clone + Debug,
{
	type Error = OdiliaError;
	type Future = Ready<Result<InputEvent<E>, Self::Error>>;
	fn try_from_state(_state: Arc<ScreenReaderState>, i_ev: E) -> Self::Future {
		ok(InputEvent(i_ev))
	}
}

impl<C> TryFromState<Arc<ScreenReaderState>, C> for Command<C>
where
	C: CommandType + Clone + Debug,
{
	type Error = OdiliaError;
	type Future = Ready<Result<Command<C>, Self::Error>>;
	fn try_from_state(_state: Arc<ScreenReaderState>, cmd: C) -> Self::Future {
		ok(Command(cmd))
	}
}

impl<C> TryFromState<Arc<ScreenReaderState>, C> for Speech
where
	C: CommandType + Debug,
{
	type Error = OdiliaError;
	type Future = Ready<Result<Speech, Self::Error>>;
	fn try_from_state(state: Arc<ScreenReaderState>, _cmd: C) -> Self::Future {
		ok(Speech(state.ssip.clone()))
	}
}

impl<E> TryFromState<Arc<ScreenReaderState>, E> for LastCaretPos
where
	E: Debug,
{
	type Error = OdiliaError;
	type Future = Ready<Result<Self, Self::Error>>;
	fn try_from_state(state: Arc<ScreenReaderState>, _event: E) -> Self::Future {
		ok(LastCaretPos(
			state.previous_caret_position
				.load(core::sync::atomic::Ordering::Relaxed),
		))
	}
}

impl<E> TryFromState<Arc<ScreenReaderState>, E> for LastFocused
where
	E: Debug,
{
	type Error = OdiliaError;
	type Future = Ready<Result<Self, Self::Error>>;
	fn try_from_state(state: Arc<ScreenReaderState>, _event: E) -> Self::Future {
		let span = tracing::span!(Level::INFO, "try_from_state");
		let _enter = span.enter();
		let Ok(ml) = state.accessible_history.lock() else {
			let e = OdiliaError::Generic("Could not get a lock on the history mutex. This is usually due to memory corruption or degradation and is a fatal error.".to_string());
			tracing::error!("{e:?}");
			return err(e);
		};
		let Some(last) = ml.iter().nth(0).cloned() else {
			let e = OdiliaError::Generic(
				"There are no previously focused items.".to_string(),
			);
			tracing::error!("{e:?}");
			return err(e);
		};
		ok(LastFocused(last))
	}
}

impl ScreenReaderState {
	#[tracing::instrument(skip_all)]
	pub async fn new(
		ssip: Sender<SSIPRequest>,
		config: ApplicationConfig,
	) -> eyre::Result<ScreenReaderState> {
		let atspi = AccessibilityConnection::new()
			.instrument(tracing::info_span!("connecting to at-spi bus"))
			.await
			.wrap_err("Could not connect to at-spi bus")?;
		let dbus = DBusProxy::new(atspi.connection())
			.instrument(tracing::debug_span!(
				"creating dbus proxy for accessibility connection"
			))
			.await
			.wrap_err("Failed to create org.freedesktop.DBus proxy")?;

		tracing::debug!("Reading configuration");

		let previous_caret_position = Arc::new(AtomicUsize::new(0));
		let accessible_history = Arc::new(Mutex::new(CircularQueue::with_capacity(16)));
		let event_history = Mutex::new(CircularQueue::with_capacity(16));
		let cache = Arc::new(Cache::new(atspi.connection().clone()));
		ssip.send(SSIPRequest::SetPitch(
			ssip_client_async::ClientScope::Current,
			config.speech.pitch,
		))
		.await?;
		ssip.send(SSIPRequest::SetVolume(
			ssip_client_async::ClientScope::Current,
			config.speech.volume,
		))
		.await?;
		ssip.send(SSIPRequest::SetOutputModule(
			ssip_client_async::ClientScope::Current,
			config.speech.module.clone(),
		))
		.await?;
		ssip.send(SSIPRequest::SetLanguage(
			ssip_client_async::ClientScope::Current,
			config.speech.language.clone(),
		))
		.await?;
		ssip.send(SSIPRequest::SetSynthesisVoice(
			ssip_client_async::ClientScope::Current,
			config.speech.person.clone(),
		))
		.await?;
		//doing it this way for now. It could have been done with a From impl, but I don't want to make ssip_client_async a dependency of odilia_common, so this conversion is done directly inside state, especially since this enum isn't supposed to grow any further, in complexity or variants
		let punctuation_mode = match config.speech.punctuation {
			PunctuationSpellingMode::Some => PunctuationMode::Some,
			PunctuationSpellingMode::Most => PunctuationMode::Most,
			PunctuationSpellingMode::None => PunctuationMode::None,
			PunctuationSpellingMode::All => PunctuationMode::All,
		};
		ssip.send(SSIPRequest::SetPunctuationMode(
			ssip_client_async::ClientScope::Current,
			punctuation_mode,
		))
		.await?;
		ssip.send(SSIPRequest::SetRate(
			ssip_client_async::ClientScope::Current,
			config.speech.rate,
		))
		.await?;
		Ok(Self {
			atspi,
			dbus,
			ssip,
			previous_caret_position,
			accessible_history,
			event_history,
			cache,
			config: Arc::new(config),
		})
	}
	#[tracing::instrument(level = "debug", skip(self), err)]
	pub async fn get_or_create_atspi_cache_item_to_cache(
		&self,
		atspi_cache_item: atspi_common::CacheItem,
	) -> OdiliaResult<CacheItem> {
		let prim = atspi_cache_item.object.clone().into();
		if self.cache.get(&prim).is_none() {
			self.cache.add(CacheItem::from_atspi_cache_item(
				atspi_cache_item,
				Arc::downgrade(&Arc::clone(&self.cache)),
				self.atspi.connection(),
			)
			.await?)?;
		}
		self.cache.get(&prim).ok_or(CacheError::NoItem.into())
	}
	#[tracing::instrument(level = "debug", skip(self), err)]
	pub async fn get_or_create_atspi_legacy_cache_item_to_cache(
		&self,
		atspi_cache_item: atspi_common::LegacyCacheItem,
	) -> OdiliaResult<CacheItem> {
		let prim = atspi_cache_item.object.clone().into();
		if self.cache.get(&prim).is_none() {
			self.cache.add(CacheItem::from_atspi_legacy_cache_item(
				atspi_cache_item,
				Arc::downgrade(&Arc::clone(&self.cache)),
				self.atspi.connection(),
			)
			.await?)?;
		}
		self.cache.get(&prim).ok_or(CacheError::NoItem.into())
	}
	#[tracing::instrument(skip_all, level = "debug", ret, err)]
	pub async fn get_or_create_event_object_to_cache<T: EventProperties>(
		&self,
		event: &T,
	) -> OdiliaResult<CacheItem> {
		let prim = AccessiblePrimitive::from_event(event);
		if self.cache.get(&prim).is_none() {
			self.cache.add(CacheItem::from_atspi_event(
				event,
				Arc::clone(&self.cache),
				self.atspi.connection(),
			)
			.await?)?;
		}
		self.cache.get(&prim).ok_or(CacheError::NoItem.into())
	}

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
	#[tracing::instrument(skip_all, err)]
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

	pub fn connection(&self) -> &zbus::Connection {
		self.atspi.connection()
	}
	#[tracing::instrument(skip(self))]
	pub async fn stop_speech(&self) -> bool {
		self.ssip.send(SSIPRequest::Cancel(MessageScope::All)).await.is_ok()
	}
	#[tracing::instrument(name = "closing speech dispatcher connection", skip(self))]
	pub async fn close_speech(&self) -> bool {
		self.ssip.send(SSIPRequest::Quit).await.is_ok()
	}
	#[tracing::instrument(skip(self))]
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
	pub fn event_history_item(&self, index: usize) -> Option<Event> {
		let history = self.event_history.lock().ok()?;
		history.iter().nth(index).cloned()
	}

	pub fn event_history_update(&self, event: Event) {
		if let Ok(mut history) = self.event_history.lock() {
			history.push(event);
		}
	}

	pub fn history_item(&self, index: usize) -> Option<AccessiblePrimitive> {
		let history = self.accessible_history.lock().ok()?;
		history.iter().nth(index).cloned()
	}

	/// Adds a new accessible to the history. We only store 16 previous accessibles, but theoretically, it should be lower.
	pub fn update_accessible(&self, new_a11y: AccessiblePrimitive) {
		if let Ok(mut history) = self.accessible_history.lock() {
			history.push(new_a11y);
		}
	}
	pub async fn build_cache<'a, T>(&self, dest: T) -> OdiliaResult<CacheProxy<'a>>
	where
		T: std::fmt::Display,
		T: TryInto<BusName<'a>>,
		<T as TryInto<BusName<'a>>>::Error: Into<zbus::Error>,
	{
		debug!("CACHE SENDER: {dest}");
		Ok(CacheProxy::builder(self.connection())
			.destination(dest)?
			.path(ObjectPath::from_static_str("/org/a11y/atspi/cache")?)?
			.build()
			.await?)
	}
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
			.get_or_create(&accessible_proxy, Arc::clone(&self.cache))
			.await
	}
	#[tracing::instrument(skip_all, err)]
	pub async fn add_cache_match_rule(&self) -> OdiliaResult<()> {
		let cache_rule = MatchRule::builder()
			.msg_type(MessageType::Signal)
			.interface("org.a11y.atspi.Cache")?
			.build();
		self.dbus.add_match_rule(cache_rule).await?;
		Ok(())
	}
}
