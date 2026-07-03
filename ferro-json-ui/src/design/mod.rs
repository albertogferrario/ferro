//! Design lint engine: intent-keyed composition rules for JSON-UI specs.
//!
//! `lint(&Spec)` is pure and static — no I/O, no data resolution. It runs on
//! the raw spec before `$each`/`$if` expansion. Findings are diagnostics only;
//! they never affect rendering or catalog validation.
//!
//! # Usage
//!
//! ```rust
//! use ferro_json_ui::{Spec, lint};
//!
//! let spec = Spec::from_json(r#"{
//!   "$schema": "ferro-json-ui/v2",
//!   "root": "r",
//!   "elements": {"r": {"type": "Text"}},
//!   "design": {"intent": "browse"}
//! }"#).unwrap();
//! let findings = lint(&spec);
//! assert!(findings.is_empty());
//! ```

mod infer;
mod rules;
pub mod types;

pub use crate::spec::DesignMeta;
pub use types::{DesignRule, Finding, Severity};

use crate::spec::Spec;
use types::Severity::{Info, Warning};

/// The seven known projection intents.
///
/// The drift test (feature `"projections"`) asserts this set equals
/// `ferro_projections::Intent::label()` for all known variants, so the
/// "archetypes ARE the projection intents" invariant is guarded by CI.
pub const KNOWN_INTENTS: &[&str] = &[
    "browse",
    "focus",
    "collect",
    "process",
    "summarize",
    "analyze",
    "track",
];

/// Return a reference to the static design-rule registry.
///
/// Phase 253 derives the pattern-catalog docs and MCP guidance from this iterator.
pub fn rules() -> &'static [DesignRule] {
    rules::RULE_REGISTRY
}

/// Run all applicable design rules against `spec` and return findings.
///
/// Findings are pure diagnostics — they never cause a parse error or affect
/// rendering. Info-level findings are advisory; Warning-level findings trip
/// `ferro design:lint --deny`.
///
/// The engine never panics and performs no I/O.
pub fn lint(spec: &Spec) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    let design = spec.design.as_ref();
    let allow: &[String] = design.map(|d| d.allow.as_slice()).unwrap_or(&[]);

    // ── Step 1: intent resolution ─────────────────────────────────────────────
    let resolved: Option<&str> = match design.and_then(|d| d.intent.as_deref()) {
        Some(s) if KNOWN_INTENTS.contains(&s) => {
            // Valid declared intent — no finding.
            Some(s)
        }
        Some(s) => {
            // Unknown declared intent — warning + fall back to inference.
            findings.push(Finding {
                rule: "declare-intent",
                element_id: None,
                severity: Warning,
                message: format!(
                    "Unknown design.intent `{s}`; expected one of the seven projection intents."
                ),
                suggestion: "Use one of: browse, focus, collect, process, summarize, analyze, track.".into(),
            });
            infer::infer_intent(spec)
        }
        None => {
            // No declared intent — infer and emit Info finding.
            let inferred = infer::infer_intent(spec);
            let message = match inferred {
                Some(i) => format!("No design.intent declared; inferred `{i}` from spec content."),
                None => {
                    "No design.intent declared and none could be inferred from spec content.".into()
                }
            };
            findings.push(Finding {
                rule: "declare-intent",
                element_id: None,
                severity: Info,
                message,
                suggestion: "Add a `design.intent` field to declare the page archetype.".into(),
            });
            inferred
        }
    };

    // ── Step 2: validate allow ids ────────────────────────────────────────────
    // Known ids = all rule ids in the registry PLUS the engine finding "declare-intent".
    for id in allow {
        let known =
            rules::RULE_REGISTRY.iter().any(|r| r.id == id.as_str()) || id == "declare-intent";
        if !known {
            findings.push(Finding {
                rule: "allow",
                element_id: None,
                severity: Warning,
                message: format!("Unknown allow id `{id}`."),
                suggestion: "Remove it or fix the typo; allow ids must match a rule id.".into(),
            });
        }
    }

    // ── Step 3: dispatch rules ────────────────────────────────────────────────
    for rule in rules::RULE_REGISTRY {
        if !rule.intents.is_empty() {
            match resolved {
                Some(i) if rule.intents.contains(&i) => {}
                _ => continue,
            }
        }
        findings.extend((rule.check)(spec, resolved));
    }

    // ── Step 4: suppress allow-listed findings ────────────────────────────────
    findings.retain(|f| !allow.iter().any(|a| a == f.rule));

    findings
}

// ── Engine unit tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod engine_tests {
    use super::*;
    use crate::spec::Spec;

    /// Helper: minimal spec JSON with a single element of the given type.
    fn spec_with(element_type: &str) -> Spec {
        Spec::from_json(&format!(
            r#"{{"$schema":"ferro-json-ui/v2","root":"r","elements":{{"r":{{"type":"{element_type}"}}}}}}"#
        ))
        .unwrap()
    }

    /// Helper: spec JSON with a `design` object.
    fn spec_with_design(design_json: &str) -> Spec {
        Spec::from_json(&format!(
            r#"{{"$schema":"ferro-json-ui/v2","root":"r","elements":{{"r":{{"type":"DataTable"}}}},"design":{design_json}}}"#
        ))
        .unwrap()
    }

    /// Helper: spec with a `design` object and an explicit element type.
    fn spec_with_type_and_design(element_type: &str, design_json: &str) -> Spec {
        Spec::from_json(&format!(
            r#"{{"$schema":"ferro-json-ui/v2","root":"r","elements":{{"r":{{"type":"{element_type}"}}}},"design":{design_json}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn undeclared_intent_with_data_table_emits_info_browse() {
        // No design field → inference: DataTable → "browse".
        let spec = spec_with("DataTable");
        let findings = lint(&spec);
        assert_eq!(findings.len(), 1, "expected exactly one finding");
        assert_eq!(findings[0].rule, "declare-intent");
        assert_eq!(findings[0].severity, Severity::Info);
        assert!(
            findings[0].message.contains("browse"),
            "message should mention inferred intent 'browse', got: {}",
            findings[0].message
        );
    }

    #[test]
    fn undeclared_intent_no_signal_emits_info_none_inferred() {
        // Text-only spec → no inference signal.
        let spec = spec_with("Text");
        let findings = lint(&spec);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, "declare-intent");
        assert_eq!(findings[0].severity, Severity::Info);
        assert!(
            findings[0].message.contains("none could be inferred"),
            "message should mention no inference, got: {}",
            findings[0].message
        );
    }

    #[test]
    fn unknown_declared_intent_emits_warning() {
        let spec = spec_with_design(r#"{"intent":"totally-made-up"}"#);
        let findings = lint(&spec);
        assert_eq!(findings.len(), 1, "expected exactly one finding");
        assert_eq!(findings[0].rule, "declare-intent");
        assert_eq!(findings[0].severity, Severity::Warning);
        assert!(
            findings[0].message.contains("totally-made-up"),
            "message should include the bad intent string, got: {}",
            findings[0].message
        );
    }

    #[test]
    fn unknown_allow_id_emits_warning() {
        // Use a permanently-unknown id — not a real rule id.
        let spec = spec_with_type_and_design(
            "DataTable",
            r#"{"intent":"browse","allow":["no-such-rule"]}"#,
        );
        let findings = lint(&spec);
        // "browse" declared → no declare-intent finding. "no-such-rule" unknown → 1 warning.
        assert_eq!(
            findings.len(),
            1,
            "expected one finding for unknown allow id"
        );
        assert_eq!(findings[0].rule, "allow");
        assert_eq!(findings[0].severity, Severity::Warning);
        assert!(
            findings[0].message.contains("no-such-rule"),
            "message should include the bad allow id, got: {}",
            findings[0].message
        );
    }

    #[test]
    fn allow_declare_intent_suppresses_info_finding() {
        // No design.intent declared + allow:["declare-intent"] → zero findings.
        let spec = spec_with_type_and_design("Text", r#"{"allow":["declare-intent"]}"#);
        let findings = lint(&spec);
        assert!(
            findings.is_empty(),
            "allow:declare-intent should suppress the info finding, got: {findings:#?}"
        );
    }

    #[test]
    fn valid_declared_intent_with_data_table_zero_findings() {
        // Empty registry: no composition rules → zero findings for valid intent.
        let spec = spec_with_type_and_design("DataTable", r#"{"intent":"browse"}"#);
        let findings = lint(&spec);
        assert!(
            findings.is_empty(),
            "valid declared intent + empty registry should produce zero findings, got: {findings:#?}"
        );
    }
}

// ── D-08 drift test ───────────────────────────────────────────────────────────

#[cfg(all(test, feature = "projections"))]
mod drift_tests {
    use super::KNOWN_INTENTS;
    use ferro_projections::Intent;

    #[test]
    fn design_intents_match_projection_intent_labels() {
        let projection_labels: Vec<&str> = [
            Intent::Browse,
            Intent::Focus,
            Intent::Collect,
            Intent::Process,
            Intent::Summarize,
            Intent::Analyze,
            Intent::Track,
        ]
        .iter()
        .map(|i| i.label())
        .collect();
        let mut design = KNOWN_INTENTS.to_vec();
        design.sort_unstable();
        let mut proj = projection_labels.clone();
        proj.sort_unstable();
        assert_eq!(
            design, proj,
            "KNOWN_INTENTS in design module drifted from ferro_projections::Intent labels"
        );
    }
}
