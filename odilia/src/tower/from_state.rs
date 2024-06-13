use futures::FutureExt;
use futures_concurrency::future::Join;

use odilia_common::errors::OdiliaError;
use std::future::Future;

pub trait TryFromState<S, T>: Sized {
	type Error;
	type Future: Future<Output = Result<Self, Self::Error>>;
	fn try_from_state(state: S, data: T) -> Self::Future;
}

impl<S, T, U1> TryFromState<S, T> for (U1,)
where
	U1: TryFromState<S, T>,
	OdiliaError: From<U1::Error>,
{
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<(U1,), OdiliaError>>;
	fn try_from_state(state: S, data: T) -> Self::Future {
		(U1::try_from_state(state, data),).join().map(|(u1,)| Ok((u1?,)))
	}
}
impl<S, T, U1, U2> TryFromState<S, T> for (U1, U2)
where
	U1: TryFromState<S, T>,
	U2: TryFromState<S, T>,
	OdiliaError: From<U1::Error> + From<U2::Error>,
	S: Clone,
	T: Clone,
{
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<(U1, U2), OdiliaError>>;
	fn try_from_state(state: S, data: T) -> Self::Future {
		(U1::try_from_state(state.clone(), data.clone()), U2::try_from_state(state, data))
			.join()
			.map(|(u1, u2)| Ok((u1?, u2?)))
	}
}
impl<S, T, U1, U2, U3> TryFromState<S, T> for (U1, U2, U3)
where
	U1: TryFromState<S, T>,
	U2: TryFromState<S, T>,
	U3: TryFromState<S, T>,
	OdiliaError: From<U1::Error> + From<U2::Error> + From<U3::Error>,
	S: Clone,
	T: Clone,
{
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<(U1, U2, U3), OdiliaError>>;
	fn try_from_state(state: S, data: T) -> Self::Future {
		(
			U1::try_from_state(state.clone(), data.clone()),
			U2::try_from_state(state.clone(), data.clone()),
			U3::try_from_state(state, data),
		)
			.join()
			.map(|(u1, u2, u3)| Ok((u1?, u2?, u3?)))
	}
}
