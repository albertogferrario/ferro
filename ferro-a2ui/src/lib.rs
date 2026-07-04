//! A2UI renderer for Ferro service projections.
//!
//! Projects a [`ferro_projections::ServiceDef`] and its derived intents into
//! [A2UI](https://a2ui.org/) surfaces — flat streaming component lists with
//! JSON Pointer data bindings. Targets the A2UI v1.0 release-candidate wire
//! format; the crate is experimental and unpublished until v1.0 stable.
//!
//! # Crate boundary
//!
//! This crate is the sole home of the A2UI `Renderer` implementation.
//! `ferro-projections` owns the trait and schema types; this crate owns the
//! wire types and emission logic. It has no dependency on `ferro-json-ui`.
