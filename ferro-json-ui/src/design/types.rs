//! Public types for the design lint engine: [`Severity`], [`Finding`], [`DesignRule`].

use crate::spec::Spec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Severity level for a design lint finding.
///
/// `Warning` is the actionable level — it trips `--deny` in CI mode.
/// `Info` is advisory only and never causes a non-zero exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Advisory: inference notices, undeclared-intent notes.
    Info,
    /// Actionable: trips `--deny` in CI mode.
    Warning,
}

/// A single design lint finding from [`super::lint`].
///
/// The `--json` CLI output is a flat array of `FileFinding` (this struct
/// wrapped with a `file` field). This serialization is the stable contract
/// consumed by gestiscilo Phase 232 CI.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Finding {
    /// Rule id (e.g. `"page-header"`, `"prefer-data-table"`).
    pub rule: &'static str,
    /// Element ID where the finding originates, when identifiable.
    pub element_id: Option<String>,
    /// Severity level of this finding.
    pub severity: Severity,
    /// Human-readable description of what is wrong.
    pub message: String,
    /// Concrete fix suggestion.
    pub suggestion: String,
}

/// A single entry in the static rule registry. All fields are `'static`
/// for zero-cost iteration and Phase 253 MCP/doc derivation.
pub struct DesignRule {
    /// Stable rule id used in `allow` lists and finding `rule` fields.
    pub id: &'static str,
    /// Short human title for docs/MCP catalog.
    pub title: &'static str,
    /// Why this rule exists (one sentence).
    pub rationale: &'static str,
    /// Intents this rule applies to. Empty slice = all intents.
    pub intents: &'static [&'static str],
    /// Pure check function. Receives the raw [`Spec`] and the resolved intent
    /// (may be `None` when intent is completely undeclared and inference found
    /// no signal). Returns zero or more findings.
    pub check: fn(&Spec, Option<&str>) -> Vec<Finding>,
}
