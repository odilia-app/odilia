//! This module describes specific values of various attributes seen either on the web or through GTK/Qt.
//! Generally, if possible, Odilia prefers to deal with explicit types and variants, not strings.
//! Instead of reading from a series of attributes, like a `HashMap<String, String>`, it should use types from this module.
//! Note that this module does not yet implement an "Attributes type", but this will be coming.

use atspi_common::Granularity;
use serde::{self, Deserialize, Serialize};

/// Defines possible values of an [aria-live](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-live) field.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase", untagged)]
pub enum AriaLive {
	/// No ARIA live settings.
	Off,
	/// The ARIA region should be spoken as more important than other text.
	/// This usually means that other text gets interrupted, but not nesessarily.
	Assertive,
	/// The ARIA region should be spoken whenever it can, without priority.
	/// This usually means that the text gets spoken once other text is done being said.
	Polite,
	/// Any other possible value.
	Other(String),
}

/// Defines whether an item is defined as an [aria-atomic](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-atomic) region.
pub type AriaAtomic = bool;
