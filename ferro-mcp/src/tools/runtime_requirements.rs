//! runtime_requirements MCP tool — maps Cargo crate deps to apt runtime
//! packages via the ferro-cli RUNTIME_DEP_REGISTRY (D-07..D-10). Read-only
//! (D-11). Cross-checks the project Dockerfile and flags apt packages that
//! are required by the crate registry but missing from the Dockerfile's
//! `apt-get install` lines.

use crate::error::{McpError, Result};
use crate::tools::deploy_common::{scan_runtime_dep_matches, RuntimeDep};
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct RequiredPackage {
    pub crate_name: String,
    pub apt_packages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeRequirementsReport {
    pub required: Vec<RequiredPackage>,
    pub dockerfile_present: bool,
    pub installed_in_dockerfile: Vec<String>,
    pub missing_in_dockerfile: Vec<String>,
}

/// Parse apt-get install package lists out of a Dockerfile.
///
/// Strategy:
/// 1. Collapse backslash line continuations (`\\\n`) into spaces so multi-line
///    `RUN apt-get install -y \\\n    chromium \\\n    fonts-liberation` becomes
///    a single line.
/// 2. Match each `apt-get install ...` occurrence with a regex.
/// 3. From each match, split on whitespace and keep tokens that look like
///    package names (not flags, not shell glue, not apt-get keywords).
pub fn parse_dockerfile_apt_packages(content: &str) -> Vec<String> {
    // Step 1: collapse `\<newline>` continuations.
    let collapsed = content.replace("\\\n", " ");

    // Step 2: find `apt-get install ...` segments. Stop at `&&`, `;`, or EOL.
    let re = Regex::new(r"(?m)apt-get\s+install([^\n&;]*)").expect("valid regex");

    const SKIP: &[&str] = &[
        "apt-get",
        "install",
        "update",
        "-y",
        "-q",
        "-qq",
        "--yes",
        "--no-install-recommends",
        "--no-install-suggests",
        "&&",
        "\\",
        "",
    ];

    let mut set: BTreeSet<String> = BTreeSet::new();
    for caps in re.captures_iter(&collapsed) {
        let Some(tail) = caps.get(1) else { continue };
        for tok in tail.as_str().split_whitespace() {
            if tok.starts_with('-') {
                continue;
            }
            if SKIP.contains(&tok) {
                continue;
            }
            // Package names: lowercase alnum + `.+-_`. Reject anything else
            // (shell glue like `rm`, `/var/lib/...`).
            if !tok
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | '_'))
            {
                continue;
            }
            set.insert(tok.to_string());
        }
    }
    set.into_iter().collect()
}

pub fn execute(project_root: &Path) -> Result<RuntimeRequirementsReport> {
    let cargo_path = project_root.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_path).map_err(|_| {
        McpError::ToolError("runtime_requirements: Cargo.toml not found".to_string())
    })?;

    let matches: Vec<&'static RuntimeDep> = scan_runtime_dep_matches(&cargo_content);

    let required: Vec<RequiredPackage> = matches
        .iter()
        .map(|m| RequiredPackage {
            crate_name: m.crate_name.to_string(),
            apt_packages: m.apt_packages.iter().map(|p| (*p).to_string()).collect(),
        })
        .collect();

    let required_set: BTreeSet<String> = matches
        .iter()
        .flat_map(|m| m.apt_packages.iter().map(|p| (*p).to_string()))
        .collect();

    let dockerfile_path = project_root.join("Dockerfile");
    let (dockerfile_present, installed_in_dockerfile, missing_in_dockerfile) =
        if dockerfile_path.exists() {
            let content = fs::read_to_string(&dockerfile_path).unwrap_or_default();
            let installed = parse_dockerfile_apt_packages(&content);
            let installed_set: BTreeSet<String> = installed.iter().cloned().collect();
            let missing: Vec<String> = required_set.difference(&installed_set).cloned().collect();
            (true, installed, missing)
        } else {
            (false, Vec::new(), Vec::new())
        };

    Ok(RuntimeRequirementsReport {
        required,
        dockerfile_present,
        installed_in_dockerfile,
        missing_in_dockerfile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(td: &TempDir, name: &str, content: &str) {
        fs::write(td.path().join(name), content).unwrap();
    }

    #[test]
    fn fixture_a_gestiscilo_chromium_missing() {
        let td = TempDir::new().unwrap();
        write(
            &td,
            "Cargo.toml",
            r#"
[package]
name = "x"
version = "0.1.0"

[dependencies]
chromiumoxide = "0.5"
"#,
        );
        write(
            &td,
            "Dockerfile",
            "FROM debian:bookworm\nRUN apt-get update && apt-get install -y ca-certificates\n",
        );
        let r = execute(td.path()).unwrap();
        assert!(r.dockerfile_present);
        assert_eq!(r.required.len(), 1);
        assert_eq!(r.required[0].crate_name, "chromiumoxide");
        assert_eq!(
            r.required[0].apt_packages,
            vec!["chromium".to_string(), "fonts-liberation".to_string()]
        );
        assert_eq!(
            r.installed_in_dockerfile,
            vec!["ca-certificates".to_string()]
        );
        assert_eq!(
            r.missing_in_dockerfile,
            vec!["chromium".to_string(), "fonts-liberation".to_string()]
        );
    }

    #[test]
    fn fixture_b_mkmenu_clean() {
        let td = TempDir::new().unwrap();
        write(
            &td,
            "Cargo.toml",
            r#"
[package]
name = "x"
version = "0.1.0"

[dependencies]
serde = "1"
tokio = { version = "1", features = ["full"] }
"#,
        );
        write(
            &td,
            "Dockerfile",
            "FROM debian:bookworm\nRUN apt-get update && apt-get install -y ca-certificates\n",
        );
        let r = execute(td.path()).unwrap();
        assert!(r.required.is_empty());
        assert!(r.missing_in_dockerfile.is_empty());
        assert!(r.dockerfile_present);
    }

    #[test]
    fn fixture_c_ffmpeg_satisfied() {
        let td = TempDir::new().unwrap();
        write(
            &td,
            "Cargo.toml",
            r#"
[package]
name = "x"
version = "0.1.0"

[dependencies]
ffmpeg-next = "6"
"#,
        );
        write(
            &td,
            "Dockerfile",
            "FROM debian:bookworm\nRUN apt-get update && apt-get install -y ffmpeg ca-certificates\n",
        );
        let r = execute(td.path()).unwrap();
        assert_eq!(r.required.len(), 1);
        assert_eq!(r.required[0].crate_name, "ffmpeg-next");
        assert!(r.installed_in_dockerfile.contains(&"ffmpeg".to_string()));
        assert!(r.missing_in_dockerfile.is_empty());
    }

    #[test]
    fn fixture_d_no_dockerfile() {
        let td = TempDir::new().unwrap();
        write(
            &td,
            "Cargo.toml",
            r#"
[package]
name = "x"
version = "0.1.0"

[dependencies]
chromiumoxide = "0.5"
"#,
        );
        let r = execute(td.path()).unwrap();
        assert!(!r.dockerfile_present);
        assert_eq!(r.required.len(), 1);
        assert!(r.installed_in_dockerfile.is_empty());
        assert!(r.missing_in_dockerfile.is_empty());
    }

    #[test]
    fn fixture_e_missing_cargo_toml_errors() {
        let td = TempDir::new().unwrap();
        let err = execute(td.path());
        assert!(err.is_err());
    }

    #[test]
    fn multi_line_apt_install_is_parsed() {
        let dockerfile = "\
FROM debian:bookworm
RUN apt-get update && apt-get install -y \\
    chromium \\
    fonts-liberation \\
    ca-certificates
";
        let pkgs = parse_dockerfile_apt_packages(dockerfile);
        assert!(pkgs.contains(&"chromium".to_string()));
        assert!(pkgs.contains(&"fonts-liberation".to_string()));
        assert!(pkgs.contains(&"ca-certificates".to_string()));
    }
}
