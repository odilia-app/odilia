use core::{
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
    mem::replace,
};
use tower::Service;

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
	Iter: IntoIterator<Item = I>,
	S2: Service<I> + Clone,
	E: From<S1::Error> + From<S2::Error>,
{
	type Response = Vec<S2::Response>;
	type Error = E;
	type Future = impl Future<Output = Result<Self::Response, Self::Error>>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		let _ = self.inner.poll_ready(cx).map_err(Into::<E>::into)?;
		self.outer.poll_ready(cx).map_err(Into::into)
	}
	fn call(&mut self, input: Req) -> Self::Future {
		let clone_outer = self.outer.clone();
		let mut outer = replace(&mut self.outer, clone_outer);
		let clone_inner = self.inner.clone();
		let mut inner = replace(&mut self.inner, clone_inner);
		async move {
			let iter = inner.call(input).await?;
			let mut results = vec![];
			for item in iter {
				let result = outer.call(item).await?;
				results.push(result);
			}
			Ok(results)
		}
	}
}
