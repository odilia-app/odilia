#![allow(clippy::module_name_repetitions)]

use futures::{future::ErrInto, TryFutureExt};
use futures_concurrency::future::TryJoin;

use crate::errors::OdiliaError;
use core::future::Future;

pub trait TryFromState<S, T>: Sized {
	type Error;
	type Future: Future<Output = Result<Self, Self::Error>>;
	fn try_from_state(state: S, data: T) -> Self::Future;
}

macro_rules! impl_try_from_state {
    ($($type:ident,)+) => {
        impl<S, T, $($type,)+> TryFromState<S, T> for ($($type,)+)
        where
              $($type: TryFromState<S, T>,)+
              $(OdiliaError: From<$type::Error>,)+
              T: Clone,
              S: Clone {
            type Error = OdiliaError;
            type Future = <($(ErrInto<$type::Future, OdiliaError>,)+) as TryJoin>::Future;
            fn try_from_state(state: S, data: T) -> Self::Future {
                (
                  $($type::try_from_state(state.clone(), data.clone()).err_into(),)+
                )
                  .try_join()
            }
        }
    }
}

impl_try_from_state!(U1,);
impl_try_from_state!(U1, U2,);
impl_try_from_state!(U1, U2, U3,);
impl_try_from_state!(U1, U2, U3, U4,);
impl_try_from_state!(U1, U2, U3, U4, U5,);
