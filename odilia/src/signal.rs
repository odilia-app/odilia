use async_signal::Signal;

use crate::tower::choice::ChooserStatic;

pub trait SignalType {}

macro_rules! impl_sig {
	($type:ident, $orig:path) => {
		#[allow(dead_code)]
		#[derive(Clone, Copy, Debug)]
		pub struct $type;
		impl SignalType for $type {}
		impl ChooserStatic<Signal> for $type {
			fn identifier() -> Signal {
				$orig
			}
		}
		impl TryFrom<Signal> for $type {
			type Error = String;
			fn try_from(sig: Signal) -> Result<$type, Self::Error> {
				if $orig == sig {
					Ok($type)
				} else {
					Err(format!(
						"Invalid signal type for {:?}: {:?}",
						$type, sig
					))
				}
			}
		}
	};
}

impl_sig!(Hup, Signal::Hup);
impl_sig!(Int, Signal::Int);
impl_sig!(Quit, Signal::Quit);
impl_sig!(Ill, Signal::Ill);
impl_sig!(Trap, Signal::Trap);
impl_sig!(Abort, Signal::Abort);
impl_sig!(Bus, Signal::Bus);
impl_sig!(Fpe, Signal::Fpe);
impl_sig!(Kill, Signal::Kill);
impl_sig!(Usr1, Signal::Usr1);
impl_sig!(Segv, Signal::Segv);
impl_sig!(Usr2, Signal::Usr2);
impl_sig!(Pipe, Signal::Pipe);
impl_sig!(Alarm, Signal::Alarm);
impl_sig!(Term, Signal::Term);
impl_sig!(Child, Signal::Child);
impl_sig!(Cont, Signal::Cont);
impl_sig!(Stop, Signal::Stop);
impl_sig!(Tstp, Signal::Tstp);
impl_sig!(Ttin, Signal::Ttin);
impl_sig!(Ttou, Signal::Ttou);
impl_sig!(Urg, Signal::Urg);
impl_sig!(Xcpu, Signal::Xcpu);
impl_sig!(Xfsz, Signal::Xfsz);
impl_sig!(Vtalarm, Signal::Vtalarm);
impl_sig!(Prof, Signal::Prof);
impl_sig!(Winch, Signal::Winch);
impl_sig!(Io, Signal::Io);
impl_sig!(Sys, Signal::Sys);
