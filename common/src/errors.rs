use std::{
	error::Error,
	fmt,
	fmt::{Debug, Display},
	str::FromStr,
};

use atspi::AtspiError;
use atspi_common::AtspiError as AtspiTypesError;
use serde_plain::Error as SerdePlainError;

use crate::{cache::AccessiblePrimitive, command::OdiliaCommand};

#[derive(Debug)]
pub enum OdiliaError {
	AtspiError(AtspiError),
	AtspiTypesError(AtspiTypesError),
	PrimitiveConversionError(AccessiblePrimitiveConversionError),
	NoAttributeError(String),
	SerdeError(SerdePlainError),
	Zbus(zbus::Error),
	ZbusFdo(zbus::fdo::Error),
	Zvariant(zbus::zvariant::Error),
	SendError(SendError),
	Cache(CacheError),
	InfallibleConversion(std::convert::Infallible),
	ConversionError(std::num::TryFromIntError),
	Config(ConfigError),
	PoisoningError,
	Generic(String),
	Static(&'static str),
	ServiceNotFound(String),
	PredicateFailure(String),
}

impl From<&'static str> for OdiliaError {
	fn from(s: &'static str) -> OdiliaError {
		Self::Static(s)
	}
}

#[derive(Debug)]
pub enum SendError {
	Atspi(Box<atspi::Event>),
	Command(OdiliaCommand),
	Ssip(ssip::Request),
}

macro_rules! send_err_impl {
	($tokio_err:ty, $variant:path) => {
		#[cfg(feature = "tokio")]
		impl From<$tokio_err> for OdiliaError {
			fn from(t_err: $tokio_err) -> OdiliaError {
				OdiliaError::SendError($variant(t_err.0))
			}
		}
	};
	($tokio_err:ty, $variant:path, Box) => {
		#[cfg(feature = "tokio")]
		impl From<$tokio_err> for OdiliaError {
			fn from(t_err: $tokio_err) -> OdiliaError {
				OdiliaError::SendError($variant(Box::new(t_err.0)))
			}
		}
	};
}

send_err_impl!(tokio::sync::broadcast::error::SendError<atspi::Event>, SendError::Atspi, Box);
send_err_impl!(tokio::sync::mpsc::error::SendError<atspi::Event>, SendError::Atspi, Box);
send_err_impl!(tokio::sync::broadcast::error::SendError<OdiliaCommand>, SendError::Command);
send_err_impl!(tokio::sync::mpsc::error::SendError<OdiliaCommand>, SendError::Command);
send_err_impl!(tokio::sync::broadcast::error::SendError<ssip::Request>, SendError::Ssip);
send_err_impl!(tokio::sync::mpsc::error::SendError<ssip::Request>, SendError::Ssip);

#[derive(Debug)]
pub enum ConfigError {
	Figment(Box<figment::Error>),
	ValueNotFound,
	PathNotFound,
}
impl From<figment::Error> for ConfigError {
	fn from(t_err: figment::Error) -> Self {
		Self::Figment(Box::new(t_err))
	}
}
impl std::fmt::Display for ConfigError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Figment(t) => Display::fmt(t, f),
			Self::ValueNotFound => f.write_str("Vlaue not found in config file."),
			Self::PathNotFound => {
				f.write_str("The path for the config file was not found.")
			}
		}
	}
}
impl std::error::Error for ConfigError {}
#[derive(Debug)]
pub enum CacheError {
	NotAvailable,
	NoItem,
	NoLock,
	TextBoundsError,
	/// This item is already in the cache.
	DuplicateItem(indextree::NodeId),
	/// The cache operation succeeded, but the cache is in an inconsistent state now.
	/// This usually means that a node has been added to the cache, but its parent was not found; in
	/// this case, it is left as a disconnected part of the graph.
	///
	/// The data is the set of keys that need to be cached to keep it in a consistent state.
	MoreData(Vec<AccessiblePrimitive>),
	IndexTree(indextree::NodeError),
}

impl From<indextree::NodeError> for CacheError {
	fn from(ixne: indextree::NodeError) -> Self {
		CacheError::IndexTree(ixne)
	}
}
impl From<indextree::NodeError> for OdiliaError {
	fn from(ixne: indextree::NodeError) -> Self {
		OdiliaError::Cache(ixne.into())
	}
}

impl std::fmt::Display for CacheError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotAvailable => f.write_str("The cache has been dropped from memory. This never happens under normal circumstances, and should never happen. Please send a detailed bug report if this ever happens."),
			Self::NoItem => f.write_str("No item in cache found."),
      Self::NoLock => f.write_str("It was not possible to get a lock on this item from the cache."),
      Self::TextBoundsError => f.write_str("The range asked for in a call to a get_string_*_offset function has invalid bounds."),
      Self::MoreData(items) => {
          f.write_str("The cache requires more data to be in a consistent state: {items:?}")?;
          Debug::fmt(items, f)
      },
      Self::DuplicateItem(nid) => {
          f.write_str("This item is already in the cache: ")?;
          Display::fmt(nid, f)
      },
      Self::IndexTree(err) => Display::fmt(err, f),
		}
	}
}
impl std::error::Error for CacheError {}
impl Error for OdiliaError {}
impl<T> From<std::sync::PoisonError<T>> for OdiliaError {
	fn from(_: std::sync::PoisonError<T>) -> Self {
		Self::PoisoningError
	}
}
impl From<std::num::TryFromIntError> for OdiliaError {
	fn from(fie: std::num::TryFromIntError) -> Self {
		Self::ConversionError(fie)
	}
}
impl From<zbus::fdo::Error> for OdiliaError {
	fn from(spe: zbus::fdo::Error) -> Self {
		Self::ZbusFdo(spe)
	}
}
impl From<std::convert::Infallible> for OdiliaError {
	fn from(infallible: std::convert::Infallible) -> Self {
		Self::InfallibleConversion(infallible)
	}
}
impl From<CacheError> for OdiliaError {
	fn from(cache_error: CacheError) -> Self {
		Self::Cache(cache_error)
	}
}
impl From<zbus::Error> for OdiliaError {
	fn from(spe: zbus::Error) -> Self {
		Self::Zbus(spe)
	}
}
impl From<zbus::zvariant::Error> for OdiliaError {
	fn from(spe: zbus::zvariant::Error) -> Self {
		Self::Zvariant(spe)
	}
}
impl From<SerdePlainError> for OdiliaError {
	fn from(spe: SerdePlainError) -> Self {
		Self::SerdeError(spe)
	}
}
impl From<AtspiError> for OdiliaError {
	fn from(err: AtspiError) -> OdiliaError {
		Self::AtspiError(err)
	}
}
impl fmt::Display for OdiliaError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self:?}")
	}
}

#[derive(Clone, Debug)]
pub enum AccessiblePrimitiveConversionError {
	ParseError(<i32 as FromStr>::Err),
	ObjectConversionError(atspi::error::ObjectPathConversionError),
	NoPathId,
	InvalidPath,
	NoFirstSectionOfSender,
	NoSecondSectionOfSender,
	NoSender,
	ErrSender,
}
impl From<AccessiblePrimitiveConversionError> for OdiliaError {
	fn from(apc_error: AccessiblePrimitiveConversionError) -> Self {
		Self::PrimitiveConversionError(apc_error)
	}
}
impl fmt::Display for AccessiblePrimitiveConversionError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self:?}")
	}
}
impl std::error::Error for AccessiblePrimitiveConversionError {}
impl From<atspi::error::ObjectPathConversionError> for AccessiblePrimitiveConversionError {
	fn from(object_conversion_error: atspi::error::ObjectPathConversionError) -> Self {
		Self::ObjectConversionError(object_conversion_error)
	}
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum KeyFromStrError {
	#[error("Empty key binding")]
	EmptyString,
	#[error("No key was provided")]
	NoKey,
	#[error("Empty key")]
	EmptyKey,
	#[error("Invalid key: {0:?}")]
	InvalidKey(String),
	#[error("Invalid repeat: {0:?}")]
	InvalidRepeat(String),
	#[error("Invalid modifier: {0:?}")]
	InvalidModifier(String),
	#[error("Invalid mode: {0:?}")]
	InvalidMode(String),
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ModeFromStrError {
	#[error("Mode not found")]
	ModeNameNotFound,
}
