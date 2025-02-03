use crate::{
	future_ext::FutureExt as CrateFutureExt, future_ext::MapFutureMultiSet,
	service_multi_iter::ServiceMultiIter, service_multiset::ServiceMultiset, MapMExt,
	ServiceSet,
};
use alloc::vec::Vec;
use core::{
	future::{ready, Future, IntoFuture},
	iter::{repeat, Repeat},
	marker::PhantomData,
	mem::replace,
	pin::Pin,
	task::{Context, Poll},
};
use futures::{
	future::{join, join_all, ErrInto, Flatten},
	stream::iter,
	FutureExt, TryFutureExt,
};
use tower::util::Oneshot;
use tower::Service;
use tower::ServiceExt;

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
	//TODO erase:
	Req: Clone,
{
	type Response = Vec<<S2::Future as Future>::Output>;
	type Error = E;
	//type Future = impl Future<Output = Result<Self::Response, Self::Error>>;
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
		let mut inner = replace(&mut self.inner, clone_inner);
		let fut = inner.oneshot(input).err_into();
		let x =
		// TODO: make a map_into_service derivative that takes a clonable service and a future of some
		// kind, then gets them to complete concurrently
             <futures::future::ErrInto<Oneshot<S1, Req>, E> as crate::future_ext::FutureExt<Result<Iter, E>, E>>::map_future_multiset::<S2, Iter, I, E>(fut, outer)
             .flatten();
		x
		/*
		async move {
			let x = inner.call(input).await?;
			let res = repeat(outer).zip(x).map_service_call();
			Ok(join_all(res).await)
		}
		inner.call(input)
		    .map_ok(move |iter| {
			outers
			    .zip(iter.into_iter())
			    .map_m::<S2, I, S2::Response>()
		    })
		    .ok_join_all()
		*/
		//join_all(services.into_iter().zip(req_rep).map_m()).map_ok()
		/*
			    async move {
				    let iter = inner.call(input).await?;
				    let mut results = Vec::new();
				    for item in iter {
					    let result = outer.call(item).await?;
					    results.push(result);
				    }
				    Ok(results)
			    }
		*/
	}
}
/*

#[allow(clippy::type_complexity)]
pub struct IterService2<'a, F, Req, Iter, I, S2, E> {
	inner: Pin<&'a mut F>,
  inner_res: Option<Iter>,
	outer: ServiceSet<S2>,
	_marker: PhantomData<fn(Req) -> Result<(Iter, I), E>>,
}
impl<'a, F, Req, Iter, I, S2, E> Clone for IterService2<'a, F, Req, Iter, I, S2, E>
where
  Iter: Clone,
	Pin<&'a mut F>: Clone,
	ServiceSet<S2>: Clone,
{
	fn clone(&self) -> Self {
		IterService2 {
			inner: self.inner.clone(),
			outer: self.outer.clone(),
      inner_res: self.inner_res.clone(),
			_marker: PhantomData,
		}
	}
}
impl<'a, F, Req, Iter, I, S2, E> IterService2<'a, F, Req, Iter, I, S2, E>
where
  F: Future<Output = Iter>,
	Iter: IntoIterator<Item = I>,
	S2: Service<I>,
{
	pub fn new(inner: Pin<&'a mut F>, outer: S2) -> Self {
		IterService2 { inner, inner_res: None, outer: ServiceSet::from(outer), _marker: PhantomData }
	}
}

impl<F, Req, Iter, I, S2, E> Service<I> for IterService2<'_, F, Req, Iter, I, S2, E>
where
  F: Future<Output = Iter>,
	Iter: Iterator<Item = I>,
	S2: Service<I> + Clone,
  I: Clone,
	E: From<S2::Error>,
{
	type Response = <ServiceSet<S2> as Service<I>>::Response;
	type Error = E;
	type Future = ErrInto<<ServiceSet<S2> as Service<I>>::Future, E>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    let f = self.inner.as_mut();
    // is we already have the result, check the inner services
    if self.inner_res.is_some() {
	let _ = self.outer.poll_ready(cx)?;
	return Poll::Ready(Ok(()));
    }
    let mut res = match f.poll(cx) {
	Poll::Pending => return Poll::Pending,
	Poll::Ready(r) => r,
    };
    self.outer.clone_expand(res.by_ref().count());
    self.inner_res = Some(res);
    // this is fine because the only case we get here is if:
    // 1. self.inner_res _was none_, and
    // 2. `f.poll(cx)` returned the ready result
    // Therefore, it only happens once, then the function short-circuits on the outer services
    // being ready.
    self.poll_ready(cx)
	}
	fn call(&mut self, input: I) -> Self::Future {
      self.outer.call(input)
	  .err_into()
	}
}
*/
