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
                Some(i) => format!(
                    "No design.intent declared; inferred `{i}` from spec content."
                ),
                None => "No design.intent declared and none could be inferred from spec content.".into(),
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
        let known = rules::RULE_REGISTRY.iter().any(|r| r.id == id.as_str()) || id == "declare-intent";
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
            design,
            proj,
            "KNOWN_INTENTS in design module drifted from ferro_projections::Intent labels"
        );
    }
}
