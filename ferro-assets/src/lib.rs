//! # ferro-assets
//!
//! Composable, content-type-aware asset pipeline for the Ferro framework.
//!
//! ## Overview
//!
//! A [`Pipeline`] runs over a heterogeneous [`Asset`] set (HTML, CSS, JS, images,
//! and any other files). Each [`Transform`] declares which [`ContentType`]s it
//! accepts; files outside that set pass through byte-for-byte unchanged.
//!
//! ## Passthrough guarantee
//!
//! Any file whose extension is not recognized by a transform exits the pipeline
//! with bytes byte-identical to the input. This makes the pipeline safe for
//! heterogeneous artifact sets that include JSON-UI spec bundles, SSR manifests,
//! and static files alongside HTML/CSS/JS/images.
//!
//! ## Zero C system dependencies
//!
//! `ferro-assets` uses only pure-Rust codecs. `cargo build` introduces no new
//! system packages — no `libvips`, no `libavif`, no `libwebp`.
//!
//! ## Execution model
//!
//! `Pipeline::run()` is **synchronous (blocking)**. Wrap it in
//! `tokio::task::spawn_blocking` when calling from an async context so that
//! CPU-bound transform work does not stall the async executor.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use ferro_assets::{Pipeline, Asset};
//! use ferro_assets::transforms::{HtmlMinify, CssMinify};
//!
//! let pipeline = Pipeline::new()
//!     .add(HtmlMinify::new())
//!     .add(CssMinify::new());
//!
//! // Wrap in spawn_blocking — pipeline.run() is synchronous.
//! let result = tokio::task::spawn_blocking(move || pipeline.run(assets)).await??;
//! ```

mod asset;
mod error;
mod pipeline;
pub mod transforms;

pub use asset::{infer_content_type, Asset, ContentType};
pub use error::Error;
pub use pipeline::{map_matching, Pipeline, Transform};
