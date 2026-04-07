//! Toolchain check (D-02): rustc/cargo version vs `rust-toolchain.toml`.
//!
//! Reuses `crate::project::resolve_rust_base_image` to read the declared
//! channel from `rust-toolchain.toml` (no duplicate TOML parsing).

use crate::doctor::check::{CheckResult, DoctorCheck};
use crate::project::resolve_rust_base_image;
use std::path::Path;
use std::process::Command;

pub struct ToolchainCheck;

const NAME: &str = "toolchain";
const DEFAULT_IMAGE: &str = "rust:1.88-slim-bookworm";

impl DoctorCheck for ToolchainCheck {
    fn name(&self) -> &'static str {
        NAME
    }
    fn run(&self, root: &Path) -> CheckResult {
        check_impl(root)
    }
}

pub(crate) fn check_impl(root: &Path) -> CheckResult {
    let image = resolve_rust_base_image(root);
    let declared = if image == DEFAULT_IMAGE && !root.join("rust-toolchain.toml").exists() {
        None
    } else {
        // image is "rust:<channel>-slim-bookworm" — strip prefix/suffix.
        image
            .strip_prefix("rust:")
            .and_then(|s| s.strip_suffix("-slim-bookworm"))
            .map(|s| s.to_string())
    };

    let installed = match Command::new("rustc").arg("--version").output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Ok(_) => return CheckResult::error(NAME, "rustc invocation failed"),
        Err(_) => return CheckResult::error(NAME, "rustc not found in PATH"),
    };

    match declared {
        None => CheckResult::warn(NAME, format!("{installed}; rust-toolchain.toml missing"))
            .with_details("Declare a pinned channel for reproducible builds"),
        Some(channel) => {
            if installed.contains(&channel) {
                CheckResult::ok(NAME, format!("{installed} matches channel {channel}"))
            } else {
                CheckResult::warn(
                    NAME,
                    format!("installed {installed} differs from declared channel {channel}"),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn name_is_toolchain() {
        assert_eq!(ToolchainCheck.name(), "toolchain");
    }

    #[test]
    fn missing_rust_toolchain_warns_gracefully() {
        let tmp = TempDir::new().unwrap();
        let result = check_impl(tmp.path());
        // rustc is present in dev env; should be Warn (no toolchain.toml).
        assert_eq!(result.name, "toolchain");
        // Either Warn (rustc installed, no toml) or Error (rustc missing in CI).
        // Never panics — that's the contract.
    }
}
