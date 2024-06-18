#![allow(clippy::module_name_repetitions)]

use crate::cache::AccessiblePrimitive;
use crate::errors::OdiliaError;
use enum_dispatch::enum_dispatch;
use ssip::Priority;
use std::convert::Infallible;

use strum::{Display, EnumDiscriminants};

pub trait TryIntoCommands {
	type Error: Into<OdiliaError>;
	/// Fallibly returns a [`Vec`] of [`Command`]s to run.
	///
	/// # Errors
	///
	/// When implemented, the function is allowed to fail with any type that can be converted into
	/// [`OdiliaError`], but conversion should between these types should be done from the
	/// implementers' side, liekly using `?`.
	fn try_into_commands(self) -> Result<Vec<OdiliaCommand>, OdiliaError>;
}
impl TryIntoCommands for Result<Vec<OdiliaCommand>, OdiliaError> {
	type Error = OdiliaError;
	fn try_into_commands(self) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		self
	}
}
impl<T: IntoCommands> TryIntoCommands for T {
	type Error = Infallible;
	fn try_into_commands(self) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		Ok(self.into_commands())
	}
}
impl<T: IntoCommands, E: Into<OdiliaError>> TryIntoCommands for Result<T, E> {
	type Error = E;
	fn try_into_commands(self) -> Result<Vec<OdiliaCommand>, OdiliaError> {
		match self {
			Ok(ok) => Ok(ok.into_commands()),
			Err(err) => Err(err.into()),
		}
	}
}

pub trait IntoCommands {
	fn into_commands(self) -> Vec<OdiliaCommand>;
}

impl IntoCommands for CaretPos {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![self.into()]
	}
}
impl IntoCommands for Focus {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![self.into()]
	}
}
impl IntoCommands for (Priority, &str) {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![Speak(self.1.to_string(), self.0).into()]
	}
}
impl IntoCommands for (Priority, String) {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![Speak(self.1, self.0).into()]
	}
}
impl IntoCommands for () {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![]
	}
}
impl<T1> IntoCommands for (T1,)
where
	T1: IntoCommands,
{
	fn into_commands(self) -> Vec<OdiliaCommand> {
		self.0.into_commands()
	}
}
impl<T1, T2> IntoCommands for (T1, T2)
where
	T1: IntoCommands,
	T2: IntoCommands,
{
	fn into_commands(self) -> Vec<OdiliaCommand> {
		let mut ret = self.0.into_commands();
		ret.extend(self.1.into_commands());
		ret
	}
}
impl<T1, T2, T3> IntoCommands for (T1, T2, T3)
where
	T1: IntoCommands,
	T2: IntoCommands,
	T3: IntoCommands,
{
	fn into_commands(self) -> Vec<OdiliaCommand> {
		let mut ret = self.0.into_commands();
		ret.extend(self.1.into_commands());
		ret.extend(self.2.into_commands());
		ret
	}
}
impl<T1, T2, T3, T4> IntoCommands for (T1, T2, T3, T4)
where
	T1: IntoCommands,
	T2: IntoCommands,
	T3: IntoCommands,
	T4: IntoCommands,
{
	fn into_commands(self) -> Vec<OdiliaCommand> {
		let mut ret = self.0.into_commands();
		ret.extend(self.1.into_commands());
		ret.extend(self.2.into_commands());
		ret.extend(self.3.into_commands());
		ret
	}
}

pub trait CommandType {
	const CTYPE: OdiliaCommandDiscriminants;
}
#[enum_dispatch]
pub trait CommandTypeDynamic {
	fn ctype(&self) -> OdiliaCommandDiscriminants;
}
impl<T: CommandType> CommandTypeDynamic for T {
	fn ctype(&self) -> OdiliaCommandDiscriminants {
		T::CTYPE
	}
}

#[derive(Debug, Clone)]
pub struct CaretPos(pub usize);

#[derive(Debug, Clone)]
pub struct Speak(pub String, pub Priority);

#[derive(Debug, Clone)]
pub struct Focus(pub AccessiblePrimitive);

impl CommandType for Speak {
	const CTYPE: OdiliaCommandDiscriminants = OdiliaCommandDiscriminants::Speak;
}
impl CommandType for Focus {
	const CTYPE: OdiliaCommandDiscriminants = OdiliaCommandDiscriminants::Focus;
}
impl CommandType for CaretPos {
	const CTYPE: OdiliaCommandDiscriminants = OdiliaCommandDiscriminants::CaretPos;
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Ord, PartialOrd, Display))]
#[enum_dispatch(CommandTypeDynamic)]
pub enum OdiliaCommand {
	Speak(Speak),
	Focus(Focus),
	CaretPos(CaretPos),
}
