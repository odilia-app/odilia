use ssip::Priority;
use enum_dispatch::enum_dispatch;

use strum::{Display, EnumDiscriminants};

pub trait IntoCommands {
	fn into_commands(self) -> Vec<OdiliaCommand>;
}
impl From<Priority> for OdiliaCommand {
	fn from(p: Priority) -> OdiliaCommand {
		SpeechPriority(p).into()
	}
}
impl From<String> for OdiliaCommand {
	fn from(s: String) -> OdiliaCommand {
		Speak(s).into()
	}
}
impl<T1> IntoCommands for T1
where T1: Into<OdiliaCommand> {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![self.into()]
	}
}
impl<T1> IntoCommands for (T1,)
where T1: Into<OdiliaCommand> {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![self.0.into()]
	}
}
impl<T1,T2> IntoCommands for (T1,T2,)
where T1: Into<OdiliaCommand>,
			T2: Into<OdiliaCommand> {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![self.0.into(), self.1.into()]
	}
}
impl<T1,T2,T3> IntoCommands for (T1,T2,T3,)
where T1: Into<OdiliaCommand>,
			T2: Into<OdiliaCommand>,
			T3: Into<OdiliaCommand> {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![self.0.into(), self.1.into(), self.2.into()]
	}
}
impl<T1,T2,T3,T4> IntoCommands for (T1,T2,T3,T4,)
where T1: Into<OdiliaCommand>,
			T2: Into<OdiliaCommand>,
			T3: Into<OdiliaCommand>,
			T4: Into<OdiliaCommand> {
	fn into_commands(self) -> Vec<OdiliaCommand> {
		vec![self.0.into(), self.1.into(), self.2.into(), self.3.into()]
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
pub struct Speak(pub String);

#[derive(Debug, Clone)]
pub struct SpeechPriority(pub Priority);

impl CommandType for Speak {
	const CTYPE: OdiliaCommandDiscriminants = OdiliaCommandDiscriminants::Speak;
}
impl CommandType for SpeechPriority {
	const CTYPE: OdiliaCommandDiscriminants = OdiliaCommandDiscriminants::SpeechPriority;
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(Ord, PartialOrd, Display))]
#[enum_dispatch(CommandTypeDynamic)]
pub enum OdiliaCommand {
	Speak(Speak),
	SpeechPriority(SpeechPriority),
}
