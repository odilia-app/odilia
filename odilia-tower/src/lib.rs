#![forbid(clippy::std_instead_of_alloc, clippy::alloc_instead_of_core, clippy::std_instead_of_core)]

extern crate alloc;

use odilia_common::errors::OdiliaError;

pub mod async_try;
pub mod error;
pub use error::Error;
pub mod from_state;
pub mod service_ext;
pub mod state_svc;
pub mod sync_try;
pub mod unwrap_svc;
pub use service_ext::ServiceExt;
