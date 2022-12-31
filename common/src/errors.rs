use std::{error::Error, fmt};
use atspi::error::AtspiError;
use smartstring::alias::String as SmartString;

#[derive(Debug)]
pub enum OdiliaError {
	AtspiError(AtspiError),
}
impl Error for OdiliaError {}
impl<T: Into<AtspiError>> From<T> for OdiliaError {
	fn from(err: T) -> OdiliaError {
		Self::AtspiError(err.into())
	}
}
impl fmt::Display for OdiliaError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
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
