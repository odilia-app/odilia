use zbus::zvariant::Type;
use zbus_lockstep_macros::validate;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants};

use crate::modes::ScreenReaderMode;
use atspi_common::Role;

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Debug, Type)]
/// A list of features supported natively by Odilia.
pub enum Feature {
	/// Unimplemented, but will eventually stop all speech until re-activated.
	Speech,
	/// Unimplemented.
	Braille, // TODO
}

#[derive(Eq, PartialEq, Clone, Hash, Serialize, Deserialize, Debug, Type)]
#[serde(tag = "direction")]
pub enum Direction {
	Forward,
	Backward,
}

pub trait EventType {
	const ETYPE: ScreenReaderEventDiscriminants;
}
#[enum_dispatch]
pub trait EventTypeDynamic {
	fn etype(&self) -> ScreenReaderEventDiscriminants;
}
impl<T: EventType> EventTypeDynamic for T {
	fn etype(&self) -> ScreenReaderEventDiscriminants {
		T::ETYPE
	}
}
macro_rules! impl_event_type {
	($type:ty, $disc:ident) => {
		impl EventType for $type {
			const ETYPE: ScreenReaderEventDiscriminants =
				ScreenReaderEventDiscriminants::$disc;
		}
	};
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct StopSpeech;
impl_event_type!(StopSpeech, StopSpeech);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct Enable(pub Feature);
impl_event_type!(Enable, Enable);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct Disable(pub Feature);
impl_event_type!(Disable, Disable);

#[validate(signal: "ModeChanged")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[repr(transparent)]
pub struct ChangeMode(pub ScreenReaderMode);
impl_event_type!(ChangeMode, ChangeMode);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
pub struct StructuralNavigation(pub Direction, pub Role);
impl_event_type!(StructuralNavigation, StructuralNavigation);

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug, EnumDiscriminants)]
/// Events which can be trigged through Odilia's external API.
/// Subject to change without notice until v1.0, but we're [open to suggestions on our Github](https://github.com/odilia-app/odilia/); please reach out with features you'd like to see.
#[strum_discriminants(derive(Ord, PartialOrd, Display))]
#[enum_dispatch(EventTypeDynamic)]
pub enum ScreenReaderEvent {
	/// Stop all current speech.
	StopSpeech(StopSpeech),
	/// Enable a feature.
	Enable(Enable),
	/// Disable a feature.
	Disable(Disable),
	/// Change mode of the screen reader. This is currently global, but it should be per application, and an update should only affect the current application.
	ChangeMode(ChangeMode),
	/// Navigate to the next [`Role`] in [`Direction`] by depth-first search.
	StructuralNavigation(StructuralNavigation),
}
