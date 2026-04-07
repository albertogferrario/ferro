//! ferro-cli library surface.
//!
//! The CLI binary lives in `src/main.rs`. This library exists so integration
//! tests (and potentially other tooling) can call into the deploy-scaffold
//! helpers without going through the CLI shell.

pub mod ai;
pub mod analyzer;
pub mod commands;
pub mod deploy;
pub mod project;
pub mod templates;
