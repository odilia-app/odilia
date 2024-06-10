use crate::{
    ScreenReaderState,
    tower::async_try::AsyncTryFrom,
};
use futures::{
    join,
    TryFutureExt,
    FutureExt,
    future::ErrInto,
};

use odilia_common::{
    errors::OdiliaError,
};
use std::{
    future::Future,
};

pub trait FromState<T>: Sized + Send {
	type Error: Send;
	type Future: Future<Output = Result<Self, Self::Error>> + Send;
	fn try_from_state(state: &ScreenReaderState, t: &T) -> Self::Future;
}

impl<T, U: FromState<T>> AsyncTryFrom<(&ScreenReaderState, &T)> for U
where
	<U as FromState<T>>::Error: Into<OdiliaError>,
{
	type Error = OdiliaError;
	type Future = ErrInto<U::Future, Self::Error>;
	fn try_from_async(state: (&ScreenReaderState, &T)) -> Self::Future {
		U::try_from_state(state.0, state.1).err_into()
	}
}

macro_rules! impl_from_state {
($(($type:ident,$err:ident),)+) => {
    #[allow(non_snake_case)]
    impl<I, $($type, $err,)+> FromState<I> for ($($type,)+)
    where
        $($type: FromState<I, Error = $err>,)+
        $(OdiliaError: From<$err>,)+
        $($err: Send,)+
        {
            type Error = OdiliaError;
            type Future = impl Future<Output = Result<Self, Self::Error>>;
            fn try_from_state(state: &ScreenReaderState, i: &I) -> Self::Future {
                $(let $type = <$type>::try_from_state(state, i);)+
                async {
                    join!(
                        $($type,)+
                    )
                }
                .map(|($($type,)+)| {
                    Ok((
                        $($type?,)+
                    ))
                })
            }
        }
    }
}

impl_from_state!((T1, E1),);
impl_from_state!((T1, E1), (T2, E2),);
impl_from_state!((T1, E1), (T2, E2), (T3, E3),);
impl_from_state!((T1, E1), (T2, E2), (T3, E3), (T4, E4),);
impl_from_state!((T1, E1), (T2, E2), (T3, E3), (T4, E4), (T5, E5),);
impl_from_state!((T1, E1), (T2, E2), (T3, E3), (T4, E4), (T5, E5), (T6, E6),);

