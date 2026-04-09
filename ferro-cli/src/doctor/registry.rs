//! Registry of doctor checks.

use super::check::DoctorCheck;
use super::checks::{
    CopyDirsDockerignoreCollisionCheck, DatabaseUrlSqliteInProdCheck, DbConnectionCheck,
    DeployEnvParityCheck, DirtyGitTreeCheck, DockerTemplateDriftCheck, GeneratedArtifactsCheck,
    LocalEnvParityCheck, MigrationsCheck, ToolchainCheck,
};

/// Returns the canonical ordered list of checks:
/// toolchain_match → db_connection → migrations_pending → local_env_parity →
/// deploy_env_parity → copy_dirs_dockerignore_collision →
/// docker_template_drift → generated_artifacts → database_url_sqlite_in_prod →
/// git_clean_and_pushed.
pub fn default_checks() -> Vec<Box<dyn DoctorCheck>> {
    vec![
        Box::new(ToolchainCheck),
        Box::new(DbConnectionCheck),
        Box::new(MigrationsCheck),
        Box::new(LocalEnvParityCheck),
        Box::new(DeployEnvParityCheck),
        Box::new(CopyDirsDockerignoreCollisionCheck),
        Box::new(DockerTemplateDriftCheck),
        Box::new(GeneratedArtifactsCheck),
        Box::new(DatabaseUrlSqliteInProdCheck),
        Box::new(DirtyGitTreeCheck),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_checks_returns_ten_in_declared_order() {
        let checks = default_checks();
        assert_eq!(checks.len(), 10);
        let names: Vec<&'static str> = checks.iter().map(|c| c.name()).collect();
        assert_eq!(
            names,
            vec![
                "toolchain_match",
                "db_connection",
                "migrations_pending",
                "local_env_parity",
                "deploy_env_parity",
                "copy_dirs_dockerignore_collision",
                "docker_template_drift",
                "generated_artifacts",
                "database_url_sqlite_in_prod",
                "git_clean_and_pushed",
            ]
        );
    }

    #[test]
    fn deploy_category_filter_returns_two() {
        use crate::doctor::check::CheckCategory;
        let checks = default_checks();
        let deploy: Vec<&'static str> = checks
            .iter()
            .filter(|c| c.category() == CheckCategory::Deploy)
            .map(|c| c.name())
            .collect();
        assert_eq!(
            deploy,
            vec!["copy_dirs_dockerignore_collision", "docker_template_drift"]
        );
    }
}
