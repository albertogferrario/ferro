//! Registry of doctor checks. Order matches D-01.

use super::check::DoctorCheck;
use super::checks::{
    ArtifactsCheck, DbConnectionCheck, EnvCompletenessCheck, MigrationsCheck, PathDepsCheck,
    ToolchainCheck, WorkspaceCheck,
};

/// Returns the canonical ordered list of checks (D-01):
/// toolchain → db_connection → migrations → env_completeness → path_deps
/// → workspace → artifacts.
pub fn default_checks() -> Vec<Box<dyn DoctorCheck>> {
    vec![
        Box::new(ToolchainCheck),
        Box::new(DbConnectionCheck),
        Box::new(MigrationsCheck),
        Box::new(EnvCompletenessCheck),
        Box::new(PathDepsCheck),
        Box::new(WorkspaceCheck),
        Box::new(ArtifactsCheck),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_checks_returns_seven_in_declared_order() {
        let checks = default_checks();
        assert_eq!(checks.len(), 7);
        let names: Vec<&'static str> = checks.iter().map(|c| c.name()).collect();
        assert_eq!(
            names,
            vec![
                "toolchain",
                "db_connection",
                "migrations",
                "env_completeness",
                "path_deps",
                "workspace",
                "artifacts",
            ]
        );
    }
}
