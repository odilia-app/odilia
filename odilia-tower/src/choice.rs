use alloc::collections::BTreeMap;
use core::{
	fmt::Debug,
	marker::PhantomData,
	mem::replace,
	task::{Context, Poll},
};
use futures::{
	future::{err, Either, ErrInto, Ready},
	TryFutureExt,
};
use tower::Service;

use crate::Error;

pub trait Chooser<K> {
	fn identifier(&self) -> K;
}

#[allow(clippy::module_name_repetitions)]
pub struct ChoiceService<K, S, Req, E>
where
	S: Service<Req>,
	Req: Chooser<K>,
{
	services: BTreeMap<K, S>,
	_marker: PhantomData<fn(Req) -> E>,
}

impl<K, S, Req, E> Clone for ChoiceService<K, S, Req, E>
where
	K: Clone,
	S: Clone + Service<Req>,
	Req: Chooser<K>,
{
	fn clone(&self) -> Self {
		ChoiceService { services: self.services.clone(), _marker: PhantomData }
	}
}

impl<K, S, Req, E> Default for ChoiceService<K, S, Req, E>
where
	S: Service<Req>,
	Req: Chooser<K>,
{
	fn default() -> Self {
		ChoiceService { services: BTreeMap::new(), _marker: PhantomData }
	}
}

impl<K, S, Req, E> ChoiceService<K, S, Req, E>
where
	S: Service<Req>,
	Req: Chooser<K>,
{
	pub fn new() -> Self {
		ChoiceService { services: BTreeMap::new(), _marker: PhantomData }
	}
}

impl<K, S, Req, E> Service<Req> for ChoiceService<K, S, Req, E>
where
	S: Service<Req> + Clone,
	Req: Chooser<K>,
	K: Ord + Debug,
	E: From<S::Error> + From<crate::Error>,
{
	type Response = S::Response;
	type Error = E;
	type Future = Either<Ready<Result<S::Response, E>>, ErrInto<S::Future, E>>;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		for (_k, svc) in &mut self.services.iter_mut() {
			let _ = svc.poll_ready(cx)?;
		}
		Poll::Ready(Ok(()))
	}
	fn call(&mut self, req: Req) -> Self::Future {
		let k = req.identifier();

		let mut svc = if let Some(orig_svc) = self.services.get_mut(&k) {
			let clone = orig_svc.clone();
			replace(orig_svc, clone)
		} else {
			return Either::Left(err(Error::ServiceNotFound(
                format!("A service with key {k:?} could not be found in a list with keys of {:?}", self.services.keys())
            ).into()));
		};
		Either::Right(svc.call(req).err_into())
	}
}
