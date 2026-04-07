//! Concrete doctor checks. SCOPE §12 — 9 revised checks.

pub mod cargo_docker_toml_staleness;
pub mod database_url_sqlite_in_prod;
pub mod db_connection;
pub mod deploy_env_parity;
pub mod dirty_git_tree;
pub mod generated_artifacts;
pub mod local_env_parity;
pub mod migrations;
pub mod toolchain;

pub use cargo_docker_toml_staleness::CargoDockerTomlStalenessCheck;
pub use database_url_sqlite_in_prod::DatabaseUrlSqliteInProdCheck;
pub use db_connection::DbConnectionCheck;
pub use deploy_env_parity::DeployEnvParityCheck;
pub use dirty_git_tree::DirtyGitTreeCheck;
pub use generated_artifacts::GeneratedArtifactsCheck;
pub use local_env_parity::LocalEnvParityCheck;
pub use migrations::MigrationsCheck;
pub use toolchain::ToolchainCheck;

use std::path::Path;
use std::process::{Command, Output};

/// Shared helper for checks that shell out to `cargo run -- <subcommand>`.
pub(crate) fn run_cargo_subcommand(root: &Path, args: &[&str]) -> std::io::Result<Output> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--quiet").arg("--").args(args);
    cmd.current_dir(root);
    cmd.output()
}
