/// A simple macro that extracts an [`CacheValue`] from a [`CacheRef`].
macro_rules! get_cache_item {
	($cache_ref:expr) => {
		$cache_ref
			.upgrade()
			.ok_or_else(|| {
				tracing::trace!("Item removed from cache before upgraded from weak reference.");
				CacheError::NoItem
			})?
	}
}
