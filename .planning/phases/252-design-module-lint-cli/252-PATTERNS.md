# Phase 252: Design module + lint + CLI — Pattern Map

**Mapped:** 2026-07-03
**Files analyzed:** 14 (6 new files, 8 modified files)
**Analogs found:** 14 / 14

All files have close analogs in the codebase. The design module follows the catalog.rs
validation-engine pattern; the CLI command follows validate_contracts.rs; the app test
follows the existing app/src/tests/ pattern; the type definitions follow action.rs.

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-json-ui/src/spec.rs` | model (wire contract) | transform (JSON↔Rust) | itself — `TitleBinding`/`DataRef` at spec.rs:47-66, optional field serde at spec.rs:85-92 | exact |
| `ferro-json-ui/src/design/types.rs` (NEW) | model (wire contract) | transform | `action.rs:37-44` (`ConfirmDialog` derive stack) + `validate_contracts.rs:16-70` (Serialize output types) | exact |
| `ferro-json-ui/src/design/rules.rs` (NEW) | utility (static registry) | batch | `catalog.rs:124-128` (BUILTIN_SPECS const array) + `catalog.rs:883-890` (RETIRED_PROPS const) | exact |
| `ferro-json-ui/src/design/infer.rs` (NEW) | utility (heuristics) | transform | `catalog.rs:760-776` (Stage 2b element walk over `spec.elements`) | role-match |
| `ferro-json-ui/src/design/mod.rs` (NEW) | utility (rule engine) | batch | `catalog.rs:689-803` (`Catalog::validate` orchestration + Stage dispatch) | exact |
| `ferro-json-ui/src/lib.rs` | re-export surface | N/A | itself — `#[cfg(feature = "projections")] pub mod projection` at lib.rs:91-95 | exact |
| `ferro-json-ui/src/catalog.rs` | service (validation) | batch | itself — `collect_retired_action_variants` at catalog.rs:898-923 + Stage 2b at catalog.rs:754-777 | exact |
| `ferro-cli/src/commands/design_lint.rs` (NEW) | command (CLI) | file-I/O + batch | `validate_contracts.rs:1-924` (json+human output, run() signature, `style()` formatting) | exact |
| `ferro-cli/src/main.rs` | config (CLI enum) | request-response | itself — `ValidateContracts` at main.rs:487-496 + `JsonUiSchema` at main.rs:357-371 | exact |
| `ferro-cli/src/commands/mod.rs` | config (module registry) | N/A | itself — existing `pub mod validate_contracts;` line | exact |
| `app/src/tests/design_lint.rs` (NEW) | test | file-I/O + batch | `app/src/tests/visual_action.rs:1-17` (module-level doc + cfg guard + imports) | role-match |
| `app/src/tests/mod.rs` | config (test registry) | N/A | itself | exact |
| `app/Cargo.toml` | config (dependencies) | N/A | itself — `ferro-projections` dev-dep at Cargo.toml:48 | exact |
| `app/src/views/*.json` (3 files) | config (fixtures) | N/A | themselves — existing `$schema` + `layout` + `elements` structure | exact |

---

## Pattern Assignments

### `ferro-json-ui/src/spec.rs` — add `DesignMeta` struct + `Spec.design` field

**Analog:** `spec.rs` itself — optional field serde pattern at lines 85-92, `TitleBinding` at lines 47-56.

**Imports pattern** (lines 14-23): no new imports needed; `schemars::JsonSchema`, `serde::{Serialize, Deserialize}` are already imported.

**Optional field serde pattern** (lines 85-92 — copy this exactly for `Spec.design`):
```rust
// spec.rs:85-92 — the #[serde(default, skip_serializing_if)] pattern for optional Spec fields
#[serde(default, skip_serializing_if = "Option::is_none")]
pub title: Option<TitleBinding>,
/// Optional layout name (e.g. `"dashboard"`, `"app"`).
#[serde(default, skip_serializing_if = "Option::is_none")]
pub layout: Option<String>,
/// Arbitrary data payload consumed by data-path references inside elements.
#[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
pub data: Value,
```

**New `DesignMeta` struct** — define immediately above `Spec`, mirroring `TitleBinding` at lines 47-56. Per RESEARCH Pitfall 1, define `DesignMeta` in `spec.rs` (not `design/types.rs`) and re-export it from `design/` with `pub use crate::spec::DesignMeta`. This keeps `spec.rs` self-contained:

```rust
// Add after DataRef (spec.rs:66), before Spec (spec.rs:73)
/// Optional design metadata attached to a [`Spec`] for lint and pattern enforcement.
///
/// The `intent` field declares the page archetype (one of the seven projection intents:
/// `browse`, `focus`, `collect`, `process`, `summarize`, `analyze`, `track`).
/// The `allow` field lists rule ids to suppress page-wide.
/// Neither field affects rendering or spec validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DesignMeta {
    /// Page archetype, one of the seven projection intents.
    /// Unknown strings produce a warning finding; they never fail spec parse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    /// Rule ids to suppress for this page. Unknown ids produce a warning finding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
}
```

Add `pub design: Option<DesignMeta>` to `Spec` with `#[serde(default, skip_serializing_if = "Option::is_none")]`, after `data` (line 92).

---

### `ferro-json-ui/src/design/types.rs` (NEW) — `DesignRule`, `Finding`, `Severity`

**Analog:** `action.rs:37-44` (ConfirmDialog derive stack) for `Finding`/`Severity`; `validate_contracts.rs:15-70` (Serialize output struct) for the serialization shape.

**Derive stack** (action.rs:37-38 — use this for `Finding` and `Severity`):
```rust
// action.rs:37-38
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConfirmDialog {
```

**Serialize enum** (validate_contracts.rs:64-70 — the `snake_case` + `Serialize` pattern for `Severity`):
```rust
// validate_contracts.rs:64-70
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Passed,
    Failed,
    Skipped,
}
```

**Concrete types for this file:**
```rust
// ferro-json-ui/src/design/types.rs

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
```

---

### `ferro-json-ui/src/design/rules.rs` (NEW) — static RULE_REGISTRY

**Analog:** `catalog.rs:124-128` (BUILTIN_SPECS static array — same `&'static [(...)]` pattern) and `catalog.rs:883-890` (RETIRED_PROPS — same const-table approach).

**Static registry pattern** (catalog.rs:124-128):
```rust
// catalog.rs:124-128 — type alias + static table pattern
type SchemaFn = fn() -> Value;

/// `(type_name, description, schema_fn, slot_fields)`
static BUILTIN_SPECS: &[(&str, &str, SchemaFn, &[&str])] = &[
    (
        "Text",
        "Semantic text element ...",
        || to_value(schema_for!(TextProps)).unwrap(),
```

**RETIRED_PROPS const pattern** (catalog.rs:883-890):
```rust
// catalog.rs:883-890 — typed-tuple const array, the DesignRule registry mirrors this shape
const RETIRED_PROPS: &[(&str, &str, &str)] = &[
    ("Card", "variant", "appearance"),
    ("Badge", "variant", "tone"),
    // ...
];
```

**Concrete shape for rules.rs:**
```rust
// ferro-json-ui/src/design/rules.rs

use crate::spec::Spec;
use super::types::{DesignRule, Finding, Severity};

pub(super) static RULE_REGISTRY: &[DesignRule] = &[
    DesignRule {
        id: "page-header",
        title: "Dashboard pages must have a PageHeader",
        rationale: "PageHeader provides consistent title, breadcrumb, and action-button placement.",
        intents: &[],          // all intents — layout gate is inside check()
        check: check_page_header,
    },
    DesignRule {
        id: "prefer-data-table",
        title: "Prefer DataTable over raw Table",
        rationale: "DataTable provides mobile card fallback and DropdownMenu row actions.",
        intents: &["browse"],
        check: check_prefer_data_table,
    },
    // ... 8 more rules following the same struct-literal shape
];

fn check_page_header(spec: &Spec, _intent: Option<&str>) -> Vec<Finding> {
    // Gate: only "dashboard" or "app" layouts
    let layout = match spec.layout.as_deref() {
        Some("dashboard") | Some("app") => {},
        _ => return vec![],
    };
    // Check presence of any PageHeader element
    let has_header = spec.elements.values().any(|el| el.type_name == "PageHeader");
    if !has_header {
        return vec![Finding {
            rule: "page-header",
            element_id: None,
            severity: Severity::Warning,
            message: "Dashboard-family layout has no PageHeader element.".into(),
            suggestion: "Add a PageHeader element as first child of root with a `title` prop.".into(),
        }];
    }
    vec![]
}
```

---

### `ferro-json-ui/src/design/infer.rs` (NEW) — intent inference heuristics

**Analog:** `catalog.rs:760-776` (Stage 2b walks `spec.elements` values to detect element-level properties — the same scan-and-collect pattern applies to inference).

**Element scan pattern** (catalog.rs:760-776):
```rust
// catalog.rs:760-776 — iterating spec.elements to detect element-level properties
for (id, el) in &spec.elements {
    let mut renamed: Vec<String> = Vec::new();
    for (ty, old, new) in RETIRED_PROPS {
        if el.type_name == *ty && el.props.get(old).is_some() {
            renamed.push(format!(
                "/{old}: `{old}` was renamed to `{new}` — update the spec"
            ));
        }
    }
    collect_retired_action_variants(&el.props, "", &mut renamed);
    if !renamed.is_empty() {
        errors.push(CatalogError::PropsInvalid { ... });
    }
}
```

**Concrete inference shape for infer.rs:**
```rust
// ferro-json-ui/src/design/infer.rs

use crate::spec::Spec;

/// Infer the dominant intent from spec structure when `design.intent` is absent.
///
/// Signal priority: KanbanBoard → process; Form-dominant → collect;
/// DataTable/Table → browse; StatCard cluster (≥2) → summarize; else None.
/// Returns the inferred intent label or `None` if no signal is found.
pub(super) fn infer_intent(spec: &Spec) -> Option<&'static str> {
    let types: Vec<&str> = spec.elements.values().map(|el| el.type_name.as_str()).collect();

    if types.iter().any(|t| *t == "KanbanBoard") {
        return Some("process");
    }
    // Count root-level Form elements as collect signal
    let form_count = types.iter().filter(|t| **t == "Form").count();
    if form_count >= 1 {
        return Some("collect");
    }
    if types.iter().any(|t| *t == "DataTable" || *t == "Table") {
        return Some("browse");
    }
    let stat_count = types.iter().filter(|t| **t == "StatCard").count();
    if stat_count >= 2 {
        return Some("summarize");
    }
    None
}
```

---

### `ferro-json-ui/src/design/mod.rs` (NEW) — `lint()`, `rules()`, `KNOWN_INTENTS`, drift test

**Analog:** `catalog.rs:689-803` — `Catalog::validate()` orchestrates Stages 1/2/2b/3 in sequence, collects errors, returns `Result<(), Vec<CatalogError>>`. The `lint()` function mirrors this structure but returns `Vec<Finding>` (never an error).

**Catalog::validate orchestration pattern** (catalog.rs:689-710):
```rust
// catalog.rs:689-710 — orchestration: collect errors from multiple stages
pub fn validate(&self, spec: &crate::spec::Spec) -> Result<(), Vec<CatalogError>> {
    let mut errors: Vec<CatalogError> = Vec::new();

    // === Stage 1: per-element type check ===
    for (id, el) in &spec.elements {
        // ...
        errors.push(CatalogError::UnknownType { ... });
    }

    // === Stage 2b: retired prop names ===
    for (id, el) in &spec.elements {
        // ...
        if !renamed.is_empty() {
            errors.push(CatalogError::PropsInvalid { ... });
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

**Feature-gated module pattern** (lib.rs:91-95):
```rust
// lib.rs:91-95 — projections-feature-gated module declaration
#[cfg(feature = "projections")]
pub mod projection;

#[cfg(feature = "projections")]
pub use projection::{JsonUiRenderer, ProjectionError, RenderMode, VisualContext};
```

**Concrete mod.rs shape:**
```rust
//! Design lint engine: intent-keyed composition rules for JSON-UI specs.
//!
//! `lint(&Spec)` is pure and static — no I/O, no data resolution. It runs on
//! the raw spec before `$each`/`$if` expansion. Findings are diagnostics only;
//! they never affect rendering or catalog validation.

mod infer;
mod rules;
pub mod types;

pub use types::{DesignRule, Finding, Severity};
pub use crate::spec::DesignMeta;

/// The seven known projection intents. The drift test (feature = "projections")
/// asserts this set equals `ferro_projections::Intent::label()` for all known variants.
pub const KNOWN_INTENTS: &[&str] = &[
    "browse", "focus", "collect", "process", "summarize", "analyze", "track",
];

/// Return a reference to the static rule registry.
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
pub fn lint(spec: &Spec) -> Vec<Finding> {
    // ... intent resolution, allow-list validation, rule dispatch
}

// ── D-08 drift test ───────────────────────────────────────────────────────────

#[cfg(all(test, feature = "projections"))]
mod drift_tests {
    use super::KNOWN_INTENTS;
    use ferro_projections::Intent;

    #[test]
    fn design_intents_match_projection_intent_labels() {
        let projection_labels: Vec<&str> = [
            Intent::Browse, Intent::Focus, Intent::Collect, Intent::Process,
            Intent::Summarize, Intent::Analyze, Intent::Track,
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
```

---

### `ferro-json-ui/src/lib.rs` — add `pub mod design`

**Analog:** lib.rs:91-95 (feature-gated `pub mod projection` block). The `design` module is unconditional (no feature gate — D-07), so it follows the simpler pattern of the existing unconditional modules at lines 29-45:

```rust
// lib.rs:29-45 — unconditional module declarations
pub mod action;
pub mod assets;
pub mod catalog;
// ...
```

Add `pub mod design;` in alphabetical order between `pub mod data;` and `pub mod expression;`. Add to pub-use block:
```rust
pub use design::{lint, rules, DesignMeta, DesignRule, Finding, Severity, KNOWN_INTENTS};
```

---

### `ferro-json-ui/src/catalog.rs` — extend Stage 2b for el.action gap (D-16 Option A)

**Analog:** The existing `collect_retired_action_variants` function at catalog.rs:898-923, and its call site at catalog.rs:769.

**Existing call site** (catalog.rs:769 — add a second call for el.action):
```rust
// catalog.rs:760-776 — Stage 2b, existing call site
for (id, el) in &spec.elements {
    let mut renamed: Vec<String> = Vec::new();
    for (ty, old, new) in RETIRED_PROPS {
        if el.type_name == *ty && el.props.get(old).is_some() {
            renamed.push(format!(
                "/{old}: `{old}` was renamed to `{new}` — update the spec"
            ));
        }
    }
    collect_retired_action_variants(&el.props, "", &mut renamed);
    // ADD: also walk el.action as a Value to catch confirm.variant on typed Action
    if !renamed.is_empty() { ... }
}
```

**Pattern to add** (after the existing `collect_retired_action_variants(&el.props, ...)` call):
```rust
// Serialize el.action to Value so collect_retired_action_variants can walk it.
// This catches `el.action.confirm.variant` which serde drops silently (typed struct
// has no `variant` field since Phase 251, but old specs may still carry it).
if let Some(action) = &el.action {
    if let Ok(action_value) = serde_json::to_value(action) {
        collect_retired_action_variants(&action_value, "/action", &mut renamed);
    }
}
```

---

### `ferro-cli/src/commands/design_lint.rs` (NEW) — CLI command impl

**Analog:** `validate_contracts.rs` — same `run(…, json: bool)` signature, same `console::style` formatting, same `serde_json::to_string_pretty` for `--json`, same `std::process::exit(1)` for hard failures.

**Imports pattern** (validate_contracts.rs:1-14):
```rust
// validate_contracts.rs:1-14
use console::style;
use serde::Serialize;
use std::fs;
use std::path::Path;
```

For design_lint.rs, swap `regex` for `walkdir`:
```rust
use console::style;
use ferro_json_ui::design::{lint, Finding, Severity};
use ferro_json_ui::spec::{Spec, SCHEMA_VERSION};
use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
```

**WalkDir file-discovery pattern** (generate_routes.rs:444-458):
```rust
// generate_routes.rs:444-458 — WalkDir + filter_map + extension filter
for entry in WalkDir::new(&src_path)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
{
    if let Ok(content) = fs::read_to_string(entry.path()) { ... }
}
```

For design_lint, filter for `.json` and gate on `$schema`:
```rust
for entry in WalkDir::new(&views_path)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
{
    let path = entry.path();
    let Ok(content) = std::fs::read_to_string(path) else { continue };
    // Skip non-ferro-json-ui files silently
    if !content.contains(SCHEMA_VERSION) { continue }
    // ...
}
```

**JSON output pattern** (validate_contracts.rs:886-908):
```rust
// validate_contracts.rs:886-908 — json branch with serde_json::to_string_pretty
if json {
    match serde_json::to_string_pretty(&result) {
        Ok(json_output) => println!("{json_output}"),
        Err(e) => {
            eprintln!("{} Failed to serialize results: {}", style("Error:").red().bold(), e);
            std::process::exit(1);
        }
    }
} else {
    // human output
    print_results(&result);
}
```

**--deny exit-code pattern:**
```rust
// After output, apply --deny gate
if deny && has_warnings {
    std::process::exit(1);
}
```

**run() signature:**
```rust
/// Main entry point for the `design:lint` command.
pub fn run(path: Option<String>, json: bool, deny: bool) {
    let views_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("src/views"));
    // ...
}
```

---

### `ferro-cli/src/main.rs` — add `DesignLint` variant + dispatch

**Analog:** `ValidateContracts` at main.rs:487-496 (colon-namespaced command with `--json` flag) and `JsonUiSchema` at main.rs:357-371 (optional path + output flag).

**Command declaration** (main.rs:487-496 — copy this shape):
```rust
// main.rs:487-496 — ValidateContracts is the closest model
/// Validate Inertia frontend/backend prop contracts
#[command(name = "validate:contracts")]
ValidateContracts {
    /// Filter by route or component name
    #[arg(long, short = 'f')]
    filter: Option<String>,

    /// Output results as JSON
    #[arg(long)]
    json: bool,
},
```

**New variant to add** (follow the same pattern, add after `ValidateContracts`):
```rust
/// Walk spec files and report design pattern findings
#[command(name = "design:lint")]
DesignLint {
    /// Directory or file to lint (default: src/views)
    path: Option<String>,
    /// Emit machine-readable JSON instead of human output
    #[arg(long)]
    json: bool,
    /// Exit non-zero when any warning-level finding exists (CI mode)
    #[arg(long)]
    deny: bool,
},
```

**Dispatch arm** (main.rs:788-789 — copy this shape):
```rust
// main.rs:788-789
Commands::ValidateContracts { filter, json } => {
    commands::validate_contracts::run(filter, json)
}
```

New arm:
```rust
Commands::DesignLint { path, json, deny } => {
    commands::design_lint::run(path, json, deny);
}
```

---

### `ferro-cli/src/commands/mod.rs` — add `pub mod design_lint;`

**Analog:** itself — existing `pub mod validate_contracts;` line (line 60). Insert `pub mod design_lint;` in alphabetical order:

```rust
// commands/mod.rs — insert between db_sync and deploy_init
pub mod db_sync;
pub mod deploy_init;
// becomes:
pub mod db_sync;
pub mod design_lint;   // NEW
pub mod deploy_init;
```

---

### `app/src/tests/design_lint.rs` (NEW) — D-17 lint-clean gate

**Analog:** `app/src/tests/visual_action.rs:1-17` — module-level doc comment, `#[cfg(test)]` nesting, path-independent `env!("CARGO_MANIFEST_DIR")` for finding test fixtures.

**Module-level doc + cfg pattern** (visual_action.rs:1-18):
```rust
// visual_action.rs:1-18 — module doc + cfg gate + test submodule
//! Visual/form write-surface fixtures for Phase 232 SC2 (EXEC-05).
//! ...

#[cfg(all(test, not(feature = "confirmation")))]
mod tests {
    use crate::migrations::Migrator;
    // ...
```

**Concrete test shape:**
```rust
//! D-17 lint-clean gate: every view under `app/src/views/*.json` must lint
//! clean when `design.intent` is declared (zero findings).

#[cfg(test)]
mod tests {
    use ferro_json_ui::design::lint;
    use ferro_json_ui::spec::Spec;

    #[test]
    fn app_views_lint_clean() {
        let views_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/views");
        let entries =
            std::fs::read_dir(views_dir).expect("app/src/views must exist");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let spec = Spec::from_json(&content)
                .unwrap_or_else(|e| panic!("parse {}: {e:?}", path.display()));
            let findings = lint(&spec);
            assert!(
                findings.is_empty(),
                "{}: {} finding(s)\n{:#?}",
                path.display(),
                findings.len(),
                findings
            );
        }
    }
}
```

---

### `app/src/tests/mod.rs` — add `pub mod design_lint;`

**Analog:** itself — existing module declarations. Add `pub mod design_lint;` in alphabetical order:

```rust
// app/src/tests/mod.rs — existing list
pub mod computed_total_e2e;
pub mod crud_e2e;
// becomes:
pub mod computed_total_e2e;
pub mod crud_e2e;
pub mod design_lint;    // NEW
```

---

### `app/Cargo.toml` — add `ferro-json-ui` dev-dependency

**Analog:** `app/Cargo.toml:48` — `ferro-projections` dev-dep as the established precedent for path-based dev-deps in the app crate:

```toml
# app/Cargo.toml:48 — existing precedent
ferro-projections = { path = "../ferro-projections" }
```

Add:
```toml
ferro-json-ui = { path = "../ferro-json-ui" }
```

---

### `app/src/views/*.json` — add `design` field

**Analog:** the three existing files themselves — same `$schema` + flat elements structure.

Required additions (RESEARCH §Sample App Views Analysis):

**login.json** — add after `"layout": "auth"`:
```json
"design": { "intent": "collect" }
```

**login_confirm.json** — add after `"layout": "auth"`:
```json
"design": { "intent": "collect" }
```

**pagamenti.json** — add after `"layout": "dashboard"`, AND add a `PageHeader` element:
```json
"design": { "intent": "summarize" }
```
Also add `"page_header"` element to `elements` with `type: "PageHeader"`, `props.title: "Pagamenti"`, and wire it into `root.children` as first child. This makes the spec architecturally correct (RESEARCH recommendation over allow-listing).

---

## Shared Patterns

### Serde optional-field discipline
**Source:** `ferro-json-ui/src/spec.rs:85-92`
**Apply to:** `DesignMeta` fields, `Spec.design` field
```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub field: Option<T>,
// For Vec fields use:
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub allow: Vec<String>,
```

### Serde enum snake_case + JsonSchema
**Source:** `ferro-projections/src/intent.rs:13-14` and `validate_contracts.rs:65-66`
**Apply to:** `Severity`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
```

### console::style human output
**Source:** `ferro-cli/src/commands/validate_contracts.rs:8` and throughout
**Apply to:** `design_lint.rs` human-readable output
```rust
use console::style;
// Usage:
eprintln!("{} Not a Ferro project", style("Error:").red().bold());
println!("{}", style("Scanning...").cyan());
println!("{} {}", style("->").green(), message);
```

### Feature-gated test module isolation
**Source:** `ferro-json-ui/src/projection/builder.rs:17` (`#![cfg(feature = "projections")]`)
**Apply to:** D-08 drift test in `design/mod.rs`
```rust
#[cfg(all(test, feature = "projections"))]
mod drift_tests {
    // entire module gated — not just the #[test] fn
}
```

### WalkDir recursive file scan
**Source:** `ferro-cli/src/commands/generate_routes.rs:444-458`
**Apply to:** `design_lint.rs` file discovery
```rust
use walkdir::WalkDir;

for entry in WalkDir::new(&path)
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
{
    // ...
}
```

### serde_json::to_string_pretty for --json output
**Source:** `ferro-cli/src/commands/validate_contracts.rs:898`
**Apply to:** `design_lint.rs` JSON output branch
```rust
match serde_json::to_string_pretty(&findings) {
    Ok(json_output) => println!("{json_output}"),
    Err(e) => {
        eprintln!("{} Failed to serialize: {}", style("Error:").red().bold(), e);
        std::process::exit(1);
    }
}
```

### Rustdoc discipline for new public module
**Source:** `ferro-json-ui/src/spec.rs:1-13` (module-level `//!` doc) and `catalog.rs:80-113` (per-type `///` docs)
**Apply to:** All public items in `ferro_json_ui::design` — `DesignMeta`, `DesignRule`, `Finding`, `Severity`, `lint()`, `rules()`, `KNOWN_INTENTS`
Per RESEARCH Pitfall 5: CI docs build uses `-D warnings` — every public item needs at least a one-line `///` comment.

---

## No Analog Found

All files have close analogs. No entries in this section.

---

## Metadata

**Analog search scope:** `ferro-json-ui/src/`, `ferro-cli/src/`, `ferro-projections/src/`, `app/src/`
**Files scanned:** ~20 source files read directly; additional discovery via grep/glob
**Pattern extraction date:** 2026-07-03
