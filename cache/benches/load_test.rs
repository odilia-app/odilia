use std::time::Duration;

use atspi::{accessible_id::AccessibleId, AccessibilityConnection};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use odilia_cache::{AccessiblePrimitive, Cache, CacheItem};
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

/// For each child, fetch it and all of its ancestors via `Cache::get`.
//
// Note: may be able to reduce noise by just doing the deepest child
fn traverse_cache(cache: &Cache, children: Vec<AccessiblePrimitive>) {
	// for each child, try going up to the root
	for child in children {
		let mut key = child;
		loop {
			let item = cache.get(&key).unwrap();
			key = item.parent.key;
			if matches!(key.id, AccessibleId::Root) {
				break;
			}
		}
	}
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
	group.sample_size(500) // def 100
		.significance_level(0.05) // def 0.05
		.noise_threshold(0.03) // def 0.01
		.measurement_time(Duration::from_secs(15));

	let cache = Cache::new(zbus_connection.clone());
	group.bench_function(BenchmarkId::new("add_all", "zbus-docs"), |b| {
		b.iter_batched(
			|| zbus_items.clone(),
			|items: Vec<CacheItem>| add_all(&cache, items),
			BatchSize::SmallInput,
		);
	});

	let cache = Cache::new(zbus_connection.clone());
	group.bench_function(BenchmarkId::new("add", "zbus-docs"), |b| {
		b.iter_batched(
			|| zbus_items.clone(),
			|items: Vec<CacheItem>| add(&cache, items),
			BatchSize::SmallInput,
		);
	});

	let (cache, children): (Cache, Vec<AccessiblePrimitive>) = rt.block_on(async {
		let all_items = wcag_items.clone();
		let children = all_items
			.iter()
			.filter_map(|item| {
				(item.parent.key.id != AccessibleId::Null)
					.then_some(item.object.clone())
			})
			.collect();
		let cache = Cache::new(zbus_connection.clone());
		cache.add_all(all_items);
		(cache, children)
	});
	group.bench_function(BenchmarkId::new("traverse_cache", "wcag-items"), |b| {
		b.iter_batched(
			|| children.clone(),
			|cs| traverse_cache(&cache, cs),
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
