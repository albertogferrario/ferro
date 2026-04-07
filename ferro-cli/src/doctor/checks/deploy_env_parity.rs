//! Deploy env parity (SCOPE §12.5): every key in `.env.production` appears
//! as a commented entry in `.do/app.yaml`'s envs scaffold.

use crate::deploy::env_production::read_env_production_keys;
use crate::doctor::check::{CheckResult, DoctorCheck};
use std::fs;
use std::path::Path;

pub struct DeployEnvParityCheck;

const NAME: &str = "deploy_env_parity";

impl DoctorCheck for DeployEnvParityCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let env_prod = root.join(".env.production");
    let app_yaml = root.join(".do/app.yaml");

    if !env_prod.is_file() || !app_yaml.is_file() {
        return CheckResult::ok(NAME, "skipped (.env.production or .do/app.yaml missing)");
    }

    let keys = match read_env_production_keys(&env_prod) {
        Ok(k) => k,
        Err(e) => return CheckResult::error(NAME, format!("failed to read .env.production: {e}")),
    };

    let yaml = fs::read_to_string(&app_yaml).unwrap_or_default();
    let missing = missing_keys(&keys, &yaml);

    if missing.is_empty() {
        CheckResult::ok(NAME, format!("{} keys scaffolded", keys.len()))
    } else {
        CheckResult::warn(
            NAME,
            format!("{} key(s) missing from .do/app.yaml", missing.len()),
        )
        .with_details(format!("missing: {}", missing.join(", ")))
    }
}

/// Return keys from `keys` that do not appear as a commented `# - KEY` line
/// (or any line containing the bare token) in the yaml body.
fn missing_keys(keys: &[String], yaml: &str) -> Vec<String> {
    keys.iter()
        .filter(|k| !yaml_contains_key(yaml, k))
        .cloned()
        .collect()
}

fn yaml_contains_key(yaml: &str, key: &str) -> bool {
    for line in yaml.lines() {
        let trimmed = line.trim_start();
        // Look only inside comment scaffolding lines.
        let body = match trimmed.strip_prefix('#') {
            Some(rest) => rest.trim_start(),
            None => continue,
        };
        // Match `- KEY` or just KEY token; require word-boundary equality.
        let token = body.trim_start_matches('-').trim();
        if token == key {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn name_is_deploy_env_parity() {
        assert_eq!(DeployEnvParityCheck.name(), "deploy_env_parity");
    }

    #[test]
    fn skips_when_files_absent() {
        let tmp = TempDir::new().unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn ok_when_all_keys_present() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".env.production"),
            "APP_ENV=x\nDATABASE_URL=y\n",
        )
        .unwrap();
        fs::create_dir(tmp.path().join(".do")).unwrap();
        fs::write(
            tmp.path().join(".do/app.yaml"),
            "envs:\n  # - APP_ENV\n  # - DATABASE_URL\n",
        )
        .unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Ok);
    }

    #[test]
    fn warn_on_missing_key() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".env.production"),
            "APP_ENV=x\nDATABASE_URL=y\n",
        )
        .unwrap();
        fs::create_dir(tmp.path().join(".do")).unwrap();
        fs::write(tmp.path().join(".do/app.yaml"), "envs:\n  # - APP_ENV\n").unwrap();
        let r = check_impl(tmp.path());
        assert_eq!(r.status, crate::doctor::check::CheckStatus::Warn);
        assert!(r.details.unwrap().contains("DATABASE_URL"));
    }
}
