use std::{fmt, fmt::Debug, str::FromStr};

use atspi::AtspiError;
use serde_plain::Error as SerdePlainError;
use thiserror::Error;

use crate::{cache::AccessiblePrimitive, command::OdiliaCommand};

#[derive(Error, Debug)]
pub enum OdiliaError {
	#[error("atspi: {0}")]
	AtspiError(AtspiError),
	#[error("conversion: {0}")]
	PrimitiveConversionError(AccessiblePrimitiveConversionError),
	#[error("No attributes: {0}")]
	NoAttributeError(String),
	#[error("Serde: {0}")]
	SerdeError(SerdePlainError),
	#[error("zbus: {0}")]
	Zbus(#[from] zbus::Error),
	#[error("zbus::fdo: {0}")]
	ZbusFdo(#[from] zbus::fdo::Error),
	#[error("zbus::zvariant: {0}")]
	Zvariant(#[from] zbus::zvariant::Error),
	#[error("Send: {0}")]
	SendError(SendError),
	#[error("Cache: {0}")]
	Cache(#[from] CacheError),
	#[error("N/A: {0}")]
	InfallibleConversion(#[from] std::convert::Infallible),
	#[error("From int: {0}")]
	ConversionError(#[from] std::num::TryFromIntError),
	#[error("Config: {0}")]
	Config(#[from] config::ConfigError),
	#[error("Poisoned")]
	PoisoningError,
	#[error("Generic: {0}")]
	Generic(String),
	#[error("{0}")]
	Static(&'static str),
	#[error("Service not found: {0}")]
	ServiceNotFound(String),
	#[error("Predicate failure: {0}")]
	PredicateFailure(String),
	#[error("I/O:: {0}")]
	Io(#[from] std::io::Error),
	#[error("Notify: {0}")]
	Notify(#[from] NotifyError),
	#[error("Opts: {0}")]
	CommandLine(#[from] lexopt::Error),
	#[error("SSIP: {0}")]
	Ssip(#[from] ssip_client_async::ClientError),
}

impl From<&'static str> for OdiliaError {
	fn from(s: &'static str) -> OdiliaError {
		Self::Static(s)
	}
}
impl From<String> for OdiliaError {
	fn from(s: String) -> OdiliaError {
		Self::Generic(s)
	}
}

#[derive(Error, Debug)]
pub enum SendError {
	#[error("atspi: {0:?}")]
	Atspi(Box<atspi::Event>),
	#[error("command: {0:?}")]
	Command(OdiliaCommand),
	#[error("SSIP: {0:?}")]
	Ssip(ssip::Request),
}

macro_rules! send_err_impl {
	($tokio_err:ty, $variant:path, $dep:literal) => {
		#[cfg(feature = $dep)]
		impl From<$tokio_err> for OdiliaError {
			fn from(t_err: $tokio_err) -> OdiliaError {
				OdiliaError::SendError($variant(t_err.0))
			}
		}
	};
	($tokio_err:ty, $variant:path, Box, $dep:literal) => {
		#[cfg(feature = $dep)]
		impl From<$tokio_err> for OdiliaError {
			fn from(t_err: $tokio_err) -> OdiliaError {
				OdiliaError::SendError($variant(Box::new(t_err.0)))
			}
		}
	};
}

send_err_impl!(
	tokio::sync::broadcast::error::SendError<atspi::Event>,
	SendError::Atspi,
	Box,
	"tokio"
);
send_err_impl!(tokio::sync::mpsc::error::SendError<atspi::Event>, SendError::Atspi, Box, "tokio");
send_err_impl!(
	tokio::sync::broadcast::error::SendError<OdiliaCommand>,
	SendError::Command,
	"tokio"
);
send_err_impl!(tokio::sync::mpsc::error::SendError<OdiliaCommand>, SendError::Command, "tokio");
send_err_impl!(tokio::sync::broadcast::error::SendError<ssip::Request>, SendError::Ssip, "tokio");
send_err_impl!(tokio::sync::mpsc::error::SendError<ssip::Request>, SendError::Ssip, "tokio");

send_err_impl!(async_channel::SendError<atspi::Event>, SendError::Atspi, Box, "async-io");
send_err_impl!(async_channel::SendError<OdiliaCommand>, SendError::Command, "async-io");
send_err_impl!(async_channel::SendError<ssip::Request>, SendError::Ssip, "async-io");

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
	#[error("The cache has been dropped from memory. This never happens under normal circumstances, and should never happen. Please send a detailed bug report if this ever happens.")]
	NotAvailable,
	#[error("Item not found in cache.")]
	NoItem,
	#[error("It was not possible to get a lock on this item from the cache.")]
	NoLock,
	#[error("The range asked for in a call to a get_string_*_offset function has invalid bounds.")]
	TextBoundsError,
	/// This item is already in the cache.
	#[error("Duplicate: {0}")]
	DuplicateItem(indextree::NodeId),
	/// The cache operation succeeded, but the cache is in an inconsistent state now.
	/// This usually means that a node has been added to the cache, but its parent was not found; in
	/// this case, it is left as a disconnected part of the graph.
	///
	/// The data is the set of keys that need to be cached to keep it in a consistent state.
	#[error("Require {} more items", .0.len())]
	MoreData(Vec<AccessiblePrimitive>),
	#[error("Indextree: ")]
	IndexTree(indextree::NodeError),
}

impl From<indextree::NodeError> for OdiliaError {
	fn from(itne: indextree::NodeError) -> Self {
		OdiliaError::Cache(itne.into())
	}
}
impl From<indextree::NodeError> for CacheError {
	fn from(itne: indextree::NodeError) -> Self {
		CacheError::IndexTree(itne)
	}
}

impl<T> From<std::sync::PoisonError<T>> for OdiliaError {
	fn from(_: std::sync::PoisonError<T>) -> Self {
		Self::PoisoningError
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

#[derive(thiserror::Error, Debug)]
pub enum NotifyError {
	#[error("connection or monitor related error")]
	Dbus(#[from] zbus::Error),
	#[error("zbus specification defined error")]
	DbusSpec(#[from] zbus::fdo::Error),
}
