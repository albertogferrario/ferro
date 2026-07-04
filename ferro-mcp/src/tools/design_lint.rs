//! design_lint tool — runs the ferro-json-ui design rule engine on a single spec,
//! either provided inline as JSON or read from a file path.
//!
//! Per Phase 253 D-01/D-04: input is spec_json XOR path; parse errors and XOR
//! violations are returned as Warning-level findings, never as tool errors.
//! The output shape is identical to the CLI `--json` envelope (252 D-11).

use ferro_json_ui::design::{lint, Finding, Severity};
use ferro_json_ui::spec::{Spec, SCHEMA_VERSION};
use serde::Serialize;

/// One finding tagged with the file it originated from.
///
/// Stable `--json` / MCP contract consumed by gestiscilo Phase 232.
/// Identical to `ferro_cli::commands::design_lint::FileFinding` by design (D-02).
#[derive(Debug, Serialize)]
pub struct FileFinding {
    /// "<inline>" for spec_json input; the given path for path input.
    pub file: String,
    #[serde(flatten)]
    pub finding: Finding,
}

/// Lint exactly one spec: inline `spec_json` XOR file `path`.
pub fn execute(spec_json: Option<&str>, path: Option<&str>) -> Vec<FileFinding> {
    match (spec_json, path) {
        (Some(json), None) => lint_string("<inline>", json),
        (None, Some(p)) => match std::fs::read_to_string(p) {
            Ok(content) => lint_string(p, &content),
            Err(e) => vec![FileFinding {
                file: p.to_string(),
                finding: Finding {
                    rule: "file-read",
                    element_id: None,
                    severity: Severity::Warning,
                    message: format!("Could not read file: {e}"),
                    suggestion: "Check file path and permissions.".into(),
                },
            }],
        },
        _ => vec![FileFinding {
            file: "<tool-input>".to_string(),
            finding: Finding {
                rule: "tool-input",
                element_id: None,
                severity: Severity::Warning,
                message: "Provide exactly one of spec_json or path, not both and not neither."
                    .into(),
                suggestion: "Pass spec_json for inline linting or path for file linting.".into(),
            },
        }],
    }
}

fn lint_string(label: &str, content: &str) -> Vec<FileFinding> {
    if !content.contains(SCHEMA_VERSION) {
        // Non-ferro JSON: silently skip (same as CLI WalkDir behaviour).
        return vec![];
    }
    match Spec::from_json(content) {
        Ok(spec) => lint(&spec)
            .into_iter()
            .map(|finding| FileFinding {
                file: label.to_string(),
                finding,
            })
            .collect(),
        Err(e) => vec![FileFinding {
            file: label.to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    // A minimal focus-intent spec that declares intent and layout so the design
    // rules have no grounds to emit findings (page-header is exempt on "auth" layout).
    const CLEAN: &str = r#"{"$schema":"ferro-json-ui/v2","root":"t","layout":"auth","design":{"intent":"focus","allow":["page-header"]},"elements":{"t":{"type":"Text","props":{"content":"hi"}}}}"#;

    #[test]
    fn inline_clean_spec_returns_empty() {
        let findings = execute(Some(CLEAN), None);
        assert!(
            findings.is_empty(),
            "expected no findings for a clean spec, got: {findings:?}"
        );
    }

    #[test]
    fn inline_malformed_returns_spec_parse_warning() {
        // Contains the schema marker so lint_string runs the parser path,
        // but root "missing" is not in elements so Spec::from_json fails.
        let findings = execute(
            Some(r#"{"$schema":"ferro-json-ui/v2","root":"missing","elements":{}}"#),
            None,
        );
        assert_eq!(
            findings.len(),
            1,
            "expected exactly 1 finding, got: {findings:?}"
        );
        assert_eq!(findings[0].finding.rule, "spec-parse");
        assert_eq!(findings[0].file, "<inline>");
        assert_eq!(findings[0].finding.severity, Severity::Warning);
    }

    #[test]
    fn path_mode_reads_and_lints() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(CLEAN.as_bytes()).unwrap();
        let p = f.path().to_str().unwrap().to_string();
        let findings = execute(None, Some(&p));
        assert!(
            findings.is_empty(),
            "clean spec should produce no findings, got: {findings:?}"
        );
    }

    #[test]
    fn both_none_returns_tool_input_warning() {
        let findings = execute(None, None);
        assert_eq!(
            findings.len(),
            1,
            "expected exactly 1 finding, got: {findings:?}"
        );
        assert_eq!(findings[0].finding.rule, "tool-input");
        assert_eq!(findings[0].finding.severity, Severity::Warning);
    }

    #[test]
    fn both_some_returns_tool_input_warning() {
        let findings = execute(Some(CLEAN), Some("x"));
        assert_eq!(
            findings.len(),
            1,
            "expected exactly 1 finding, got: {findings:?}"
        );
        assert_eq!(findings[0].finding.rule, "tool-input");
        assert_eq!(findings[0].finding.severity, Severity::Warning);
    }
}
