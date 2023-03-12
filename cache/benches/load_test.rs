use std::{
	sync::{Arc, Mutex},
	time::Duration,
};

use atspi::{accessible::Accessible, accessible_id::AccessibleId, AccessibilityConnection};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use odilia_cache::{clone_arc_mutex, AccessiblePrimitive, Cache, CacheItem};

use odilia_common::errors::{CacheError, OdiliaError};
use rand::seq::SliceRandom;
use tokio::select;
use tokio_test::block_on;

macro_rules! load_items {
	($file:expr) => {
		::serde_json::from_str(include_str!($file)).unwrap()
	};
}

/// Load the given items into cache via `Cache::add_all`.
fn add_all(cache: &Cache, items: Vec<CacheItem>) {
	cache.add_all(items);
}

/// Load the given items into cache via repeated `Cache::add`.
/// Note: now that concurrency is handled by dashmap and there is no outer lock
/// on the hashmap, this should be the same as `add_all`.
fn add(cache: &Cache, items: Vec<CacheItem>) {
	for item in items {
		cache.add(item);
	}
}

/// For each child, fetch all of its ancestors via `CacheItem::parent_ref`.
async fn traverse_up_refs(children: Vec<Arc<Mutex<CacheItem>>>) {
	// for each child, try going up to the root
	for child_ref in children {
		let mut item_ref = child_ref;
		loop {
			let item_ref_copy = Arc::clone(&item_ref);
			let mut item = item_ref_copy.lock().expect("Could not lock item");
			if matches!(item.object.id, AccessibleId::Root) {
				break;
			}
			item_ref = item.parent_ref().expect("Could not get parent reference");
		}
	}
}

/// For each child, fetch all of its ancestors in full (cloned) via
/// `Accessible::parent`.
async fn traverse_up(children: Vec<CacheItem>) {
	// for each child, try going up to the root
	for child in children {
		let mut item = child;
		loop {
			item = match item.parent().await {
				Ok(item) => item,
				Err(OdiliaError::Cache(CacheError::NoItem)) => {
					// Missing item from cache; there's always exactly one.
					// Perhaps an item pointing to a special root/null node gets
					// through? Not super important.
					break;
				}
				Err(e) => {
					panic!("Odilia error {:?}", e);
				}
			};
			if matches!(item.object.id, AccessibleId::Root) {
				break;
			}
		}
	}
}

/// Depth first traversal
fn traverse_depth_first(root: CacheItem) -> Result<(), OdiliaError> {
	for child in root.get_children()? {
		traverse_depth_first(child)?;
	}
	Ok(())
}

/// Observe throughput of reads (`Cache::get`) while writing to cache
/// (`Cache::add`).
async fn reads_while_writing(cache: &Cache, ids: Vec<AccessiblePrimitive>, items: Vec<CacheItem>) {
	let cache_1 = cache.clone();
	let mut write_handle = tokio::spawn(async move {
		for item in items {
			cache_1.add(item);
		}
	});
	let cache_2 = cache.clone();
	let mut read_handle = tokio::spawn(async move {
		for id in ids {
			cache_2.get(&id);
		}
	});
	let mut write_finished = false;
	loop {
		select! {
		    // we don't care when the write finishes, keep looping
		    _ = &mut write_handle, if !write_finished => write_finished = true,
		    // return as soon as we're done with these reads
		    _ = &mut read_handle => break
		}
	}
}

fn cache_benchmark(c: &mut Criterion) {
	let rt = tokio::runtime::Runtime::new().unwrap();
	let a11y = block_on(AccessibilityConnection::open()).unwrap();
	let zbus_connection = a11y.connection();

	let zbus_items: Vec<CacheItem> = load_items!("./zbus_docs_cache_items.json");
	let wcag_items: Vec<CacheItem> = load_items!("./wcag_cache_items.json");

	let mut group = c.benchmark_group("cache");
	group.sample_size(200) // def 100
		.significance_level(0.05) // def 0.05
		.noise_threshold(0.03) // def 0.01
		.measurement_time(Duration::from_secs(20));

	let cache = Arc::new(Cache::new(zbus_connection.clone()));
	group.bench_function(BenchmarkId::new("add_all", "zbus-docs"), |b| {
		b.to_async(&rt).iter_batched(
			|| {
				zbus_items
					.clone()
					.into_iter()
					.map(|mut item| {
						item.cache = Arc::downgrade(&cache);
						item
					})
					.collect()
			},
			|items: Vec<CacheItem>| async { add_all(&cache, items) },
			BatchSize::SmallInput,
		);
	});

	let cache = Arc::new(Cache::new(zbus_connection.clone()));
	group.bench_function(BenchmarkId::new("add", "zbus-docs"), |b| {
		b.to_async(&rt).iter_batched(
			|| {
				zbus_items
					.clone()
					.into_iter()
					.map(|mut item| {
						item.cache = Arc::downgrade(&cache);
						item
					})
					.collect()
			},
			|items: Vec<CacheItem>| async { add(&cache, items) },
			BatchSize::SmallInput,
		);
	});

	let (cache, children): (Arc<Cache>, Vec<Arc<Mutex<CacheItem>>>) = rt.block_on(async {
		let cache = Arc::new(Cache::new(zbus_connection.clone()));
		let all_items: Vec<CacheItem> = wcag_items
			.clone()
			.into_iter()
			.map(|mut item| {
				item.cache = Arc::downgrade(&cache);
				item
			})
			.collect();
		cache.add_all(all_items);
		let children = cache
			.by_id
			.iter()
			.filter_map(|entry| {
				if entry.lock().unwrap().children.is_empty() {
					Some(Arc::clone(&entry))
				} else {
					None
				}
			})
			.collect();
		(cache, children)
	});
	group.bench_function(BenchmarkId::new("traverse_up_refs", "wcag-items"), |b| {
		b.to_async(&rt).iter_batched(
			|| children.clone(),
			|cs| async { traverse_up_refs(cs).await },
			BatchSize::SmallInput,
		);
	});

	group.bench_function(BenchmarkId::new("traverse_up", "wcag-items"), |b| {
		b.to_async(&rt).iter_batched(
			|| children.iter().map(clone_arc_mutex).collect(),
			|cs| async { traverse_up(cs).await },
			BatchSize::SmallInput,
		);
	});

	group.bench_function(BenchmarkId::new("traverse_depth_first", "wcag-items"), |b| {
		b.iter_batched(
			|| {
				cache.get(&AccessiblePrimitive {
					id: AccessibleId::Root,
					sender: ":1.22".into(),
				})
				.unwrap()
			},
			traverse_depth_first,
			BatchSize::SmallInput,
		);
	});

	let mut rng = &mut rand::thread_rng();
	let cache = Cache::new(zbus_connection.clone());
	let all_items = zbus_items.clone();
	for size in [10, 100, 1000] {
		let sample = all_items
			.choose_multiple(&mut rng, size as usize)
			.map(|item| item.object.clone())
			.collect::<Vec<_>>();
		group.throughput(criterion::Throughput::Elements(size));
		group.bench_function(BenchmarkId::new("reads_while_writing", size), |b| {
			b.to_async(&rt).iter_batched(
				|| (sample.clone(), all_items.clone()),
				|(ids, items)| async {
					reads_while_writing(&cache, ids, items).await
				},
				BatchSize::SmallInput,
			);
		});
	}

	group.finish();
}

criterion_group!(benches, cache_benchmark);
criterion_main!(benches);
