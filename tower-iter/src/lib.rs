//! Extensions to [Tower](https://docs.rs/tower/latest/tower/) aiming at ease of use for sets and
//! iterators of [Service](https://docs.rs/tower/0.5.2/tower/trait.Service.html)s.
//!
//! This crate is not `no_std` compatible, as it requires `tower::util`.

#![deny(
	clippy::all,
	clippy::pedantic,
	unsafe_code,
	clippy::cargo,
	rustdoc::all,
	clippy::std_instead_of_core
)]

pub mod call_iter;
pub use call_iter::{MapM, MapMExt};
pub mod error;
pub use error::Error;
pub mod future_ext;
pub use future_ext::{FutureExt, MapOk};
pub mod iter_svc;
pub mod service_set;
pub use service_set::ServiceSet;
pub mod service_multi_iter;
pub mod service_multiset;
