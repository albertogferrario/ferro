//! Built-in asset transforms.
//!
//! Each transform implements [`crate::Transform`] and is gated to its accepted
//! [`crate::ContentType`]s via [`crate::map_matching`]. Files outside a
//! transform's accepted set pass through byte-identical.
//!
//! ## Pipeline ordering (consumer reference)
//!
//! The canonical ordering gestiscilo's `PublishFrontendJob` uses:
//!
//! ```text
//! html_minify → css_minify → js_minify → image_transcode
//!     → responsive_images → inject_before_tag → replace_tokens
//! ```
//!
//! The crate does not enforce this order; the consumer adds transforms in sequence.
//!
//! ## Planned transforms (Plans 02-03)
//!
//! TODO(Plan 02): declare and pub-use html_minify, css_minify, js_minify
//! TODO(Plan 03): declare and pub-use image_transcode, responsive_images,
//!                inject_before_tag, replace_tokens

pub use crate::pipeline::Transform;
