//! Shared web-bin detection for `docker:init` and `do:init` (D-02).
//!
//! Resolves the "web bin" — the binary that should be the container
//! ENTRYPOINT and the DigitalOcean `web` service target — using a stable
//! 4-step order so both scaffolders stay in sync by construction.

use std::path::Path;

use crate::project::{package_name, read_deploy_metadata};
use crate::templates::docker::read_bins;

/// Resolve the web bin name using the D-02 4-step order:
/// 1. `[package.metadata.ferro.deploy].web_bin` explicit override
/// 2. The bin matching `package.name`
/// 3. The first declared `[[bin]]`
/// 4. Fall back to `package.name` if no `[[bin]]` is declared
pub fn detect_web_bin(project_root: &Path) -> anyhow::Result<String> {
    if let Ok(meta) = read_deploy_metadata(project_root) {
        if let Some(explicit) = meta.web_bin {
            return Ok(explicit);
        }
    }
    let pkg = package_name(project_root);
    let bins = read_bins(project_root)?;
    if bins.iter().any(|b| b == &pkg) {
        return Ok(pkg);
    }
    if let Some(first) = bins.first() {
        return Ok(first.clone());
    }
    Ok(pkg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_cargo(body: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), body).unwrap();
        tmp
    }

    #[test]
    fn bin_detect_explicit_override() {
        let tmp = write_cargo(
            r#"
[package]
name = "myapp"
version = "0.1.0"

[[bin]]
name = "alpha"

[[bin]]
name = "beta"

[package.metadata.ferro.deploy]
web_bin = "foo"
"#,
        );
        assert_eq!(detect_web_bin(tmp.path()).unwrap(), "foo");
    }

    #[test]
    fn bin_detect_package_name_match() {
        let tmp = write_cargo(
            r#"
[package]
name = "api"
version = "0.1.0"

[[bin]]
name = "api"

[[bin]]
name = "worker"
"#,
        );
        assert_eq!(detect_web_bin(tmp.path()).unwrap(), "api");
    }

    #[test]
    fn bin_detect_first_bin_fallback() {
        let tmp = write_cargo(
            r#"
[package]
name = "other"
version = "0.1.0"

[[bin]]
name = "alpha"

[[bin]]
name = "beta"
"#,
        );
        assert_eq!(detect_web_bin(tmp.path()).unwrap(), "alpha");
    }

    #[test]
    fn bin_detect_no_bins_uses_package_name() {
        let tmp = write_cargo(
            r#"
[package]
name = "myapp"
version = "0.1.0"
"#,
        );
        assert_eq!(detect_web_bin(tmp.path()).unwrap(), "myapp");
    }
}
