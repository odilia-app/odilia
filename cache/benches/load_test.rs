use std::{collections::VecDeque, hint::black_box, sync::Arc, time::Duration};

use atspi_connection::AccessibilityConnection;
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use indextree::{Arena, NodeId};
use odilia_cache::{Cache, CacheItem};

use odilia_common::{cache::AccessiblePrimitive, errors::OdiliaError};
use tokio::select;
use tokio_test::block_on;

macro_rules! load_items {
	($file:expr) => {
		::serde_json::from_str(include_str!($file)).unwrap()
	};
}

/// Load the given items into cache via `Cache::add_all`.
/// This is different from `add` in that it postpones populating references
/// until after all items have been added.
fn add_all(cache: &Cache, items: Vec<CacheItem>) {
	let _ = cache.add_all(items);
}

/// Load the given items into cache via repeated `Cache::add`.
fn add(cache: &Cache, items: Vec<CacheItem>) {
	for item in items {
		let _ = cache.add(item);
	}
}

const ROOT_A11Y: &str = "/org/a11y/atspi/accessible/root";

/// For each child, fetch all of its ancestors via `NodeId::ancestors`.
async fn traverse_up_refs(children: Vec<NodeId>, arena: &Arena<CacheItem>) {
	// for each child, try going up to the root
	for child_ref in children {
		child_ref.ancestors(arena).for_each(|anc| {
			let _ = black_box(anc);
		});
	}
}

/// Depth first traversal
fn traverse_depth_first((root, cache): (NodeId, &Cache)) -> Result<(), OdiliaError> {
	let lock = cache.tree.read();
	for child in root.descendants(&lock) {
		black_box(child);
	}
	Ok(())
}

/// Observe throughput of successful reads (`Cache::get`) while writing to cache
/// (`Cache::add_all`).
async fn reads_while_writing(cache: Cache, ids: Vec<AccessiblePrimitive>, items: Vec<CacheItem>) {
	let cache_1 = Arc::new(cache);
	let cache_2 = Arc::clone(&cache_1);
	let mut write_handle = tokio::spawn(async move {
		let _ = cache_1.add_all(items);
	});
	let mut read_handle = tokio::spawn(async move {
		let mut ids = VecDeque::from(ids);
		loop {
			match ids.pop_front() {
				None => break, // we're done
				Some(id) => {
					if cache_2.id_lookup.get(&id).is_none() {
						ids.push_back(id);
					}
				}
			}
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
	let a11y = block_on(AccessibilityConnection::new()).unwrap();
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
			|| zbus_items.clone(),
			|items: Vec<CacheItem>| async {
				let _ = cache.clear();
				add_all(&cache, items);
			},
			BatchSize::SmallInput,
		);
	});
	let cache = Arc::new(Cache::new(zbus_connection.clone()));
	group.bench_function(BenchmarkId::new("add_all", "wcag-docs"), |b| {
		b.to_async(&rt).iter_batched(
			|| wcag_items.clone(),
			|items: Vec<CacheItem>| async {
				let _ = cache.clear();
				add_all(&cache, items);
			},
			BatchSize::SmallInput,
		);
	});

	let cache = Arc::new(Cache::new(zbus_connection.clone()));
	group.bench_function(BenchmarkId::new("add", "zbus-docs"), |b| {
		b.to_async(&rt).iter_batched(
			|| zbus_items.clone(),
			|items: Vec<CacheItem>| async { add(&cache, items) },
			BatchSize::SmallInput,
		);
	});

	let (cache, children): (Arc<Cache>, Vec<NodeId>) = rt.block_on(async {
		let cache = Arc::new(Cache::new(zbus_connection.clone()));
		let all_items: Vec<CacheItem> = wcag_items.clone();
		let _ = cache.add_all(all_items);
		let read_cache = cache.tree.read();
		let children = read_cache
			.iter()
			.filter_map(|entry| {
				if entry.first_child().is_none() {
					read_cache.get_node_id(entry)
				} else {
					None
				}
			})
			.collect();
		drop(read_cache);
		(cache, children)
	});
	let read_cache = cache.tree.read();
	group.bench_function(BenchmarkId::new("traverse_up_refs", "wcag-items"), |b| {
		b.to_async(&rt).iter_batched(
			|| children.clone(),
			|cs| async { traverse_up_refs(cs, &read_cache).await },
			BatchSize::SmallInput,
		);
	});

	group.bench_function(BenchmarkId::new("traverse_depth_first", "wcag-items"), |b| {
		b.iter_batched(
			|| {
				(
					*cache.id_lookup
						.get(&AccessiblePrimitive {
							id: ROOT_A11Y.to_string(),
							sender: ":1.30".into(),
						})
						.unwrap(),
					&cache,
				)
			},
			traverse_depth_first,
			BatchSize::SmallInput,
		);
	});

	let all_items = wcag_items.clone();
	for size in [10, 100, 1000] {
		let sample = all_items[0..size]
			.iter()
			.map(|item| item.object.clone())
			.collect::<Vec<_>>();
		group.throughput(criterion::Throughput::Elements(size as u64));
		group.bench_function(BenchmarkId::new("reads_while_writing", size), |b| {
			b.to_async(&rt).iter_batched(
				|| {
					(
						Cache::new(zbus_connection.clone()),
						sample.clone(),
						all_items.clone(),
					)
				},
				|(cache, ids, items)| async {
					reads_while_writing(cache, ids, items).await
				},
				BatchSize::SmallInput,
			);
		});
	}

	group.finish();
}

criterion_group!(benches, cache_benchmark);
criterion_main!(benches);
