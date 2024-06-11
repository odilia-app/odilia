#![allow(clippy::module_name_repetitions)]

use odilia_cache::Cache;
use std::{
	marker::PhantomData,
	sync::Arc,
	task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct CacheLayer<I> {
	cache: Arc<Cache>,
	_marker: PhantomData<I>,
}
impl<I> Clone for CacheLayer<I> {
	fn clone(&self) -> Self {
		CacheLayer { cache: Arc::clone(&self.cache), _marker: PhantomData }
	}
}
impl<I> CacheLayer<I> {
	pub fn new(cache: Arc<Cache>) -> Self {
		CacheLayer { cache, _marker: PhantomData }
	}
}
impl<S, I> Layer<S> for CacheLayer<I>
where
	S: Service<(I, Arc<Cache>)>,
{
	type Service = CacheService<S, I>;
	fn layer(&self, inner: S) -> CacheService<S, I> {
		CacheService { inner, cache: Arc::clone(&self.cache), _marker: PhantomData }
	}
}
pub struct CacheService<S, I> {
	inner: S,
	cache: Arc<Cache>,
	_marker: PhantomData<fn(I)>,
}
impl<I, S> Service<I> for CacheService<S, I>
where
	S: Service<(I, Arc<Cache>)>,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future = S::Future;
	fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.inner.poll_ready(cx)
	}
	fn call(&mut self, req: I) -> Self::Future {
		self.inner.call((req, Arc::clone(&self.cache)))
	}
}
