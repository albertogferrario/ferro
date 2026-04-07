//! Concrete doctor checks. Each submodule implements one `DoctorCheck`
//! using helpers from Phase 122/123 — no duplication.

pub mod artifacts;
pub mod db_connection;
pub mod env_completeness;
pub mod migrations;
pub mod path_deps;
pub mod toolchain;
pub mod workspace;

pub use artifacts::ArtifactsCheck;
pub use db_connection::DbConnectionCheck;
pub use env_completeness::EnvCompletenessCheck;
pub use migrations::MigrationsCheck;
pub use path_deps::PathDepsCheck;
pub use toolchain::ToolchainCheck;
pub use workspace::WorkspaceCheck;

use std::path::Path;
use std::process::{Command, Output};

/// Shared helper for checks that shell out to `cargo run -- <subcommand>`.
/// Mirrors `commands::db_status` exactly so doctor avoids pulling SeaORM
/// into its own compile graph.
#[allow(dead_code)] // Consumed by db_connection + migrations checks (Task 2).
pub(crate) fn run_cargo_subcommand(root: &Path, args: &[&str]) -> std::io::Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--quiet").arg("--").args(args);
    cmd.current_dir(root);
    cmd.output()
}
