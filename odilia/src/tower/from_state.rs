use crate::{
	tower::async_try::{AsyncTryFrom, AsyncTryInto},
	ScreenReaderState,
};
use atspi::{EventProperties, EventTypeProperties};
use futures::{
	future::{ok, ErrInto, Map},
	join, FutureExt, TryFutureExt,
};
use futures_concurrency::future::Join;
use futures_concurrency::prelude::*;

use odilia_common::{command::CommandType, errors::OdiliaError};
use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;

pub trait FromState<S, T>: Sized {
	async fn from_state(state: S, data: T) -> Self;
}

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

pub trait EventTryFromState<S, E>: TryFromState<S, E>
where
	E: EventProperties,
{
}
impl<T, S, E> EventTryFromState<S, E> for T
where
	T: TryFromState<S, E>,
	E: EventProperties,
{
}

pub trait CommandTryFromState<S, C>: TryFromState<S, C>
where
	C: CommandType,
{
}
impl<T, S, C> CommandTryFromState<S, C> for T
where
	T: TryFromState<S, C>,
	C: CommandType,
{
}
