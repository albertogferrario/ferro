//! design:lint command — walk JSON-UI spec files and report design-pattern findings.
//!
//! Discovers `*.json` files under a directory (default `src/views`), skips files
//! without the `ferro-json-ui/v2` schema marker, and runs `ferro_json_ui::design::lint`
//! on every matching spec. Findings are grouped by file in human-readable mode or
//! emitted as a flat JSON array via `--json`.
//!
//! Exit code is 0 unless `--deny` is set and at least one warning-level finding exists.
//! Info findings never cause a non-zero exit.

use console::style;
use ferro_json_ui::design::{lint, Finding, Severity};
use ferro_json_ui::spec::{Spec, SCHEMA_VERSION};
use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

/// One finding tagged with the file it came from.
///
/// This flat shape is the stable `--json` contract consumed by downstream CI
/// (gestiscilo Phase 232). The `file` field identifies the source file;
/// all other fields come directly from [`Finding`] via `#[serde(flatten)]`.
#[derive(Serialize)]
pub struct FileFinding {
    /// Path to the source file, as discovered by the walker.
    pub file: String,
    /// The finding from the design lint engine.
    #[serde(flatten)]
    pub finding: Finding,
}

/// Lint the JSON content of a single named file.
///
/// Returns an empty vec if the content does not contain the `ferro-json-ui/v2`
/// schema marker (silently skipped — non-ferro JSON files are ignored).
///
/// Returns one `spec-parse` warning-level finding if the marker is present but
/// `Spec::from_json` fails (malformed spec flagged without panicking).
///
/// Otherwise returns the findings produced by `ferro_json_ui::design::lint`.
pub(crate) fn lint_content(file: &str, content: &str) -> Vec<FileFinding> {
    if !content.contains(SCHEMA_VERSION) {
        return Vec::new();
    }
    match Spec::from_json(content) {
        Ok(spec) => lint(&spec)
            .into_iter()
            .map(|finding| FileFinding {
                file: file.to_string(),
                finding,
            })
            .collect(),
        Err(e) => vec![FileFinding {
            file: file.to_string(),
            finding: Finding {
                rule: "spec-parse",
                element_id: None,
                severity: Severity::Warning,
                message: format!("Failed to parse spec: {e:?}"),
                suggestion: "Fix the spec so it parses as ferro-json-ui/v2.".into(),
            },
        }],
    }
}

/// Returns `true` if any finding in the slice has warning-level severity.
///
/// Used to drive the `--deny` CI gate: info findings never fail.
pub(crate) fn has_warning(findings: &[FileFinding]) -> bool {
    findings
        .iter()
        .any(|f| matches!(f.finding.severity, Severity::Warning))
}

/// Main entry point for the `design:lint` command.
///
/// Walks `*.json` files under `path` (default `src/views`) without following
/// symlinks (confining the walk to the given root), lints each ferro-json-ui
/// spec, and prints findings grouped by file in human-readable mode.
///
/// `--json` emits a flat JSON array of [`FileFinding`] suitable for programmatic
/// consumption. `--deny` causes a non-zero exit when any warning-level finding
/// exists (info findings never fail).
pub fn run(path: Option<String>, json: bool, deny: bool) {
    let root = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("src/views"));

    let mut all: Vec<FileFinding> = Vec::new();

    // WalkDir default: follow_links = false (symlinks not traversed — T-252-01).
    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
    {
        let file_path = entry.path();
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let label = file_path.display().to_string();
        all.extend(lint_content(&label, &content));
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&all).unwrap_or_else(|_| "[]".into())
        );
    } else {
        print_human(&all);
    }

    if deny && has_warning(&all) {
        std::process::exit(1);
    }
}

/// Print findings in human-readable form, grouped by file.
fn print_human(all: &[FileFinding]) {
    // Collect files in encounter order (preserve walker order).
    let mut files_seen: Vec<&str> = Vec::new();
    for ff in all {
        let f = ff.file.as_str();
        if !files_seen.contains(&f) {
            files_seen.push(f);
        }
    }

    if files_seen.is_empty() {
        println!(
            "{}",
            style("No findings — all specs are clean.").green().bold()
        );
        return;
    }

    for file in &files_seen {
        println!("\n{}", style(file).bold().underlined());
        for ff in all.iter().filter(|ff| ff.file.as_str() == *file) {
            let sev_label = match ff.finding.severity {
                Severity::Warning => style("warning").yellow().bold(),
                Severity::Info => style("info").cyan(),
            };
            println!(
                "  {} [{}] {}",
                sev_label,
                style(ff.finding.rule).dim(),
                ff.finding.message
            );
            println!(
                "    {} {}",
                style("→").dim(),
                style(&ff.finding.suggestion).dim()
            );
        }
    }

    let warn_count = all
        .iter()
        .filter(|ff| matches!(ff.finding.severity, Severity::Warning))
        .count();
    let info_count = all
        .iter()
        .filter(|ff| matches!(ff.finding.severity, Severity::Info))
        .count();
    println!(
        "\n{} {} warning(s), {} info finding(s) across {} file(s)",
        style("Summary:").bold(),
        warn_count,
        info_count,
        files_seen.len()
    );
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal conforming spec: auth layout + focus intent + Text element.
    ///
    /// Auth layout is exempt from page-header and breadcrumb-on-subpages.
    /// Focus intent is valid. No composition violations expected.
    const CLEAN: &str = r#"{"$schema":"ferro-json-ui/v2","root":"t","layout":"auth","design":{"intent":"focus"},"elements":{"t":{"type":"Text","props":{"content":"hi"}}}}"#;

    /// Same structure as CLEAN but with an unknown intent — produces 1 Warning.
    const WARN: &str = r#"{"$schema":"ferro-json-ui/v2","root":"t","layout":"auth","design":{"intent":"totally-made-up"},"elements":{"t":{"type":"Text","props":{"content":"hi"}}}}"#;

    #[test]
    fn lint_content_clean_zero_findings() {
        let findings = lint_content("clean.json", CLEAN);
        assert!(
            findings.is_empty(),
            "clean spec should produce no findings, got: {:#?}",
            findings
                .iter()
                .map(|f| (f.finding.rule, &f.finding.message))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn lint_content_warn_one_warning_finding() {
        let findings = lint_content("warn.json", WARN);
        assert_eq!(
            findings.len(),
            1,
            "expected exactly 1 finding, got: {:#?}",
            findings
                .iter()
                .map(|f| (f.finding.rule, &f.finding.message))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            findings[0].finding.severity,
            Severity::Warning,
            "finding must be warning-level"
        );
        assert_eq!(findings[0].file, "warn.json", "file field must match");
    }

    #[test]
    fn lint_content_skip_non_marker_file() {
        let findings = lint_content("skip.json", r#"{"hello":1}"#);
        assert!(
            findings.is_empty(),
            "non-marker file must be silently skipped"
        );
    }

    #[test]
    fn lint_content_bad_parse_emits_spec_parse_warning() {
        // Marker present but root "missing" does not exist in the elements map.
        let findings = lint_content(
            "bad.json",
            r#"{"$schema":"ferro-json-ui/v2","root":"missing","elements":{}}"#,
        );
        assert_eq!(
            findings.len(),
            1,
            "expected exactly 1 spec-parse finding, got: {:#?}",
            findings
                .iter()
                .map(|f| (f.finding.rule, &f.finding.message))
                .collect::<Vec<_>>()
        );
        assert_eq!(findings[0].finding.severity, Severity::Warning);
        assert_eq!(findings[0].finding.rule, "spec-parse");
    }

    #[test]
    fn has_warning_true_when_warn_level_present() {
        let findings = lint_content("warn.json", WARN);
        assert!(
            has_warning(&findings),
            "has_warning must be true for a warning-level finding"
        );
    }

    #[test]
    fn has_warning_false_for_clean_spec() {
        let findings = lint_content("clean.json", CLEAN);
        assert!(
            !has_warning(&findings),
            "has_warning must be false when there are no findings"
        );
    }

    #[test]
    fn has_warning_false_for_skipped_file() {
        let findings = lint_content("skip.json", r#"{"hello":1}"#);
        assert!(
            !has_warning(&findings),
            "has_warning must be false for silently skipped files"
        );
    }
}
