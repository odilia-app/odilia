use futures::future::err;
use futures::future::Either;
use futures::TryFutureExt;
use odilia_common::errors::OdiliaError;
use std::collections::{btree_map::Entry, BTreeMap};
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use tower::Service;

pub trait Chooser<K> {
	fn identifier(&self) -> K;
}

pub struct ChoiceService<K, S, Req>
where
	S: Service<Req>,
	Req: Chooser<K>,
{
	services: BTreeMap<K, S>,
	_marker: PhantomData<Req>,
}

impl<K, S, Req> ChoiceService<K, S, Req>
where
	S: Service<Req>,
	Req: Chooser<K>,
{
	pub fn new() -> Self {
		ChoiceService { services: BTreeMap::new(), _marker: PhantomData }
	}
	pub fn insert(&mut self, k: K, s: S)
	where
		K: Ord,
	{
		self.services.insert(k, s);
	}
	pub fn entry(&mut self, k: K) -> Entry<K, S>
	where
		K: Ord,
	{
		self.services.entry(k)
	}
}

impl<K, S, Req> Service<Req> for ChoiceService<K, S, Req>
where
	S: Service<Req> + Clone,
	Req: Chooser<K>,
	K: Ord + Debug,
	OdiliaError: From<S::Error>,
{
	type Response = S::Response;
	type Error = OdiliaError;
	type Future = impl Future<Output = Result<Self::Response, Self::Error>>;
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
			std::mem::replace(orig_svc, clone)
		} else {
			return Either::Left(err(OdiliaError::ServiceNotFound(
                format!("A service with key {k:?} could not be found in a list with keys of {:?}", self.services.keys())
            )));
		};
		Either::Right(svc.call(req).err_into())
	}
}
