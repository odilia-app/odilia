use atspi::error::AtspiError;
use serde_plain::Error as SerdePlainError;
use smartstring::alias::String as SmartString;
use std::{error::Error, fmt, str::FromStr};

#[derive(Debug)]
pub enum OdiliaError {
	AtspiError(AtspiError),
	PrimitiveConversionError(AccessiblePrimitiveConversionError),
	NoAttributeError(String),
	SerdeError(SerdePlainError),
	Zbus(zbus::Error),
	ZbusFdo(zbus::fdo::Error),
	Zvariant(zbus::zvariant::Error),
	InfallibleConversion(std::convert::Infallible),
}
impl Error for OdiliaError {}
impl From<zbus::fdo::Error> for OdiliaError {
	fn from(spe: zbus::fdo::Error) -> Self {
		Self::ZbusFdo(spe)
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
	InvalidKey(SmartString),
	#[error("Invalid repeat: {0:?}")]
	InvalidRepeat(SmartString),
	#[error("Invalid modifier: {0:?}")]
	InvalidModifier(SmartString),
	#[error("Invalid mode: {0:?}")]
	InvalidMode(SmartString),
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ModeFromStrError {
	#[error("Mode not found")]
	ModeNameNotFound,
}
