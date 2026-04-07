//! Registry of doctor checks. Order matches SCOPE §12.

use super::check::DoctorCheck;
use super::checks::{
    CargoDockerTomlStalenessCheck, DatabaseUrlSqliteInProdCheck, DbConnectionCheck,
    DeployEnvParityCheck, DirtyGitTreeCheck, GeneratedArtifactsCheck, LocalEnvParityCheck,
    MigrationsCheck, ToolchainCheck,
};

/// Returns the canonical ordered list of checks (SCOPE §12):
/// toolchain_match → db_connection → migrations_pending → local_env_parity →
/// deploy_env_parity → cargo_docker_toml_staleness → generated_artifacts →
/// database_url_sqlite_in_prod → git_clean_and_pushed.
pub fn default_checks() -> Vec<Box<dyn DoctorCheck>> {
    vec![
        Box::new(ToolchainCheck),
        Box::new(DbConnectionCheck),
        Box::new(MigrationsCheck),
        Box::new(LocalEnvParityCheck),
        Box::new(DeployEnvParityCheck),
        Box::new(CargoDockerTomlStalenessCheck),
        Box::new(GeneratedArtifactsCheck),
        Box::new(DatabaseUrlSqliteInProdCheck),
        Box::new(DirtyGitTreeCheck),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_checks_returns_nine_in_declared_order() {
        let checks = default_checks();
        assert_eq!(checks.len(), 9);
        let names: Vec<&'static str> = checks.iter().map(|c| c.name()).collect();
        assert_eq!(
            names,
            vec![
                "toolchain_match",
                "db_connection",
                "migrations_pending",
                "local_env_parity",
                "deploy_env_parity",
                "cargo_docker_toml_staleness",
                "generated_artifacts",
                "database_url_sqlite_in_prod",
                "git_clean_and_pushed",
            ]
        );
    }
}
