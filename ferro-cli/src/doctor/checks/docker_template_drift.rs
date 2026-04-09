//! Doctor check: flag when the committed `Dockerfile` has drifted from what
//! the current scaffolder would generate.
//!
//! Severity is `Warn`, not `Error` — hand-editing the Dockerfile is
//! legitimate. The check exists to inform, not to block. See Phase 131
//! research open question 4 for rationale.

use crate::deploy::bin_detect::detect_web_bin;
use crate::doctor::check::{CheckCategory, CheckResult, DoctorCheck};
use crate::project::read_deploy_metadata;
use crate::templates::docker::{read_bins, read_rust_channel, render_dockerfile, DockerContext};
use std::fs;
use std::path::Path;

const NAME: &str = "docker_template_drift";

pub struct DockerTemplateDriftCheck;

impl DoctorCheck for DockerTemplateDriftCheck {
    fn name(&self) -> &'static str {
        NAME
    }

    fn category(&self) -> CheckCategory {
        CheckCategory::Deploy
    }

    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let dockerfile = root.join("Dockerfile");

    if !dockerfile.is_file() {
        return CheckResult::ok(NAME, "skipped (Dockerfile absent)");
    }

    let committed = match fs::read_to_string(&dockerfile) {
        Ok(s) => s,
        Err(e) => return CheckResult::error(NAME, format!("read failed: {e}")),
    };

    let metadata = match read_deploy_metadata(root) {
        Ok(m) => m,
        Err(e) => return CheckResult::error(NAME, format!("metadata: {e}")),
    };

    let bins = match read_bins(root) {
        Ok(b) => b,
        Err(e) => return CheckResult::error(NAME, format!("read_bins: {e}")),
    };

    let web_bin = match detect_web_bin(root) {
        Ok(w) => w,
        Err(e) => return CheckResult::error(NAME, format!("web_bin: {e}")),
    };

    let copy_dirs_present: Vec<String> = metadata
        .copy_dirs
        .iter()
        .filter(|d| root.join(d.as_str()).exists())
        .cloned()
        .collect();

    let ctx = DockerContext {
        rust_channel: read_rust_channel(root),
        has_frontend: root.join("frontend/package.json").is_file(),
        bins,
        web_bin,
        copy_dirs_present,
        runtime_apt: metadata.runtime_apt,
    };

    let rendered = render_dockerfile(&ctx);

    if rendered.trim_end() == committed.trim_end() {
        CheckResult::ok(NAME, "Dockerfile matches scaffolder output")
    } else {
        CheckResult::warn(NAME, "Dockerfile has drifted from scaffolder")
            .with_details("run `ferro docker:init --dry-run` to inspect the delta")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::check::CheckStatus;
    use std::fs;
    use tempfile::TempDir;

    fn write(p: &Path, body: &str) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, body).unwrap();
    }

    fn minimal_cargo_toml(name: &str) -> String {
        format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"{name}\"\npath = \"src/main.rs\"\n"
        )
    }

    #[test]
    fn name_and_category() {
        assert_eq!(DockerTemplateDriftCheck.name(), NAME);
        assert_eq!(DockerTemplateDriftCheck.category(), CheckCategory::Deploy);
    }

    #[test]
    fn docker_template_drift_ok_when_dockerfile_absent() {
        let td = TempDir::new().unwrap();
        write(&td.path().join("Cargo.toml"), &minimal_cargo_toml("sample"));
        let r = check_impl(td.path());
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("skipped"));
    }

    #[test]
    fn docker_template_drift_ok_on_matching_fixture() {
        use crate::templates::docker::DockerContext;

        // Build a minimal project that matches the freshly-rendered Dockerfile.
        let td = TempDir::new().unwrap();
        write(
            &td.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"sample\"\npath = \"src/main.rs\"\n",
        );

        // Render a Dockerfile and write it to the tempdir so the check can
        // verify it matches.
        let ctx = DockerContext {
            rust_channel: "stable".to_string(),
            has_frontend: false,
            bins: vec!["sample".to_string()],
            web_bin: "sample".to_string(),
            copy_dirs_present: vec![],
            runtime_apt: vec![],
        };
        let rendered = render_dockerfile(&ctx);
        write(&td.path().join("Dockerfile"), &rendered);

        let r = check_impl(td.path());
        assert_eq!(
            r.status,
            CheckStatus::Ok,
            "matching Dockerfile must not warn: {}",
            r.message
        );
        assert!(r.message.contains("matches"));
    }

    #[test]
    fn docker_template_drift_warn_on_mutation() {
        use crate::templates::docker::DockerContext;

        let td = TempDir::new().unwrap();
        write(
            &td.path().join("Cargo.toml"),
            "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n\n[[bin]]\nname = \"sample\"\npath = \"src/main.rs\"\n",
        );

        // Render the correct Dockerfile, then append a spurious comment.
        let ctx = DockerContext {
            rust_channel: "stable".to_string(),
            has_frontend: false,
            bins: vec!["sample".to_string()],
            web_bin: "sample".to_string(),
            copy_dirs_present: vec![],
            runtime_apt: vec![],
        };
        let mut rendered = render_dockerfile(&ctx);
        rendered.push_str("# spurious comment added by hand\n");
        write(&td.path().join("Dockerfile"), &rendered);

        let r = check_impl(td.path());
        assert_eq!(
            r.status,
            CheckStatus::Warn,
            "mutated Dockerfile must warn: {}",
            r.message
        );
        assert!(r.message.contains("drifted"));
        assert!(r.details.as_ref().unwrap().contains("docker:init"));
    }
}
