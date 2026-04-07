//! Deploy scaffold primitives consumed by `docker:init`, `do:init`, and
//! `deploy:check` commands. Pure functions only — no filesystem side effects
//! beyond reading explicit input paths. See plan 122-02.

#![allow(dead_code, unused_imports)] // Consumed by plans 122-03..07.

pub mod classify;
pub mod env_example;
pub mod ferro_deps;

pub use classify::is_secret;
pub use env_example::{parse_env_example, EnvEntry};
pub use ferro_deps::render_rewrite_script;
