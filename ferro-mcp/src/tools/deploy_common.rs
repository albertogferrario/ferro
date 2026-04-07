//! Shared imports used by deploy_* MCP tools (Plans 03/04/05).
//! This module exists to prove the ferro-cli dependency resolves and
//! to provide a single place for re-exports ferro-mcp tools consume.
#![allow(unused_imports)]

pub use ferro_cli::commands::deploy_check::check_ref;
pub use ferro_cli::deploy::{
    find_ferro_path_deps, is_secret, parse_env_example, scan_runtime_dep_matches,
    scan_runtime_deps_str, EnvEntry, RuntimeDep, RUNTIME_DEP_REGISTRY,
};
pub use ferro_cli::project::find_project_root;
