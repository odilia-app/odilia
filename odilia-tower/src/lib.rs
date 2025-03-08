//! Integrations for the Odilia screen reader and the [`tower`] ecosystem.
//! While most Tower crates focus on networking applications, we beleive that it has great
//! usefulness to us as assistive technology makers.
//!
//! Most of these are generic-ish utilities we've needed in Odilia.
//! We are open to removing them in the case that general-purpose ones are avaialbe and easy to
//! integrate.

#![forbid(
	clippy::std_instead_of_core,
	clippy::alloc_instead_of_core,
	clippy::std_instead_of_alloc,
	clippy::all,
	clippy::pedantic,
	clippy::cargo,
	unsafe_code,
	missing_docs
)]

use odilia_common::errors::OdiliaError;

pub mod async_try;
pub mod service_ext;
pub mod state_svc;
pub mod sync_try;
pub mod unwrap_svc;
pub use service_ext::ServiceExt;
