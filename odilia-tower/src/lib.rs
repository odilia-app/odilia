#![deny(std_instead_of_alloc, alloc_instead_of_core)]
#![allow(clippy::cargo)]
#![feature(impl_trait_in_assoc_type)]

use odilia_common::errors::OdiliaError;

pub mod async_try;
pub mod cache_event;
pub mod choice;
pub mod from_state;
//pub mod handler;
pub mod iter_svc;
pub mod service_ext;
pub mod service_set;
pub mod state_changed;
pub mod state_svc;
pub mod sync_try;
pub mod unwrap_svc;
//pub use handler::Handler;
pub use service_ext::ServiceExt;

//pub mod handlers;
//pub use handlers::*;
