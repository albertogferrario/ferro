//! In-memory immutable byte blobs with content-hashed URLs and one-year immutable caching.
//!
//! See the crate README for the bundle-vs-filesystem split: ferro-bundle handles
//! compile-time-embedded immutable assets; the framework's filesystem static-file
//! handler at `ferro_rs::static_files` handles mutable on-disk tenant assets.
//!
//! The public API will be added in Plan 02 (`Bundle::new`, `.content_type`,
//! `.with_alias`, `.hashed_url`, `Bundle::serve`).
