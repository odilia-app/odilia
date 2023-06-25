use std::{
	collections::VecDeque,
	sync::{Arc, Weak},
	time::Duration,
};

use atspi_connection::AccessibilityConnection;
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use odilia_cache::{Cache, CacheItem};

use odilia_common::{errors::{OdiliaError, CacheError}, cache::{AccessiblePrimitive}};
use tokio::select;
use tokio::sync::Mutex;
use tokio_test::block_on;

macro_rules! load_items {
	($file:expr) => {
		::serde_json::from_str(include_str!($file)).unwrap()
	};
}

/// Load the given items into cache via `Cache::add_all`.
/// This is different from `add` in that it postpones populating references
/// until after all items have been added.
async fn add_all(cache: &Cache, items: Vec<CacheItem>) {
	let _ = cache.add_all(items).await;
}

/// Load the given items into cache via repeated `Cache::add`.
async fn add(cache: &Cache, items: Vec<CacheItem>) {
	for item in items {
		let _ = cache.add(item).await;
	}
}

const ROOT_A11Y: &str = "/org/a11y/atspi/accessible/root";

/// For each child, fetch all of its ancestors via `CacheItem::parent_ref`.
async fn traverse_up_refs(children: Vec<Arc<Mutex<CacheItem>>>, cache: &Cache) {
	// for each child, try going up to the root
	for child_ref in children {
		let mut item_ref = child_ref;
		loop {
			let item_ref_copy = Arc::clone(&item_ref);
			let item = item_ref_copy.lock().await;
			let root = ROOT_A11Y.to_string();
			if matches!(&item.object.id, root) {
				break;
			}
			item_ref = cache.get_ref(&item.parent.key).expect("Could not get parent reference");
		}
	}
}

/// For each child, fetch all of its ancestors in full (cloned) via
/// `Accessible::parent`.
async fn traverse_up(children: Vec<Arc<Mutex<CacheItem>>>, cache: &Cache) {
	// for each child, try going up to the root
	for child in children {
		let mut item = child.lock().await.clone();
		loop {
			item = match cache.get_ref(&item.parent.key) {
				Some(i_item) => i_item.lock().await.clone(),
				None => {
					panic!("Fatal error: could not find item in cache!");
				}
			};
			let root = ROOT_A11Y.to_string();
			if matches!(&item.object.id, root) {
				break;
			}
		}
	}
}

/// Depth first traversal
fn traverse_depth_first(data: (CacheItem, &Cache)) -> Result<(), OdiliaError> {
	for child in data.0.children {
		let child_item = match Weak::upgrade(&child.item) {
			Some(arc) => block_on(arc.lock()).clone(),
			None => block_on(data.1.get(&child.key)).clone().ok_or::<OdiliaError>(CacheError::NoItem.into())?,
		};
		traverse_depth_first((child_item, data.1))?;
	}
	Ok(())
}

/// Observe throughput of successful reads (`Cache::get`) while writing to cache
/// (`Cache::add_all`).
async fn reads_while_writing(cache: Cache, ids: Vec<AccessiblePrimitive>, items: Vec<CacheItem>) {
	let cache_1 = Arc::new(cache);
	let cache_2 = Arc::clone(&cache_1);
	let mut write_handle = tokio::spawn(async move {
		let _ = cache_1.add_all(items).await;
	});
	let mut read_handle = tokio::spawn(async move {
		let mut ids = VecDeque::from(ids);
		loop {
			match ids.pop_front() {
				None => break, // we're done
				Some(id) => {
					if cache_2.get(&id).await.is_none() {
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
			},
			|items: Vec<CacheItem>| async { add_all(&cache, items).await },
			BatchSize::SmallInput,
		);
	});
	let cache = Arc::new(Cache::new(zbus_connection.clone()));
	group.bench_function(BenchmarkId::new("add_all", "wcag-docs"), |b| {
		b.to_async(&rt).iter_batched(
			|| {
				wcag_items
					.clone()
			},
			|items: Vec<CacheItem>| async { add_all(&cache, items).await },
			BatchSize::SmallInput,
		);
	});

	let cache = Arc::new(Cache::new(zbus_connection.clone()));
	group.bench_function(BenchmarkId::new("add", "zbus-docs"), |b| {
		b.to_async(&rt).iter_batched(
			|| {
				zbus_items
					.clone()
			},
			|items: Vec<CacheItem>| async { add(&cache, items).await },
			BatchSize::SmallInput,
		);
	});

	let (cache, children): (Arc<Cache>, Vec<Arc<Mutex<CacheItem>>>) = rt.block_on(async {
		let cache = Arc::new(Cache::new(zbus_connection.clone()));
		let all_items: Vec<CacheItem> = wcag_items
			.clone();
		let _ = cache.add_all(all_items).await;
		let mut children = Vec::new();
		for entry in cache.by_id.iter() {
			if entry.lock().await.children.is_empty() {
				children.push(Arc::clone(&entry));
			}
		}
		(cache, children)
	});
	group.bench_function(BenchmarkId::new("traverse_up_refs", "wcag-items"), |b| {
		b.to_async(&rt).iter_batched(
			|| children.clone(),
			|cs| async { traverse_up_refs(cs, &*cache).await },
			BatchSize::SmallInput,
		);
	});

	group.bench_function(BenchmarkId::new("traverse_up", "wcag-items"), |b| {
		b.to_async(&rt).iter_batched(
			|| children.clone(),
			|cs| async { traverse_up(cs, &*cache).await },
			BatchSize::SmallInput,
		);
	});

	group.bench_function(BenchmarkId::new("traverse_depth_first", "wcag-items"), |b| {
		b.iter_batched(
			|| {
				(block_on(cache.get(&AccessiblePrimitive {
					id: "/org/a11y/atspi/accessible/root".to_string(),
					sender: ":1.22".into(),
				}))
				.unwrap(), &*cache)
			},
			traverse_depth_first,
			BatchSize::SmallInput,
		);
	});

	let all_items = zbus_items.clone();
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
