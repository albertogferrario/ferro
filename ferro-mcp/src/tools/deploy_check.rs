//! `deploy_check` MCP tool (Phase 128 D-03) — shells out to
//! `ferro doctor --deploy --json` and returns the JSON Report verbatim.
//! Single-source-of-truth: the check registry lives in ferro-cli; this
//! tool never duplicates it.

use crate::error::{McpError, Result};
use std::path::Path;
use std::process::Command;

/// Args forwarded to `ferro doctor` when running the deploy-only check.
pub(crate) const DEPLOY_CHECK_ARGS: &[&str] = &["doctor", "--deploy", "--json"];

/// Run `ferro doctor --deploy --json` from `project_root` and return the
/// JSON Report on stdout.
///
/// The doctor binary exits 1 when any check returns `error`, but still writes
/// a valid JSON Report to stdout.  Only spawn / IO failures or a completely
/// empty stdout are treated as hard errors; a non-zero exit with JSON output
/// is returned as-is so callers can inspect the Report directly.
pub fn execute(project_root: &Path) -> Result<String> {
    let output = Command::new("ferro")
        .args(DEPLOY_CHECK_ARGS)
        .current_dir(project_root)
        .output()
        .map_err(|e| {
            McpError::ExecutionError(format!(
                "failed to spawn `ferro doctor --deploy --json`: {e}"
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Doctor exits 1 on check errors but still writes a valid JSON Report
    // on stdout. Only treat output-less failures as hard errors.
    if stdout.trim().is_empty() {
        return Err(McpError::ExecutionError(format!(
            "`ferro doctor --deploy --json` produced no output (exit={:?}, stderr={})",
            output.status.code(),
            stderr.trim()
        )));
    }

    // Sanity-parse to catch unparseable payloads early.
    if serde_json::from_str::<serde_json::Value>(&stdout).is_err() {
        return Err(McpError::ExecutionError(format!(
            "`ferro doctor --deploy --json` returned non-JSON output: {}",
            stdout.chars().take(200).collect::<String>()
        )));
    }

    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy_check_args_are_correct() {
        assert_eq!(DEPLOY_CHECK_ARGS, &["doctor", "--deploy", "--json"]);
    }

    #[test]
    fn deploy_check_args_length() {
        assert_eq!(DEPLOY_CHECK_ARGS.len(), 3);
    }
}
