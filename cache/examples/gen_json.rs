////! This example demonstrates how to construct a tree of accessible objects on the accessibility bus.
////! Additionally, it places all the information into the [`odilia_cache::Cache`] struct and uses
////! [`serde`] to output it in its entirety (in JSON format).
////!
////! This is then used to run the benchmarks.
////!
////! ```sh
////! cargo run --example gen-json
////! ```
////! Authors:
////!    Luuk van der Duim,
////!    Tait Hoyem
//
//use std::{collections::VecDeque, sync::Arc};
//
//use atspi::{
//	connection::set_session_accessibility,
//	proxy::accessible::{AccessibleProxy, ObjectRefExt},
//	AccessibilityConnection,
//};
//use futures_util::future::try_join_all;
//use odilia_cache::{accessible_to_cache_item, Cache, CacheItem};
//use odilia_common::errors::{CacheError, OdiliaError};
//use serde_json::to_string;
//use zbus::{proxy::CacheProperties, Connection};
//
//type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
//
//const REGISTRY_DEST: &str = "org.a11y.atspi.Registry";
//const REGISTRY_PATH: &str = "/org/a11y/atspi/accessible/root";
//const ACCESSIBLE_INTERFACE: &str = "org.a11y.atspi.Accessible";
//const MAX_CHILDREN: i32 = 65536;
//
//async fn from_a11y_proxy(ap: AccessibleProxy<'_>) -> Result<Arc<Cache<Connection>>> {
//	let connection = ap.inner().connection().clone();
//	// Contains the processed `A11yNode`'s.
//	let cache = Arc::new(Cache::new(connection.clone()));
//
//	// Contains the `AccessibleProxy` yet to be processed.
//	let mut stack: VecDeque<AccessibleProxy> = vec![ap].into();
//
//	// If the stack has an `AccessibleProxy`, we take the last.
//	while let Some(ap) = stack.pop_front() {
//		let cache_item = accessible_to_cache_item(&ap).await?;
//		match cache.add(cache_item) {
//			Ok(_) | Err(OdiliaError::Cache(CacheError::MoreData(_))) => {}
//			Err(e) => return Err(Box::new(e)),
//		};
//		// Prevent objects with a huge child count from stalling the program.
//		if ap.child_count().await? > MAX_CHILDREN {
//			continue;
//		}
//
//		let child_objects = ap.get_children().await?;
//		let mut children_proxies = try_join_all(
//			child_objects
//				.into_iter()
//				.map(|child| child.into_accessible_proxy(&connection)),
//		)
//		.await?
//		.into();
//		stack.append(&mut children_proxies);
//	}
//	Ok(cache)
//}
//
//async fn get_registry_accessible<'a>(conn: &Connection) -> Result<AccessibleProxy<'a>> {
//	let registry = AccessibleProxy::builder(conn)
//		.destination(REGISTRY_DEST)?
//		.path(REGISTRY_PATH)?
//		.interface(ACCESSIBLE_INTERFACE)?
//		.cache_properties(CacheProperties::No)
//		.build()
//		.await?;
//
//	Ok(registry)
//}
//
//#[tokio::main(flavor = "current_thread")]
//async fn main() -> Result<()> {
//	set_session_accessibility(true).await?;
//	let a11y = AccessibilityConnection::new().await?;
//
//	let conn = a11y.connection();
//	let registry = get_registry_accessible(conn).await?;
//
//	let tree = from_a11y_proxy(registry).await?;
//
//	let read_cache = tree.tree.read();
//	// this makes sure that all parent nodes are listed first in the list
//	let output = read_cache
//		.iter()
//		// first, find all parents
//		.filter_map(|node| {
//			if node.parent().is_none() {
//				read_cache.get_node_id(node)
//			} else {
//				None
//			}
//		})
//		// then call descendants (which will include the parent itself) and flatten it
//		.flat_map(|parent| parent.descendants(&read_cache))
//		// now convert to CacheItem
//		.map(|node_id| read_cache.get(node_id).expect("Valid Node ID").get().to_owned())
//		.collect::<Vec<CacheItem>>();
//	println!("{}", to_string(&output).expect("successful serialization"));
//
//	Ok(())
//}
fn main() {}
