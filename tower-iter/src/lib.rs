//! Extensions to [Tower](https://docs.rs/tower/latest/tower/) aiming at ease of use for sets and
//! iterators of [Service](https://docs.rs/tower/0.5.2/tower/trait.Service.html)s.
//!
//! This crate is `no_std` compatible, but requires `alloc`.

#![no_std]
#![deny(clippy::all, clippy::pedantic)]

extern crate alloc;

pub mod call_iter;
pub use call_iter::{MapM, MapMExt};
pub mod choice;
pub use choice::{ChoiceService, Chooser};
pub mod error;
pub use error::Error;
pub mod future_ext;
pub use future_ext::{FutureExt, MapOk};
pub mod iter_svc;
pub mod service_set;
pub use service_set::ServiceSet;
pub mod service_multi_iter;
pub mod service_multiset;
