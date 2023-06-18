use atspi_common::{AtspiError};
use serde_plain::Error as SerdePlainError;
use std::{error::Error, fmt};

#[non_exhaustive]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum OdiliaError {
	Atspi(AtspiError),
	PrimitiveConversionError(AccessiblePrimitiveConversionError),
	NoAttributeError(String),
	Cache(CacheError),
	InfallibleConversion,
	/// A parsing error converting a string into a type (usually an integer).
	/// The error message is preserved through the `String` variant data.
	ParseError(String),
	Config(ConfigError),
	PoisoningError,
	/// A generic error type where the error message is preserved, but it is not enumerable.
	/// These are the kind of errors that generally should have a [bug filed](https://github.com/odilia-app/odilia/issues) for them.
	Generic(String),
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ConfigError {
	Tini(String),
	ValueNotFound,
	PathNotFound,
}
#[cfg(feature = "zbus")]
impl From<zbus::Error> for OdiliaError {
	fn from(z_err: zbus::Error) -> Self {
		OdiliaError::Generic(z_err.to_string())
	}
}
#[cfg(feature = "zbus")]
impl From<zbus::fdo::Error> for OdiliaError {
	fn from(z_err: zbus::fdo::Error) -> Self {
		OdiliaError::Generic(z_err.to_string())
	}
}
impl From<tini::Error> for ConfigError {
	fn from(t_err: tini::Error) -> Self {
		Self::Tini(t_err.to_string())
	}
}
impl std::fmt::Display for ConfigError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Tini(t) => t.fmt(f),
			Self::ValueNotFound => f.write_str("Vlaue not found in config file."),
			Self::PathNotFound => {
				f.write_str("The path for the config file was not found.")
			}
		}
	}
}
impl std::error::Error for ConfigError {}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum CacheError {
	NotAvailable,
	NoItem,
	NoLock,
	TextBoundsError,
}
impl std::fmt::Display for CacheError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotAvailable => f.write_str("The cache has been dropped from memory. This never happens under normal circumstances, and should never happen. Please send a detailed bug report if this ever happens."),
			Self::NoItem => f.write_str("No item in cache found."),
      Self::NoLock => f.write_str("It was not possible to get a lock on this item from the cache."),
      Self::TextBoundsError => f.write_str("The range asked for in a call to a get_string_*_offset function has invalid bounds."),
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
		Self::ParseError(fie.to_string())
	}
}
impl From<std::convert::Infallible> for OdiliaError {
	fn from(_infallible: std::convert::Infallible) -> Self {
		Self::InfallibleConversion
	}
}
impl From<CacheError> for OdiliaError {
	fn from(cache_error: CacheError) -> Self {
		Self::Cache(cache_error)
	}
}
impl From<zvariant::Error> for OdiliaError {
	fn from(spe: zvariant::Error) -> Self {
		Self::Atspi(spe.into())
	}
}
impl From<SerdePlainError> for OdiliaError {
	fn from(spe: SerdePlainError) -> Self {
		Self::Generic(spe.to_string())
	}
}
impl From<AtspiError> for OdiliaError {
	fn from(err: AtspiError) -> OdiliaError {
		Self::Atspi(err)
	}
}
impl fmt::Display for OdiliaError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self:?}")
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum AccessiblePrimitiveConversionError {
	ParseError,
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

#[derive(Debug, thiserror::Error)]
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

#[derive(Debug, thiserror::Error)]
pub enum ModeFromStrError {
	#[error("Mode not found")]
	ModeNameNotFound,
}
