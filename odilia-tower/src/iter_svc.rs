use core::{
	future::Future,
	marker::PhantomData,
	mem::replace,
	task::{Context, Poll},
};
use futures::{
	future::{ErrInto, Flatten},
	FutureExt, TryFutureExt,
};
use tower::{util::Oneshot, Service, ServiceExt};
use tower_iter::future_ext::{FutureExt as TowerIterFutureExt, MapFutureMultiSet};

#[allow(clippy::type_complexity)]
pub struct IterService<S1, Req, Iter, I, S2, E> {
	inner: S1,
	outer: S2,
	_marker: PhantomData<fn(Req) -> Result<(Iter, I), E>>,
}
impl<S1, Req, Iter, I, S2, E> Clone for IterService<S1, Req, Iter, I, S2, E>
where
	S1: Clone,
	S2: Clone,
{
	fn clone(&self) -> Self {
		IterService {
			inner: self.inner.clone(),
			outer: self.outer.clone(),
			_marker: PhantomData,
		}
	}
}
impl<S1, Req, Iter, I, S2, E> IterService<S1, Req, Iter, I, S2, E>
where
	S1: Service<Req, Response = Iter>,
	Iter: IntoIterator<Item = I>,
	S2: Service<I>,
{
	pub fn new(inner: S1, outer: S2) -> Self {
		IterService { inner, outer, _marker: PhantomData }
	}
}

impl<S1, Req, Iter, I, S2, E> Service<Req> for IterService<S1, Req, Iter, I, S2, E>
where
	S1: Service<Req, Response = Iter> + Clone,
	Iter: Iterator<Item = I>,
	S2: Service<I> + ServiceExt<I> + Clone,
	E: From<S1::Error> + From<S2::Error>,
{
	type Response = Vec<<S2::Future as Future>::Output>;
	type Error = E;
	type Future = Flatten<
		MapFutureMultiSet<futures::future::ErrInto<Oneshot<S1, Req>, E>, S2, Iter, I, E>,
	>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		let _ = self.inner.poll_ready(cx).map_err(Into::<E>::into)?;
		self.outer.poll_ready(cx).map_err(Into::into)
	}
	fn call(&mut self, input: Req) -> Self::Future {
		let clone_outer = self.outer.clone();
		let outer = replace(&mut self.outer, clone_outer);
		let clone_inner = self.inner.clone();
		let inner = replace(&mut self.inner, clone_inner);
		let fut = inner.oneshot(input).err_into();

		<ErrInto<Oneshot<S1, Req>, E> as TowerIterFutureExt<
                                                            Result<Iter, E>,
                                                                        E,
                                                                                >>::map_future_multiset::<S2, Iter, I, E>(fut, outer)
                                                                                        .flatten()
	}
}
