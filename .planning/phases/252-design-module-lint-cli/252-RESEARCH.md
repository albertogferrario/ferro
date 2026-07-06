# Phase 252: Design Module + Lint + CLI — Research

**Researched:** 2026-07-03
**Domain:** ferro-json-ui design rule engine, ferro-cli command registration, app crate test infrastructure
**Confidence:** HIGH — all findings verified against codebase source; no unverified claims

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Wire shape and field semantics (D-01 through D-09)**
- `"design": {"intent": "browse", "allow": ["prefer-data-table"]}` — one optional serde-default field on `Spec`, absent from serialized output when unset (`skip_serializing_if`).
- Intent values are the seven projection intents (`browse`, `focus`, `collect`, `process`, `summarize`, `analyze`, `track`). Invalid intent values and unknown `allow` ids → findings, never errors.
- Rule set: 10 rules from anchor spec §3 (page-header, prefer-data-table, list-empty-state, row-actions-grouped, process-kanban, create-separate-page, breadcrumb-on-subpages, form-default-values, destructive-confirmation, card-actions-in-menu), each with a violating + conforming unit-test pair.
- `Severity`: `Info | Warning` only. `Finding` carries `rule`, `element_id: Option<String>`, `severity`, `message`, `suggestion`.
- CLI contract: `ferro design:lint [path] [--json] [--deny]`, default path `src/views`, recursive over `*.json`, human-readable findings grouped by file, `--json` for machine consumption, exit 0 always unless `--deny`.
- Undeclared intent is inferred from spec content and reported as an info-level finding.
- `Spec.design` and the whole `design` module compile WITHOUT the `projections` feature — string-typed `DesignMeta { intent: Option<String>, allow: Vec<String> }` makes D-02 structural.
- Drift test asserting design module's archetype labels equal `ferro_projections::Intent::label()` — gate behind the `projections` feature (CI `--all-features` enforces it).
- A declared intent outside the seven → warning finding (`unknown intent`), lint falls back to inference path.
- Rules are a static registry — `DesignRule { id, title, rationale, intents, check }` with a public `design::rules()` iterator. Metadata is public for Phase 253 docs/MCP derivation.
- `Finding` (and `Severity`) derive `Serialize` (+ `JsonSchema`). `--json` shape is the stable contract.
- `lint(&Spec) -> Vec<Finding>` is pure and static: no I/O, runs on raw spec before `$each`/`$if` expansion.
- `allow` exempts rule ids page-wide. Unknown `allow` ids → warning finding.
- `page-header` rule fires on layout values `"dashboard"` and `"app"`. `"auth"`, custom layouts, layout-less specs do not trigger.
- `destructive-confirmation`: styled destructive = `variant: destructive` on Button/ActionGroup items; conformance = `Action.confirm` field (`ConfirmDialog`) present. Covers both element-level `action` and props-embedded actions.
- **D-16 (stale-prop handoff from Phase 251):** stale-prop / migration-hygiene diagnostics join the rule set as an additional all-intents rule. Locate WR-01, decide single-home placement (no duplicate control surface). Research directive: find the gap, the planner decides.
- Sample `app/` views lint clean: enforced by a test in `app` crate walking `app/src/views/*.json`. App views DECLARE `design.intent` — gate asserts zero findings.
- Per-rule violating + conforming test pairs in the design module's tests. Inference heuristics get their own coverage.
- No ferro-mcp changes this phase (Phase 253 owns MCP surface). Component count stays 47.
- CI-exact gate before commit: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, plus docs build.

### Claude's Discretion
- Rule engine internals: fn pointers vs trait objects, one file per rule vs grouped modules under `design/`.
- Inference heuristic details: tie-breaking, "StatCard cluster" threshold, fallback when nothing matches.
- Human-readable CLI output formatting (grouping, colors, summary counts).
- CLI file-discovery edge semantics: recommended — JSON files without `"$schema": "ferro-json-ui/v2"` marker skipped silently; files with marker that fail `Spec` parse reported as warning-level file diagnostics.
- OQ-3 `dot_colors` raw-Tailwind lint — optional bonus rule.
- Exact `--json` envelope (flat findings array with `file` field vs grouped by file).

### Deferred Ideas (OUT OF SCOPE)
- `design_lint` MCP tool, `json_ui_catalog` / `generation_context` extensions, `docs/src/design-system/` chapter, crates.io publish — Phase 253.
- CSS-hygiene lint (dead utility definitions in `ferro-base.css` from negative test assertions) — different artifact class, not this phase.
- OQ-3 `dot_colors` raw-Tailwind rule — discretionary; may be deferred to gestiscilo FRICTION.md loop.
- gestiscilo reference-case adoption (68-spec sweep, `--deny` CI gate, FRICTION.md) — gestiscilo Phase 232, gated on Phase 253 publish.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DS-05 | `Spec` gains optional `design` field; pure `design::lint` engine implements ~10 intent-keyed rules; intent inferred with info-level finding when undeclared; lint never affects rendering or validation; each rule has violating/conforming unit-test pair | Spec struct shape confirmed (spec.rs:73); serde pattern for optional field established; `projections` feature gate verified; props shapes for all 10 rules mapped below |
| DS-06 | `ferro design:lint [path] [--json] [--deny]` CLI — recursive over spec JSON files, human-readable + `--json` output, exit 0 always unless `--deny` | clap colon-command pattern verified (main.rs:345,358); `--json` precedent in validate_contracts.rs; `--deny` exit-code pattern established; commands/mod.rs registration pattern confirmed |
</phase_requirements>

---

## Summary

Phase 252 adds a pure diagnostic layer to ferro-json-ui: a `design` module that expresses the dashboard-page patterns from CLAUDE.md as a versioned, intent-keyed rule set, plus a `ferro design:lint` CLI command that surfaces findings. All critical implementation paths are verified against the live codebase.

The most consequential decision is D-16 (stale-prop single-home): the WR-01 lint already in `catalog.rs` Stage 2b is a hard-error validator covering element-level props and props-embedded actions, but it cannot reach the element-level typed `action` field. The planner must choose between extending `catalog.rs` Stage 2b to close that gap (keeping stale-prop as a hard error in one place) vs. absorbing it into `design::lint` as a Warning rule (making stale-prop detection uniform but downgrading it from hard error). Both are one source of truth. Research recommends extending `catalog.rs` Stage 2b.

The sample app has one lint-triggering gap to resolve before the D-17 gate can pass: `pagamenti.json` uses `layout: "dashboard"` but lacks a `PageHeader` element, so the `page-header` rule fires. The planner must decide: add a `PageHeader` element to the spec, or add `"allow": ["page-header"]` to its `design` field.

**Primary recommendation:** implement `design::lint` in `ferro-json-ui/src/design/` (no new crate), register `DesignLint` in `ferro-cli/src/commands/design_lint.rs`, add `ferro-json-ui` as a dev-dependency to the `app` crate for the D-17 gate test, and extend `catalog.rs` Stage 2b to close the element-level `action` gap (single-home stale-prop detection).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Rule engine + `lint()` | ferro-json-ui (`design/` module) | — | Spec-adjacent; rules inspect the Spec struct that ferro-json-ui owns |
| `design.intent` / `design.allow` wire field | ferro-json-ui (`spec.rs`) | — | Additive optional field on `Spec`, serde-default |
| CLI command `ferro design:lint` | ferro-cli | ferro-json-ui | CLI delegates to `ferro_json_ui::design::lint()`; path traversal in the command |
| App views lint-clean gate | `app` crate (test) | ferro-json-ui | Test in app/src/tests/, calls design::lint on live spec files |
| D-08 drift test | ferro-json-ui (`#[cfg(feature = "projections")]` test) | ferro-projections | Intent-label equality guard |

---

## Standard Stack

### Core (existing crates, no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ferro-json-ui` | workspace | `Spec`, `Element`, `Action`, `ConfirmDialog`, component props | Already owns spec validation and catalog |
| `serde` + `serde_json` | workspace | `Finding` / `Severity` serialization; `--json` CLI output | Established crate pattern |
| `schemars` | `1` (workspace) | `JsonSchema` derive on `Finding` / `Severity` (Phase 253 MCP schema) | All ferro-json-ui public types derive this |
| `walkdir` | `2` | Recursive `*.json` file discovery in ferro-cli | Already in ferro-cli Cargo.toml |
| `console` | `0.15` | Human-readable CLI output (styled findings) | Used by every other ferro-cli command |

[VERIFIED: ferro-json-ui/Cargo.toml, ferro-cli/Cargo.toml — no new dependencies required]

### New dev-dependency (app crate only)

| Library | Version | Purpose | When to Add |
|---------|---------|---------|-------------|
| `ferro-json-ui` | `{ path = "../ferro-json-ui" }` | Direct access to `ferro_json_ui::design::lint` in D-17 test | Add to `app/Cargo.toml [dev-dependencies]` (not a build dependency) |

[VERIFIED: app/Cargo.toml — ferro-json-ui is not yet a dev-dependency; ferro-projections already is a precedent]

---

## Architecture Patterns

### System Architecture Diagram

```
spec.json files on disk
        │
        ▼ (ferro-cli command)
ferro design:lint [path] [--json] [--deny]
        │ file discovery via walkdir (*.json)
        │ filter: $schema == "ferro-json-ui/v2"
        │
        ▼ Spec::from_json(&content)  [ferro-json-ui::spec]
parsed Spec (raw, unexpanded)
        │
        ▼ design::lint(&spec)        [ferro-json-ui::design]
        │
        ├─ DesignMeta extraction (spec.design: Option<DesignMeta>)
        ├─ Intent resolution:
        │    declared → validate against KNOWN_INTENTS (&[&str; 7])
        │    missing  → infer from element type heuristics + emit Info
        │    unknown  → emit Warning, fall back to infer
        ├─ allow-list validation (unknown ids → Warning)
        │
        ├─ for each DesignRule in registry:
        │    if rule.intents is empty OR resolved_intent ∈ rule.intents:
        │        rule.check(&spec, resolved_intent) → Vec<Finding>
        │        filter out rule ids in spec.design.allow
        │
        └─ Vec<Finding>              ← returned to caller
                │
                ├─ (CLI human mode): grouped by file, colored via console
                └─ (CLI --json mode): serde_json::to_string_pretty(findings_with_file)
                        │
                        └─ exit code: 0 always, unless --deny + any Warning-level finding
```

### Recommended Project Structure

```
ferro-json-ui/src/
├── design/
│   ├── mod.rs          # pub use, module doc, KNOWN_INTENTS const, lint() fn
│   ├── types.rs        # DesignMeta, DesignRule, Finding, Severity (Serialize+JsonSchema)
│   ├── rules.rs        # static RULE_REGISTRY: [DesignRule; N], rules() fn
│   └── infer.rs        # intent inference heuristics
ferro-cli/src/commands/
└── design_lint.rs      # run(path, json, deny) command impl
app/src/tests/
└── design_lint.rs      # D-17 lint-clean gate for app/src/views/*.json
```

### Pattern 1: Spec `design` Field Addition

```rust
// ferro-json-ui/src/spec.rs  (additive to existing Spec struct)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
}

// Inside Spec struct:
#[serde(default, skip_serializing_if = "Option::is_none")]
pub design: Option<DesignMeta>,
```

[VERIFIED: existing Spec fields at spec.rs:73 use identical `#[serde(default, skip_serializing_if = "Option::is_none")]` pattern for `title`, `layout`, `data`]

### Pattern 2: DesignRule Registry

```rust
// ferro-json-ui/src/design/types.rs
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Finding {
    pub rule: &'static str,
    pub element_id: Option<String>,
    pub severity: Severity,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Severity { Info, Warning }

// ferro-json-ui/src/design/mod.rs
pub struct DesignRule {
    pub id: &'static str,
    pub title: &'static str,
    pub rationale: &'static str,
    pub intents: &'static [&'static str],  // empty = all intents
    pub check: fn(&Spec, Option<&str>) -> Vec<Finding>,
}

pub const KNOWN_INTENTS: &[&str] = &[
    "browse", "focus", "collect", "process", "summarize", "analyze", "track",
];

pub fn rules() -> &'static [DesignRule] { &RULE_REGISTRY }
pub fn lint(spec: &Spec) -> Vec<Finding> { /* ... */ }
```

[VERIFIED: anchor spec §3 — matches. `Finding`/`Severity` derive `Serialize` + `JsonSchema` per D-11]

### Pattern 3: D-08 Drift Test (feature-gated)

```rust
// ferro-json-ui/src/design/mod.rs (in #[cfg(test)] block)
#[cfg(all(test, feature = "projections"))]
mod drift_tests {
    use ferro_projections::Intent;
    use super::KNOWN_INTENTS;

    #[test]
    fn design_intents_match_projection_intent_labels() {
        let projection_labels: Vec<&str> = [
            Intent::Browse, Intent::Focus, Intent::Collect, Intent::Process,
            Intent::Summarize, Intent::Analyze, Intent::Track,
        ].iter().map(|i| i.label()).collect();
        let mut design = KNOWN_INTENTS.to_vec();
        design.sort_unstable();
        let mut proj = projection_labels.clone();
        proj.sort_unstable();
        assert_eq!(design, proj,
            "KNOWN_INTENTS in design module drifted from ferro_projections::Intent labels");
    }
}
```

[VERIFIED: ferro-projections/src/intent.rs:label() returns "browse"/"focus"/"collect"/"process"/"summarize"/"analyze"/"track" for the 7 known variants; ferro-json-ui Cargo.toml:14 — ferro-projections already an optional dep under the `projections` feature; CI `cargo test --all-features` enables this feature]

### Pattern 4: ferro-cli Command Registration

```rust
// ferro-cli/src/main.rs — in Commands enum:
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

// In main() match:
Commands::DesignLint { path, json, deny } => {
    commands::design_lint::run(path, json, deny);
}
```

[VERIFIED: existing pattern at main.rs:345 (`db:migrate`), main.rs:358 (`json-ui:schema`); validate_contracts.rs:866 shows the `--json` + `serde_json::to_string_pretty` pattern; commands/mod.rs shows `pub mod json_ui_schema;` registration style]

### Pattern 5: `--json` Output Shape (stable public contract)

```rust
// flatten findings with file path for machine consumption
#[derive(Serialize)]
struct FileFinding<'a> {
    file: &'a str,
    #[serde(flatten)]
    finding: &'a Finding,
}
// output: serde_json::to_string_pretty(file_findings)
```

[ASSUMED — exact envelope not in anchor spec; follow validate_contracts.rs pattern of flat array with context field; document in --help as the stable contract gestiscilo CI will consume]

### Pattern 6: D-17 App Crate Lint-Clean Test

```rust
// app/src/tests/design_lint.rs
#[test]
fn app_views_lint_clean() {
    let views_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/views");
    let entries = std::fs::read_dir(views_dir).expect("views dir must exist");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
        let content = std::fs::read_to_string(&path).unwrap();
        let spec = ferro_json_ui::spec::Spec::from_json(&content).unwrap();
        let findings = ferro_json_ui::design::lint(&spec);
        // All views must declare design.intent (no inference info noise)
        assert!(
            findings.is_empty(),
            "{}: {} finding(s): {:?}",
            path.display(), findings.len(), findings
        );
    }
}
```

[VERIFIED: app/src/tests/ pattern from mod.rs; `env!("CARGO_MANIFEST_DIR")` is standard Rust test practice for paths relative to crate root; ferro-json-ui not yet in app dev-dependencies — must be added]

### Anti-Patterns to Avoid

- **Two stale-prop surfaces**: Do not add a `stale-props` rule in design::lint if catalog.rs Stage 2b already covers it. Extend Stage 2b to close the element-level `action` gap instead.
- **Hard-erroring on invalid intent**: `DesignMeta.intent` is a `String`, not an enum. An invalid string must produce a Warning finding, never a `Spec` parse error (D-02).
- **Running lint inside rendering**: `lint()` is called only by the CLI command and tests, never inside `render()` or `catalog.validate()`.
- **Skipping `cargo doc`**: The new `design` module is public API; `cargo doc` with `-D warnings` runs in CI. All public types need `///` doc comments.

---

## D-16: WR-01 Retired-Prop Lint — Exact Findings

### What exists (verified)

**Location:** `ferro-json-ui/src/catalog.rs:754` (Stage 2b inside `Catalog::validate`)

**Table:** `RETIRED_PROPS` const at line 883:
```rust
const RETIRED_PROPS: &[(&str, &str, &str)] = &[
    ("Card", "variant", "appearance"),
    ("Badge", "variant", "tone"),
    ("Alert", "variant", "tone"),
    ("Toast", "variant", "tone"),
    ("ActionCard", "variant", "tone"),
    ("MediaCardGrid", "badge_variant_key", "badge_tone_key"),
];
```

**Function:** `collect_retired_action_variants` at line 898 — recursively walks `el.props` (a `serde_json::Value`) looking for `confirm` objects and `on_success`/`on_error` notify outcomes that still carry a `variant` key.

**What it covers:** element-level props (`el.props.get(old)`) and props-embedded actions (row_actions, buttons, ActionGroup items that contain `confirm` or notify outcomes).

**Produces:** `CatalogError::PropsInvalid` — a hard validation error (spec fails `catalog.validate()`).

### The element-level `action` gap (verified)

`el.action: Option<Action>` is a typed Rust field deserialized before `validate()` runs. `ConfirmDialog` (action.rs:38) does NOT have a `variant` field — it has `tone: Tone`. So a JSON spec containing:
```json
"action": { "handler": "orders.destroy", "confirm": { "title": "Delete?", "variant": "danger" } }
```
silently drops `variant: "danger"` during serde deserialization. The `collect_retired_action_variants` walk operates on `el.props`, not on `el.action`, so this case is invisible to Stage 2b.

### Single-home options

| Option | Mechanism | Stale-prop severity | Gap closed? | Duplicate surface? |
|--------|-----------|-------------------|-------------|-------------------|
| **A (recommended):** Extend catalog.rs Stage 2b | Serialize `el.action` to `Value` and run the same `collect_retired_action_variants` walk on it | Hard error (PropsInvalid) | Yes | No |
| B: Absorb into design::lint | Remove Stage 2b from catalog.rs; add `stale-props` all-intents Warning rule | Warning (downgrade) | Yes | No |
| C: Add to design::lint only for el.action gap | Keep Stage 2b + add design lint rule for el.action only | Mixed (error + warning) | Yes | Yes — two surfaces |

**Recommendation:** Option A. Extend `catalog.rs` Stage 2b to also serialize `el.action` as a `serde_json::Value` and run `collect_retired_action_variants` on that subtree. This:
- Closes the gap in the existing hard-error path (correct severity for migration hygiene)
- Keeps stale-prop detection in one place
- Requires ~5 lines of code change in catalog.rs
- design::lint gets NO stale-prop rule (no duplicate surface)

If Option B is preferred (uniform Warning treatment), Stage 2b must be removed from catalog.rs in the same commit.

[VERIFIED: catalog.rs:754-777, :879-920; action.rs:38-44 (ConfirmDialog struct has no `variant` field)]

---

## D-07/D-08: Feature Flag Reality

### Verified facts

1. `ferro-json-ui/Cargo.toml:14`: `projections = ["dep:ferro-projections", "dep:ferro-theme"]` — non-default ✓
2. `ferro-cli/Cargo.toml:44`: `ferro-json-ui = { path = "../ferro-json-ui", version = "0.2" }` — no features specified → ferro-json-ui compiles without `projections` feature in the ferro-cli build ✓
3. `ferro-cli/Cargo.toml:53-55`: ferro-cli has its OWN `projections` feature (`default = ["projections"]`) that enables `dep:ferro-projections` for ferro-cli's own use (e.g. `ai:make`, `ai:explain` commands). This is SEPARATE from ferro-json-ui's projections feature.
4. `ferro-projections/src/intent.rs:42-53`: `Intent::label()` returns exactly: `"browse"`, `"focus"`, `"collect"`, `"process"`, `"summarize"`, `"analyze"`, `"track"` for the 7 known variants.
5. No dependency cycle: `ferro-projections` does not depend on `ferro-json-ui` (it is schema-only per its CLAUDE.md). Adding a drift test gated with `#[cfg(feature = "projections")]` in ferro-json-ui uses the existing optional dep — no new dependency needed.

### D-07 consequence for implementation

The `design` module must compile without `ferro-projections`. Use:
```rust
// In design/mod.rs — no use of ferro_projections
pub const KNOWN_INTENTS: &[&str] = &[
    "browse", "focus", "collect", "process", "summarize", "analyze", "track",
];
```

The drift test is the ONLY place `ferro_projections::Intent` appears, and it is gated:
```rust
#[cfg(all(test, feature = "projections"))]
```

CI command `cargo test --all-features` enables `ferro-json-ui`'s `projections` feature → drift test runs → enforced ✓

[VERIFIED: ferro-json-ui/Cargo.toml:1-30; ferro-cli/Cargo.toml:1-60; ferro-projections/src/intent.rs:42-53]

---

## Component Prop Shapes for the 10 Rules

Each rule's check reads specific props from the flat `Spec.elements` map. Rules operate on `Value` (props are `serde_json::Value`).

### `page-header` (all intents, layout: "dashboard" | "app")

| Signal | Source | Check |
|--------|--------|-------|
| Layout gate | `spec.layout: Option<String>` | == "dashboard" or == "app" |
| PageHeader present | `el.type_name` in elements | Any element with `type_name == "PageHeader"` |
| PageHeader.title | `el.props["title"]` | Must be present (non-null string or binding) |

Note: "dashboard" is user-registered, not a built-in in `LayoutRegistry::new()` (which only includes "default", "app", "auth"). The rule checks the string value of `spec.layout` directly.

[VERIFIED: layout.rs:663-667; component.rs:945-958 (PageHeaderProps)]

### `prefer-data-table` (browse intent)

| Signal | Check |
|--------|-------|
| Raw `Table` element present | Any element with `type_name == "Table"` → Warning |
| Suggestion | Replace with `DataTable` (responsive, mobile cards) |

`TableProps` (component.rs:236): columns, data_path, row_actions, empty_message, sortable, sort_column, sort_direction.

### `list-empty-state` (browse intent)

| Signal | Check |
|--------|-------|
| DataTable without empty config | `el.type_name == "DataTable"` and `props["empty_message"]` is null AND no `EmptyState` sibling/descendant |
| MediaCardGrid without empty config | Same pattern |
| Conforming | `empty_message` present OR `EmptyState` element exists in spec |

`DataTableProps.empty_message: Option<String>` — check `el.props.get("empty_message")` is_none AND no `EmptyState` type_name anywhere in spec.

### `row-actions-grouped` (browse, process)

| Signal | Check |
|--------|-------|
| Multiple Button siblings in $each template | Elements with `type_name == "Button"` that are siblings (same parent's children) and their parent is an `$each`-templated element |
| DataTable/KanbanBoard with raw `row_actions: [Action]` shape | `DataTable.row_actions` using old `Action` type (not `DropdownMenuAction`) — check if props have handler/method/label mismatch |

Practically: scan for elements that have 2+ `Button` children where the parent is `$each`-templated → suggest `ActionGroup`. Also check `Table.row_actions` (which uses the old `Vec<Action>` type vs `DataTable`'s `Vec<DropdownMenuAction>`).

[VERIFIED: component.rs:240 (Table.row_actions: Option<Vec<Action>>), component.rs:1167 (DataTable.row_actions: Option<Vec<DropdownMenuAction>>), component.rs:1008 (ActionGroupProps.items: Vec<ActionItem>)]

### `process-kanban` (process intent)

| Signal | Check |
|--------|-------|
| No KanbanBoard | No element with `type_name == "KanbanBoard"` → Warning |
| KanbanBoard.columns count | `props["columns"]` is array (non-empty) — columns carry count badge via `KanbanColumnProps.count` |

[VERIFIED: component.rs:1250-1282 (KanbanBoardProps)]

### `create-separate-page` (collect intent)

| Signal | Check |
|--------|-------|
| Modal containing a Form | Any element with `type_name == "Modal"` that has a child element with `type_name == "Form"` (via the children/elements traversal) |

`ModalProps` (component.rs:426) has `id: String`, `title: String`, `footer: Vec<String>` (as element IDs). Check if Modal's children include a Form-type element.

[VERIFIED: component.rs:426-428; spec.rs:104-118 (Element.children: Vec<String>)]

### `breadcrumb-on-subpages` (collect, focus intents)

| Signal | Check |
|--------|-------|
| No Breadcrumb | No element with `type_name == "Breadcrumb"` in spec → Warning |
| PageHeader.breadcrumb present | Check `PageHeaderProps.breadcrumb: Vec<BreadcrumbItem>` — if PageHeader exists and has non-empty breadcrumb, considered conforming too |

`BreadcrumbProps.items: Vec<BreadcrumbItem>` (component.rs:587-588).

[VERIFIED: component.rs:579-596]

### `form-default-values` (collect intent)

| Signal | Check |
|--------|-------|
| Edit-form detection | Any form field (`Input`, `Select`, `Textarea`, `RichTextEditor`) has `props["default_value"]` as a `$data` object (`{"$data": "..."}`) |
| Sibling fields without default_value | Other form fields in same spec lacking `default_value` or `data_path` |
| Pure create form | No `$data` binding on any `default_value` → no findings |

`InputProps.default_value: Option<String>` (component.rs:339), `SelectProps.default_value: Option<String>` (component.rs:401). In JSON, a `$data` binding would appear as `{"default_value": {"$data": "/user/email"}}`.

[VERIFIED: component.rs:339, 401; anchor spec §3 rule text: "When any form field binds default_value via a $data path..."]

### `destructive-confirmation` (all intents)

| Signal | Check |
|--------|-------|
| Destructive-styled action | Element-level `el.action` where `el.action.confirm` is None AND the action's intent is destructive |
| Destructive Button | `el.type_name == "Button"` with `props["variant"] == "destructive"` AND `el.action.confirm` is None |
| ActionGroup item | Props-embedded `ActionItem.destructive == true` AND `item.action.confirm` is None |
| DataTable/Kanban row_actions | `DropdownMenuAction` with `destructive: true` and no `confirm` |

Detection of "styled destructive": `variant: "destructive"` on Button, `ActionItem.destructive: true` (component.rs:985), or `DropdownMenuAction` destructive flag. Conformance: `Action.confirm: Some(ConfirmDialog)` present.

[VERIFIED: action.rs:148 (Action.confirm: Option<ConfirmDialog>); component.rs:985 (ActionItem.destructive: bool); CONTEXT.md D-15]

### `card-actions-in-menu` (process intent)

| Signal | Check |
|--------|-------|
| KanbanBoard with loose Button children | Elements that are siblings/children in kanban card structure using Button instead of ActionGroup |
| Correct order | First item should be "detail" action; destructive items last |
| All inside ActionGroup/DropdownMenu | row_actions use DropdownMenuAction (the correct channel) |

Since kanban card content is typically data-bound (items_path), this rule checks the spec structure for static kanban specs or warns when a KanbanBoard has children with loose Button elements that look like per-card actions.

[VERIFIED: component.rs:1250 (KanbanBoardProps); component.rs:1219-1234 (KanbanColumnProps.children for static specs)]

### Additional all-intents rule: stale-prop detection (D-16 — subject to planner decision)

If Option B (move to design::lint) is chosen, this rule covers:
- Element-level retired prop names: `Card.variant`, `Badge.variant`, `Alert.variant`, `Toast.variant`, `ActionCard.variant`, `MediaCardGrid.badge_variant_key`
- Element-level `action.confirm.variant`, `action.on_success.notify.variant`
- Severity: Warning (downgrade from current hard error)

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recursive JSON file discovery | Manual `read_dir` recursion | `walkdir::WalkDir` | Already in ferro-cli deps; handles symlinks and errors |
| Styled terminal output | ANSI escape codes | `console::style()` | Already in ferro-cli; matches every other command's style |
| JSON serialization for `--json` | Manual formatting | `serde_json::to_string_pretty()` | Pattern from validate_contracts.rs:899 |
| Component presence check across elements | Tree traversal | Flat `HashMap<String, Element>` iteration | Spec elements are already flat — just scan the values |

---

## Sample App Views Analysis (for D-17 planning)

### Current state of all three views

| File | Layout | Design field | Predicted findings WITHOUT design field |
|------|--------|--------------|----------------------------------------|
| `login.json` | `"auth"` | absent | `declare-intent` info (inference → collect from Form); NO page-header (auth not dashboard-family); NO form-default-values (no `$data` default_value bindings) |
| `login_confirm.json` | `"auth"` | absent | `declare-intent` info (inference → collect); NO other findings |
| `pagamenti.json` | `"dashboard"` | absent | `declare-intent` info; `page-header` WARNING (dashboard layout, no PageHeader element); `list-empty-state` conforming (DataTable has `empty_message`) |

### What must change for D-17 zero-findings gate

1. All three views must add `"design": {"intent": "<inferred>"}` field to declare intent.
2. `pagamenti.json` must either:
   - Add a `PageHeader` element (requires adding it to root's children) — makes the spec more complete
   - Or add `"allow": ["page-header"]` to its design field — keeps spec as-is

The planner must decide. Research observation: `pagamenti.json` is a payments summary page — a `PageHeader` with title "Pagamenti" would be architecturally correct and matches the CLAUDE.md dashboard pattern ("PageHeader on every page"). Adding it is the conforming choice; `allow` is acceptable if the page is intentionally header-less.

[VERIFIED: app/src/views/login.json, login_confirm.json, pagamenti.json — direct file reads]

---

## Common Pitfalls

### Pitfall 1: `DesignMeta` must move to `spec.rs` or be imported carefully

`DesignMeta` is a field on `Spec`. If defined in `design/types.rs`, `spec.rs` must import it — creating a module dependency. The cleanest approach: define `DesignMeta` in `spec.rs` alongside `Spec` (matching the precedent for `DataRef`, `TitleBinding`), and re-export it from `design/` with `pub use crate::spec::DesignMeta`. This avoids a circular module dependency between `spec` and `design`.

### Pitfall 2: `lint()` receives a `Spec` that may not have been catalog-validated

The planner must not assume the spec was valid before lint runs. Rules must be defensive: `props.get("x")` returns `None` gracefully; `el.props.as_object()` may be `None` if `props` is `Value::Null`. Use `.and_then()` chains, not `.unwrap()`.

### Pitfall 3: `app` dev-dependency for D-17 test requires explicit addition

The `app` crate depends on `ferro = { path = "../framework" }` which transitively includes `ferro-json-ui`, but the Rust module system requires an explicit `ferro-json-ui = { path = "../ferro-json-ui" }` in `[dev-dependencies]` to use `ferro_json_ui::design::lint` in tests. Without this, the test will fail to compile with "use of undeclared crate or module".

[VERIFIED: app/Cargo.toml — ferro-json-ui is NOT listed in [dev-dependencies]; ferro-projections IS a precedent for this pattern]

### Pitfall 4: `"dashboard"` is user-registered, not a built-in

`LayoutRegistry::new()` in `layout.rs:663` inserts only `"default"`, `"app"`, and `"auth"`. The `"dashboard"` name is registered by the app at startup. The `page-header` rule checks `spec.layout` as a plain string — DO NOT check registry membership (lint is pure, no registry access). The rule fires on `"dashboard"` or `"app"` by string equality.

[VERIFIED: layout.rs:664-667]

### Pitfall 5: `cargo doc` requirement for the new module

The CI docs build (`cargo doc` with `-D warnings`) catches missing or malformed rustdoc. Every public item in `ferro_json_ui::design` needs a `///` doc comment: `DesignMeta`, `DesignRule`, `Finding`, `Severity`, `lint()`, `rules()`, `KNOWN_INTENTS`. The module itself needs a `//!` comment. Missing docs = CI failure.

### Pitfall 6: `#[cfg(feature = "projections")]` test isolation

The drift test in `ferro-json-ui` will NOT compile (missing `ferro_projections` import) when the `projections` feature is off. Ensure the entire test module is `#[cfg(all(test, feature = "projections"))]`, not just the single test function. Without this, `cargo test` (without `--all-features`) will fail to compile.

---

## Runtime State Inventory

Not applicable — this is a greenfield addition to ferro-json-ui and ferro-cli. No rename/refactor/migration phase.

---

## Environment Availability

No external dependencies beyond the Rust toolchain.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable | Build | ✓ | 1.88.0 (from `rustfmt` version in REVIEW-FIX.md) | — |
| walkdir | CLI file discovery | ✓ | already in ferro-cli/Cargo.toml | — |
| schemars | Finding/Severity derive | ✓ | v1 (workspace) | — |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (no external test runner) |
| Config file | none — workspace Cargo.toml |
| Quick run command | `cargo test -p ferro-json-ui design` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DS-05 | 10 rules each have violating + conforming spec | unit | `cargo test -p ferro-json-ui` | ❌ Wave 0 (new module) |
| DS-05 | Intent inference branches (DataTable→browse, KanbanBoard→process, Form→collect, StatCard→summarize) | unit | `cargo test -p ferro-json-ui design::infer` | ❌ Wave 0 |
| DS-05 | D-08 drift: KNOWN_INTENTS == Intent::label() for 7 variants | unit (feature-gated) | `cargo test -p ferro-json-ui --all-features design::drift` | ❌ Wave 0 |
| DS-05 | D-16 gap (if Option A): element-level action confirm.variant caught by catalog Stage 2b | unit | `cargo test -p ferro-json-ui` (catalog tests) | ❌ Wave 0 (extend catalog tests) |
| DS-05 | Unknown allow ids → warning finding | unit | `cargo test -p ferro-json-ui design` | ❌ Wave 0 |
| DS-06 | App views lint clean (D-17 gate) | integration | `cargo test -p app design_lint` | ❌ Wave 0 |
| DS-06 | CLI `--deny` exits non-zero with warning findings | unit | `cargo test -p ferro-cli design_lint` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui` (635 + new design tests)
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features && cargo doc --no-deps 2>&1 | grep -v "^$"`

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/design/mod.rs` — module entry, `lint()`, `rules()`, `KNOWN_INTENTS`
- [ ] `ferro-json-ui/src/design/types.rs` — `DesignMeta`, `DesignRule`, `Finding`, `Severity`
- [ ] `ferro-json-ui/src/design/rules.rs` — static registry of 10 rules
- [ ] `ferro-json-ui/src/design/infer.rs` — intent inference heuristics
- [ ] `ferro-cli/src/commands/design_lint.rs` — command impl
- [ ] `app/src/tests/design_lint.rs` — D-17 lint-clean gate

---

## Security Domain

The design lint is a pure diagnostic engine operating on already-parsed spec data. No user input handling, no persistence, no authentication. ASVS categories not applicable.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | No | Spec parse is handled by existing `Spec::from_json` (never panics on arbitrary input per spec.rs:278 contract) |
| All others | No | Pure in-process diagnostic computation |

---

## Open Questions (RESOLVED)

1. **D-16 single-home: Option A vs Option B**
   - What we know: catalog.rs Stage 2b exists as a hard error; the el.action gap is small and fixable with ~5 lines
   - What's unclear: planner must decide severity (hard error vs warning) for migration hygiene
   - RESOLVED: Option A (extend Stage 2b) — keeps the migration-era lint in the validation layer where it belongs; design::lint focuses on composition patterns

2. **`pagamenti.json` page-header conformance**
   - What we know: layout is "dashboard", no PageHeader element exists
   - What's unclear: should the spec gain a PageHeader element (better), or should the design field allow the rule?
   - RESOLVED: add a PageHeader element; the payments list page architecturally wants a title header. The test asserts zero findings — PageHeader is the path to zero without allow-listing.

3. **`row-actions-grouped` exact heuristic**
   - What we know: the rule targets loose Button siblings in row-context positions
   - What's unclear: detecting "row-context" reliably from the flat element map without runtime data
   - RESOLVED: the simplest heuristic is checking for multiple `Button` elements that are direct children of an `$each`-templated element — any such pattern suggests row action buttons that should be in an ActionGroup

4. **`card-actions-in-menu` order enforcement**
   - What we know: the rule says "detail action first, destructive actions last, all inside ActionGroup"
   - What's unclear: is order-checking feasible on raw specs where row_actions order is defined in props arrays?
   - RESOLVED: check that `KanbanBoard.row_actions` (DropdownMenuAction array) has any destructive action NOT in last position → Warning; first item should have "detail"/"view" in label

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `--json` output shape is a flat array with `file` field per finding (not grouped by file object) | Pattern 5, Standard Stack | gestiscilo CI may need to adjust JSON parsing; fix before Phase 253 publish |
| A2 | `pagamenti.json` intent inference would produce `browse` or `summarize` based on DataTable + StatCard presence | Sample App Views | Inference may produce unexpected intent; no correctness impact as long as design.intent is declared |

**All other claims in this research were verified against the live codebase.**

---

## Sources

### Primary (HIGH confidence — verified against codebase)

- `ferro-json-ui/src/spec.rs:73` — `Spec` struct, optional field serde pattern
- `ferro-json-ui/src/action.rs:38,148` — `ConfirmDialog`, `Action.confirm`
- `ferro-json-ui/src/catalog.rs:754,883,898` — WR-01 Stage 2b, `RETIRED_PROPS`, `collect_retired_action_variants`
- `ferro-json-ui/Cargo.toml:14` — `projections` feature non-default
- `ferro-projections/src/intent.rs:42-53` — `Intent::label()` verified strings
- `ferro-cli/src/main.rs:345,358,534-808` — clap colon-command pattern, dispatch structure
- `ferro-cli/src/commands/mod.rs` — command registration
- `ferro-cli/src/commands/validate_contracts.rs:866,890-924` — `--json` + serde_json pattern
- `ferro-cli/Cargo.toml:44,53-55` — ferro-json-ui dep (no features), ferro-cli projections feature
- `ferro-json-ui/src/component.rs:240,1008,1163,1250,945,579,913` — DataTable, ActionGroup, Kanban, PageHeader, Breadcrumb, EmptyState props
- `ferro-json-ui/src/layout.rs:663-667` — LayoutRegistry built-ins ("default","app","auth"), dashboard is user-registered
- `app/Cargo.toml:39-49` — dev-dependencies, ferro-projections precedent
- `app/src/tests/mod.rs` — test registration pattern
- `app/src/views/*.json` — all three sample view files (content verified)
- `docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md` — anchor spec §3,4,7,8

### Secondary (MEDIUM confidence — verified via grep + file scan)

- RETIRED_PROPS coverage verified via `grep -n` against catalog.rs
- Layout registry built-ins verified via grep of layout.rs

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all existing deps confirmed in Cargo.toml files
- Architecture: HIGH — module structure follows established ferro-json-ui patterns
- WR-01 analysis: HIGH — exact line numbers verified
- Feature flag analysis: HIGH — verified against both Cargo.toml files
- App views predictions: HIGH — specs read directly; rule logic traced against prop structs
- Rule-to-props mapping: HIGH — all prop structs read from component.rs

**Research date:** 2026-07-03
**Valid until:** 2026-08-03 (stable domain; no fast-moving dependencies)
