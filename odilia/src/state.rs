use std::{
	fmt,
	fmt::Debug,
	process::Child,
	sync::{atomic::AtomicUsize, Arc, Mutex},
};

use async_channel::Sender;
use atspi::{
	connection::AccessibilityConnection,
	events::{DBusMatchRule, RegistryEventString},
	Event,
};
use circular_queue::CircularQueue;
use futures_util::future::{err, ok, Ready};
use odilia_cache::{CacheActor, CacheItem, CacheRequest, CacheResponse, Item};
use odilia_common::{
	cache::AccessiblePrimitive,
	command::CommandType,
	errors::OdiliaError,
	events::EventType,
	settings::{speech::PunctuationSpellingMode, ApplicationConfig},
	Result as OdiliaResult,
};
use ssip_client_async::{Priority, PunctuationMode, Request as SSIPRequest};
use tracing::{Instrument, Level};

use crate::tower::from_state::TryFromState;

impl Debug for ScreenReaderState {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("State").finish_non_exhaustive()
	}
}

#[allow(clippy::module_name_repetitions)]
pub struct ScreenReaderState {
	pub atspi: AccessibilityConnection,
	pub ssip: Sender<SSIPRequest>,
	pub previous_caret_position: Arc<AtomicUsize>,
	pub accessible_history: Arc<Mutex<CircularQueue<AccessiblePrimitive>>>,
	pub cache_actor: CacheActor,
	pub config: Arc<ApplicationConfig>,
	pub children_pids: Arc<Mutex<Vec<Child>>>,
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
		cache_actor: CacheActor,
	) -> Result<ScreenReaderState, OdiliaError> {
		let atspi = AccessibilityConnection::new()
			.instrument(tracing::info_span!("connecting to at-spi bus"))
			.await?;

		tracing::debug!("Reading configuration");

		let previous_caret_position = Arc::new(AtomicUsize::new(0));
		let accessible_history = Arc::new(Mutex::new(CircularQueue::with_capacity(16)));
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
			ssip,
			previous_caret_position,
			accessible_history,
			cache_actor,
			config: Arc::new(config),
			children_pids: Arc::new(Mutex::new(Vec::new())),
		})
	}

	#[tracing::instrument(skip_all, level = "debug", ret, err)]
	pub async fn cache_from_event(&self, event: Event) -> OdiliaResult<CacheItem> {
		self.cache_actor
			.request(CacheRequest::EventHandler(Box::new(event)))
			.await
			.map(|cr| match cr {
				CacheResponse::Item(Item(ci)) => ci,
				e => panic!("Inappropriate response: {e:?}"),
			})
	}

	#[tracing::instrument(skip_all, err)]
	pub async fn register_event<E: RegistryEventString + DBusMatchRule>(
		&self,
	) -> OdiliaResult<()> {
		Ok(self.atspi.register_event::<E>().await?)
	}

	#[allow(dead_code)]
	pub async fn deregister_event<E: RegistryEventString + DBusMatchRule>(
		&self,
	) -> OdiliaResult<()> {
		Ok(self.atspi.deregister_event::<E>().await?)
	}

	pub fn connection(&self) -> &zbus::Connection {
		self.atspi.connection()
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

	pub fn history_item(&self, index: usize) -> Option<AccessiblePrimitive> {
		let history = self.accessible_history.lock().ok()?;
		history.iter().nth(index).cloned()
	}

	#[tracing::instrument(skip_all, err)]
	pub fn add_child_proc(&self, child: Child) -> OdiliaResult<()> {
		let mut children = self.children_pids.lock()?;
		children.push(child);
		Ok(())
	}
}

//impl<Cr> TryFromState<Arc<ScreenReaderState>, Cr> for Cr
//where
//	Cr: RequestExt
//{
//	type Error = OdiliaError;
//	type Future = Pin<Box<(dyn Future<Output = Result<Self, Self::Error>> + Send + 'static)>>;
//	#[tracing::instrument(skip(state), level = "trace", ret)]
//	fn try_from_state(state: Arc<ScreenReaderState>, req: Cr) -> Self::Future {
//		Box::pin(async move {
//			let cache_item = state.get_or_create(&event).await?;
//			Ok(InnerEvent::new(event, cache_item))
//		})
//	}
//}
