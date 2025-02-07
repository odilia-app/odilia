#![allow(clippy::module_name_repetitions)]

use crate::cache::AccessiblePrimitive;
use crate::errors::OdiliaError;
use enum_dispatch::enum_dispatch;
use ssip::Priority;
use std::array::IntoIter;
use std::convert::Infallible;
use std::iter::Chain;

use strum::{Display, EnumDiscriminants};

pub trait TryIntoCommands {
	type Error: Into<OdiliaError>;
	type Iter: Iterator<Item = OdiliaCommand> + Send;
	/// Fallibly returns a [`Vec`] of [`OdiliaCommand`]s to run.
	/// This collects the commands returned from [`TryIntoCommands::try_into_commands`] and returns
	/// them collected upon successful conversion.
	///
	/// # Errors
	///
	/// When implemented, the function is allowed to fail with any type that can be converted into
	/// [`OdiliaError`], but conversion should between these types should be done from the
	/// implementers' side, liekly using `?`.
	fn try_into_commands_collected(self) -> Result<Vec<OdiliaCommand>, OdiliaError>
	where
		Self: Sized,
	{
		match self.try_into_commands() {
			Ok(vec) => Ok(vec.collect()),
			Err(e) => Err(e),
		}
	}
	/// Fallibly returns an iterator of [`OdiliaCommand`]s to run.
	///
	/// # Errors
	///
	/// When implemented, the function is allowed to fail with any type that can be converted into
	/// [`OdiliaError`], but conversion should between these types should be done from the
	/// implementers' side, liekly using `?`.
	fn try_into_commands(self) -> Result<Self::Iter, OdiliaError>;
}
impl TryIntoCommands for Result<Vec<OdiliaCommand>, OdiliaError> {
	type Error = OdiliaError;
	type Iter = std::vec::IntoIter<OdiliaCommand>;
	fn try_into_commands(self) -> Result<Self::Iter, OdiliaError> {
		self.map(|vec| vec.into_iter())
	}
}
impl<T: IntoCommands> TryIntoCommands for T {
	type Error = Infallible;
	type Iter = T::Iter;
	fn try_into_commands(self) -> Result<Self::Iter, OdiliaError> {
		Ok(self.into_commands())
	}
}
impl<T: IntoCommands, E: Into<OdiliaError>> TryIntoCommands for Result<T, E> {
	type Error = E;
	type Iter = T::Iter;
	fn try_into_commands(self) -> Result<Self::Iter, OdiliaError> {
		match self {
			Ok(ok) => Ok(ok.into_commands()),
			Err(err) => Err(err.into()),
		}
	}
}

pub trait IntoCommands {
	type Iter: Iterator<Item = OdiliaCommand> + Send;
	fn into_commands(self) -> Self::Iter;
}

impl IntoCommands for CaretPos {
	type Iter = IntoIter<OdiliaCommand, 1>;
	fn into_commands(self) -> Self::Iter {
		[self.into()].into_iter()
	}
}
impl IntoCommands for Focus {
	type Iter = IntoIter<OdiliaCommand, 1>;
	fn into_commands(self) -> Self::Iter {
		[self.into()].into_iter()
	}
}
impl IntoCommands for (Priority, &str) {
	type Iter = IntoIter<OdiliaCommand, 1>;
	fn into_commands(self) -> Self::Iter {
		[Speak(self.1.to_string(), self.0).into()].into_iter()
	}
}
impl IntoCommands for (Priority, String) {
	type Iter = IntoIter<OdiliaCommand, 1>;
	fn into_commands(self) -> Self::Iter {
		[Speak(self.1, self.0).into()].into_iter()
	}
}
impl IntoCommands for () {
	type Iter = IntoIter<OdiliaCommand, 0>;
	fn into_commands(self) -> Self::Iter {
		[].into_iter()
	}
}

impl<const N: usize> IntoCommands for [OdiliaCommand; N] {
	type Iter = IntoIter<OdiliaCommand, N>;
	fn into_commands(self) -> Self::Iter {
		self.into_iter()
	}
}

impl<T1> IntoCommands for (T1,)
where
	T1: IntoCommands,
{
	type Iter = T1::Iter;
	fn into_commands(self) -> Self::Iter {
		self.0.into_commands()
	}
}
impl<T1, T2> IntoCommands for (T1, T2)
where
	T1: IntoCommands,
	T2: IntoCommands,
{
	type Iter = Chain<T1::Iter, T2::Iter>;
	fn into_commands(self) -> Self::Iter {
		self.0.into_commands().chain(self.1.into_commands())
	}
}
impl<T1, T2, T3> IntoCommands for (T1, T2, T3)
where
	T1: IntoCommands,
	T2: IntoCommands,
	T3: IntoCommands,
{
	type Iter = Chain<Chain<T1::Iter, T2::Iter>, T3::Iter>;
	fn into_commands(self) -> Self::Iter {
		self.0.into_commands()
			.chain(self.1.into_commands())
			.chain(self.2.into_commands())
	}
}
impl<T1, T2, T3, T4> IntoCommands for (T1, T2, T3, T4)
where
	T1: IntoCommands,
	T2: IntoCommands,
	T3: IntoCommands,
	T4: IntoCommands,
{
	type Iter = Chain<Chain<Chain<T1::Iter, T2::Iter>, T3::Iter>, T4::Iter>;
	fn into_commands(self) -> Self::Iter {
		self.0.into_commands()
			.chain(self.1.into_commands())
			.chain(self.2.into_commands())
			.chain(self.3.into_commands())
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
