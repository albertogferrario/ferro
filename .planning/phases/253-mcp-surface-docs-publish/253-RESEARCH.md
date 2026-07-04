# Phase 253: MCP surface + docs + publish - Research

**Researched:** 2026-07-04
**Domain:** ferro-mcp tool authoring, JSON-UI catalog/generation-context extension, mdBook documentation, crates.io workspace publish
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**design_lint MCP tool:**
- D-01: Input = `spec_json` (inline JSON string) OR `path` (single spec file) — exactly one required. Directory sweeps are the CLI's job; the MCP tool is the in-session author→validate loop for one spec.
- D-02: Output reuses the `Finding`/`Severity` serialization from 252 D-11 — the same shape the CLI `--json` emits. No MCP-only envelope invention.
- D-03: In-process call to `ferro_json_ui::design::lint`. Registration follows `json_ui_validate_spec` / `service.rs:1405` pattern.
- D-04: Lint-only — does NOT run catalog validation. A spec that fails `Spec` parse returns a parse diagnostic inside the findings envelope (WR-03 posture), never a tool error.

**json_ui_catalog + generation_context extensions:**
- D-05: Canonical variant vocabulary derived from canonical enums (Variant/Tone/Size/CardAppearance from Phase 251); per-component guidance derived from `design::rules()` metadata. All new catalog fields additive (backward-compatible). Every static supplement drift-guarded.
- D-06: `generation_context` design-system summary contains: (a) 30-slot token vocabulary sourced from ferro-theme constants, (b) per-intent pattern expectations from rule registry (id + title + rationale grouped by intent), (c) canonical variant/tone/size value lists. Compact — ids and one-liners, pointer to docs for depth.
- D-07: Component count stays 47. The ferro-mcp documented mirror assertion (`json_ui_catalog.rs:294`) is untouched.

**docs/src/design-system/ chapter:**
- D-08: Five pages + SUMMARY.md section: `principles.md`, `tokens.md`, `variants.md`, `patterns.md`, `linting.md`. Cross-link `features/themes.md` (token authoring recipe) and `json-ui/components.md` (Phase 251 D-17 migration table). Do not duplicate.
- D-09: patterns.md is hand-written prose but drift-guarded: a test asserts every rule id from `design::rules()` appears in `patterns.md` and every documented rule id exists in the registry.
- D-10: Neutral product documentation voice. No "v2 vs legacy" framing.

**Publish:**
- D-11: Single publish at phase end. Local master at 0.2.83 (unpushed); crates.io at 0.2.80. Land all 253 code → CI-exact gate → ONE final workspace bump → push via gh HTTPS helper → CI publishes.
- D-12: CI-exact gate: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, plus the Docs build (`cargo doc -D warnings`) and cargo-deny awareness.
- D-13: ferro-payments (0.1.3) untouched. No new crates. No publish.yml wave changes.
- D-14: Operator-gated publish. Pre-publish UAT checklist includes 252's human CLI output check and 251's suggested pixel-level visual pass.
- D-15: Fold 252's deferred info-findings as pre-publish cleanup: IN-01 remove dead `"Textarea"` from `FIELD_TYPES` (`design/rules.rs:298`); IN-02 fix misleading "No findings — all specs are clean" when zero files were linted (`commands/design_lint.rs`).
- D-16: Publishing unblocks gestiscilo Phase 232. Cross-repo handoff is a brief only.

### Claude's Discretion

- Exact field names and struct layout for new catalog / generation_context fields; whether per-intent expectations embed rationale verbatim or trimmed.
- Whether `design_lint` returns rule metadata (title/rationale) inline per finding or only ids (consistency with CLI `--json` wins ties).
- Doc page ordering, titles, and intra-chapter navigation within the five-section requirement.
- Whether the docs drift test lives in ferro-json-ui or a workspace-level test.
- The final version number (next patch after whatever master carries when the publish commit is cut).

### Deferred Ideas (OUT OF SCOPE)

- gestiscilo Phase 232 reference-case adoption.
- `/gsd-complete-milestone` archival.
- CSS-hygiene lint (dead utilities in generated ferro-base.css).
- OQ-3 `dot_colors` raw-Tailwind rule.
- v16.4 Work Distribution phases 244-249.
- Pre-existing flaky `serve.rs` PGID test (documented in 252's deferred-items.md).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DS-07 | ferro-mcp gains a `design_lint` tool (inline spec or path); `json_ui_catalog` extends with canonical variant vocabulary and per-component design guidance; `generation_context` gains a design-system summary | Tool registration pattern confirmed via `json_ui_validate_spec` analog at service.rs:1404. Catalog extension: additive fields to `JsonUiCatalog` struct. generation_context extension: additive `design_system` field to `GenerationContext`. All source data verified in-codebase. |
| DS-08 | New `docs/src/design-system/` chapter (principles, token v2 reference, variant vocabulary, pattern catalog, linting guide); single crates.io publish | SUMMARY.md insertion point confirmed (after JSON-UI section, lines 63-73). Cross-link targets verified (`features/themes.md:55-132`, `json-ui/components.md:42-99`). Publish mechanics confirmed: workspace at 0.2.83, CI-exact gate documented, no wave changes needed. |
</phase_requirements>

---

## Summary

Phase 253 is a derivation and surface-exposure phase: all the raw material ships from Phases 250–252; this phase derives three agent-facing outputs from it (the `design_lint` MCP tool, catalog/generation-context extensions, and the docs chapter) and issues the single milestone publish.

The `design_lint` tool is a direct in-process call to `ferro_json_ui::design::lint` — `ferro-mcp` already depends on `ferro-json-ui` with the `projections` feature (Cargo.toml:24), so the function, type registry, and `Finding`/`Severity` serialization are all available. The tool param struct follows the established `#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]` pattern; both `spec_json` and `path` are `Option<String>` with runtime XOR enforcement.

The catalog and generation-context extensions are additive field groups on existing structs (`JsonUiCatalog`, `GenerationContext`) — the struct shape is backward-compatible and existing tests are extended, not replaced. The generation-context token vocabulary requires adding `ferro-theme` as a dependency to `ferro-mcp` (for count drift-guard against `ALL_TOKENS`; not currently in ferro-mcp's Cargo.toml).

The docs chapter is five mdBook markdown files plus a SUMMARY.md insert. Two pre-publish cleanup items (IN-01, IN-02) are one-liners. The publish commit is the last action: bump workspace version past 0.2.83, push master via gh HTTPS helper, CI publishes.

**Primary recommendation:** Complete in four distinct waves — (1) design_lint MCP tool, (2) catalog + generation_context extensions with drift guards, (3) docs chapter + drift test, (4) pre-publish cleanup (IN-01/IN-02) + CI-exact gate + publish.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| design_lint MCP tool | API / Backend (ferro-mcp) | ferro-json-ui (source) | Pure in-process function call; no rendering, no I/O except optional file read for `path` input |
| json_ui_catalog design extension | API / Backend (ferro-mcp) | ferro-json-ui (enum source) | Catalog data derived from canonical enums and rule registry; serialized and returned to MCP caller |
| generation_context design-system summary | API / Backend (ferro-mcp) | ferro-theme (token constants source), ferro-json-ui (rule registry source) | Static data assembled in-process from constants; no runtime I/O |
| docs/src/design-system/ chapter | Static / Docs | — | mdBook markdown pages; no runtime component; D-09 drift test enforces registry sync |
| crates.io publish | CI/CD | — | GitHub Actions publish.yml wave execution triggered by Cargo.toml version bump |

---

## Standard Stack

### Core (all existing — no new crates except one new dependency)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rmcp` | 0.12 | MCP server tool registration (`#[tool]`, `Parameters<T>`) | Established ferro-mcp server framework |
| `serde` + `serde_json` | 1 | Serialization of all tool responses | Universal in the workspace |
| `schemars` | 1 | JSON Schema derive for param structs | Required by `Parameters<T>` deserialization |
| `ferro_json_ui::design` | workspace | `lint()`, `rules()`, `Finding`, `Severity`, `KNOWN_INTENTS` | Ships from Phase 252; already available via `projections` feature |
| `ferro_theme::token` | workspace | `ALL_TOKENS`, token constants for count drift guard | Token vocabulary source |
| `walkdir` | 2 | File discovery for `path` input mode of design_lint | Already in ferro-mcp/Cargo.toml |

### New dependency (one addition)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `ferro-theme` | workspace | Access to `ALL_TOKENS.len()` for count drift guard in generation_context | Add to ferro-mcp/Cargo.toml: `ferro-theme = { path = "../ferro-theme", version = "0.2" }` |

`ferro-theme` is not currently in ferro-mcp/Cargo.toml. The token descriptions can be maintained as a static array in `generation_context.rs` (since descriptions are stable and not in `ALL_TOKENS`); the count drift guard asserts `DESIGN_TOKEN_DESCRIPTIONS.len() == ferro_theme::token::ALL_TOKENS.len()`.

**No new crates** — D-13 and the anchor spec §8 non-goal are both explicit on this.

**Installation:**
```bash
# Add to ferro-mcp/Cargo.toml [dependencies]:
ferro-theme = { path = "../ferro-theme", version = "0.2" }
```

**Version verification:** [VERIFIED: Cargo.toml:24] `ferro-json-ui` dependency already present with `features = ["projections"]`. [VERIFIED: ferro-theme/src/token.rs] `ALL_TOKENS` constant contains 30 entries.

---

## Architecture Patterns

### System Architecture Diagram

```
Agent session
    │
    ├── generation_context tool ─────→ GenerationContext { ..., design_system }
    │                                          ↑
    │                                 ferro_json_ui::design::rules()
    │                                 ferro_theme::token::ALL_TOKENS
    │                                 Canonical enum values (Variant/Tone/Size)
    │
    ├── json_ui_catalog tool ─────────→ JsonUiCatalog { ..., design_system }
    │                                          ↑
    │                                 global_catalog() component schemas
    │                                 ferro_json_ui::design::rules()
    │                                 Canonical enum constants
    │
    ├── design_lint tool
    │       │
    │       ├── spec_json input ────→ Spec::from_json() ─→ lint(&spec) ─→ Vec<Finding> as JSON
    │       └── path input ─────────→ fs::read_to_string() ─→ same path
    │                                      ↑ parse error → parse-diagnostic Finding
    │
    └── docs/src/design-system/ ─────→ patterns.md drift test
                                              ↑
                                       design::rules() rule ids
```

### Recommended Project Structure (additions only)

```
ferro-mcp/src/
├── tools/
│   └── design_lint.rs          # new tool: execute(spec_json, path) -> Vec<FileFinding>
├── service.rs                  # add DesignLintParams struct + #[tool] method

ferro-json-ui/src/
└── design/                     # unchanged — Phase 252
    └── rules.rs                # IN-01 fix: remove "Textarea" from FIELD_TYPES

ferro-cli/src/commands/
└── design_lint.rs              # IN-02 fix: "No findings" only when files were linted

docs/src/
├── SUMMARY.md                  # add "# Design System" chapter block
└── design-system/              # new directory
    ├── principles.md
    ├── tokens.md
    ├── variants.md
    ├── patterns.md             # drift-guarded against design::rules() ids
    └── linting.md
```

### Pattern 1: MCP Tool Registration

Tool registration pattern verified from `json_ui_validate_spec` (the closest analog):

```rust
// In ferro-mcp/src/service.rs — param struct (all tool params follow this pattern):
// [VERIFIED: service.rs:246-250]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DesignLintParams {
    /// Inline JSON-UI v2 spec string to lint. Provide either this or `path`, not both.
    pub spec_json: Option<String>,
    /// Path to a single JSON-UI spec file to lint. Provide either this or `spec_json`, not both.
    pub path: Option<String>,
}

// In ferro-mcp/src/service.rs — tool method on the FerroMcpService impl:
// [VERIFIED: service.rs:1403-1424 pattern]
#[tool(
    name = "design_lint",
    description = "..."
)]
pub async fn design_lint(&self, params: Parameters<DesignLintParams>) -> String {
    let result = tools::design_lint::execute(
        params.0.spec_json.as_deref(),
        params.0.path.as_deref(),
    );
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "[]".to_string())
}
```

The XOR constraint ("exactly one required") is enforced at runtime inside `execute()` by checking `(spec_json.is_some()) ^ (path.is_some())` and returning a single error-level finding if violated. This mirrors how CLI validation errors are surfaced as findings (WR-03 posture, D-04).

### Pattern 2: design_lint Tool Execute Function

```rust
// ferro-mcp/src/tools/design_lint.rs
// [VERIFIED: CLI shape from design_lint.rs:22-30, FileFinding is the stable --json contract]
use ferro_json_ui::design::{lint, Finding, Severity};
use ferro_json_ui::spec::{Spec, SCHEMA_VERSION};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FileFinding {
    pub file: String,           // "<inline>" for spec_json input
    #[serde(flatten)]
    pub finding: Finding,       // rule, element_id, severity, message, suggestion
}

pub fn execute(spec_json: Option<&str>, path: Option<&str>) -> Vec<FileFinding> {
    match (spec_json, path) {
        (Some(json), None) => lint_string("<inline>", json),
        (None, Some(p)) => {
            match std::fs::read_to_string(p) {
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
            }
        }
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
```

The `lint_string` helper mirrors `ferro_cli::commands::design_lint::lint_content` exactly (checks SCHEMA_VERSION marker, handles parse error as Warning finding, calls `ferro_json_ui::design::lint`).

### Pattern 3: Additive Catalog Extension

```rust
// ferro-mcp/src/tools/json_ui_catalog.rs — additive field on existing struct
// [VERIFIED: existing JsonUiCatalog struct at json_ui_catalog.rs:12-24]
#[derive(Debug, Serialize)]
pub struct JsonUiCatalog {
    pub components: Vec<CatalogComponent>,
    pub plugin_components: Vec<CatalogComponent>,
    pub builder_api: String,
    pub action_api: String,
    pub json_schema: serde_json::Value,
    pub component_schemas: std::collections::HashMap<String, serde_json::Value>,
    pub directives: Vec<DirectiveInfo>,
    // New additive fields for D-05:
    pub design_system: DesignVocabulary,
}

#[derive(Debug, Serialize)]
pub struct DesignVocabulary {
    /// Canonical variant values — visual weight of interactive elements.
    pub variant_values: Vec<&'static str>,
    /// Canonical tone values — semantic status color for stateful display components.
    pub tone_values: Vec<&'static str>,
    /// Canonical size values.
    pub size_values: Vec<&'static str>,
    /// Design rules applicable to each component, keyed by component type name.
    pub component_guidance: std::collections::HashMap<String, Vec<DesignRuleRef>>,
}

#[derive(Debug, Serialize)]
pub struct DesignRuleRef {
    pub id: &'static str,
    pub title: &'static str,
    pub rationale: &'static str,
}
```

The `variant_values`/`tone_values`/`size_values` come from the `CANONICAL_VARIANT/CANONICAL_TONE/CANONICAL_SIZE` constants already in `ferro-json-ui/src/catalog.rs:1229-1231` (or can be derived from the canonical enum serde representations).

`component_guidance` is derived by scanning each rule's `title` + `rationale` text for component type name occurrences (a text scan is sufficient; adding a `components` field to `DesignRule` is also viable but modifies Phase 252's shipped type).

### Pattern 4: generation_context Design-System Summary

```rust
// ferro-mcp/src/tools/generation_context.rs — additive field
// [VERIFIED: existing GenerationContext struct at generation_context.rs:7-13]
#[derive(Debug, Serialize)]
pub struct GenerationContext {
    pub naming_conventions: NamingConventions,
    pub file_structure: FileStructure,
    pub common_patterns: CommonPatterns,
    pub avoid: Vec<String>,
    pub imports: ImportTemplates,
    // New field for D-06:
    pub design_system: DesignSystemSummary,
}

#[derive(Debug, Serialize)]
pub struct DesignSystemSummary {
    /// 30-slot semantic token vocabulary. Each entry: (CSS var name, one-line purpose).
    pub tokens: Vec<TokenInfo>,
    /// Per-intent pattern expectations derived from design::rules() grouped by intent.
    pub intent_patterns: std::collections::HashMap<String, Vec<PatternExpectation>>,
    /// Canonical variant vocabulary: variant/tone/size value lists.
    pub canonical_variants: CanonicalVariants,
    /// Pointer to full documentation.
    pub docs: &'static str,
}

#[derive(Debug, Serialize)]
pub struct TokenInfo {
    pub name: &'static str,    // e.g. "--color-primary"
    pub purpose: &'static str, // e.g. "Primary action color (buttons, links, highlights)"
}

#[derive(Debug, Serialize)]
pub struct PatternExpectation {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub rationale: &'static str,
}

#[derive(Debug, Serialize)]
pub struct CanonicalVariants {
    pub variant: &'static [&'static str],
    pub tone: &'static [&'static str],
    pub size: &'static [&'static str],
}
```

The 30 token entries are maintained as a static `DESIGN_TOKEN_DESCRIPTIONS: &[(&str, &str)]` in `generation_context.rs`. A count drift guard asserts `DESIGN_TOKEN_DESCRIPTIONS.len() == ferro_theme::token::ALL_TOKENS.len()` (requiring the new `ferro-theme` dependency).

### Pattern 5: D-09 Docs Drift Test

The drift test reads `docs/src/design-system/patterns.md` at test time using `CARGO_MANIFEST_DIR` to locate the file:

```rust
// In ferro-json-ui/src/design/mod.rs or a tests/ file
#[cfg(test)]
mod docs_drift_tests {
    use super::rules;

    #[test]
    fn patterns_md_covers_all_rule_ids() {
        // CARGO_MANIFEST_DIR for ferro-json-ui is ferro-json-ui/
        // patterns.md is at ../../docs/src/design-system/patterns.md
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let patterns_path = std::path::Path::new(&manifest_dir)
            .join("../docs/src/design-system/patterns.md");
        let content = std::fs::read_to_string(&patterns_path)
            .expect("patterns.md must exist (D-09)");

        for rule in rules() {
            assert!(
                content.contains(rule.id),
                "patterns.md is missing rule id `{}` (D-09 drift guard)",
                rule.id
            );
        }
    }
}
```

This is a new class of test in this codebase (file-reading in a unit test). Home: `ferro-json-ui/src/design/mod.rs` drift_tests section — matching the D-08 drift test already there at line 284.

### Pattern 6: docs/src/SUMMARY.md Addition

Insert a new chapter block after the JSON-UI section (currently ends at line 73):

```markdown
# Design System

- [Principles](design-system/principles.md)
- [Token Reference](design-system/tokens.md)
- [Variant Vocabulary](design-system/variants.md)
- [Pattern Catalog](design-system/patterns.md)
- [Linting Guide](design-system/linting.md)
```

Cross-link targets are:
- `features/themes.md` — token authoring recipe, v2 table (lines 55-132), dark mode block, `ThemeMiddleware` docs
- `json-ui/components.md` — canonical enum section (line 42), migration table (line 72-99)

### Anti-Patterns to Avoid

- **Do not duplicate the migration table**: `json-ui/components.md:72-99` owns the full component-rename migration table. `variants.md` must link to it, never copy it.
- **Do not duplicate the token authoring recipe**: `features/themes.md` owns the `tokens.css` authoring guide. `tokens.md` must link to it.
- **Do not invent a new FileFinding shape for MCP**: The `FileFinding { file, #[serde(flatten)] finding }` from the CLI is the stable contract (252 D-11). The MCP tool reuses it verbatim.
- **Do not return a tool-level error for spec-parse failures**: D-04 mandates that parse failures return a finding inside the findings envelope, not an MCP protocol error.
- **Do not skip the `test_generation_context_has_all_sections` update**: That test explicitly checks each field name (`context.naming_conventions`, etc.) — adding `design_system` without updating the assertion will fail.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Spec parsing in design_lint | Custom JSON parser | `Spec::from_json` (ferro-json-ui) | Already handles SCHEMA_VERSION check, SpecError variants, all edge cases |
| Design rule execution | New rule dispatcher | `ferro_json_ui::design::lint(&spec)` | Pure function, 10 rules, already tested in Phase 252 |
| Canonical enum values | New enum declaration | `CANONICAL_VARIANT/CANONICAL_TONE/CANONICAL_SIZE` in `ferro-json-ui/src/catalog.rs:1229-1231` | Phase 251 drift-guarded source; importing ensures drift cannot diverge |
| FileFinding shape | New MCP-specific envelope | `FileFinding { file, #[serde(flatten)] finding }` from `ferro-cli::commands::design_lint` | Reuse for identical CLI/MCP contract (252 D-11) |
| Token count check | Hard-coded `30` assertion | `ferro_theme::token::ALL_TOKENS.len()` | Relational drift guard; survives any future ALL_TOKENS extension |

**Key insight:** This entire phase is a derivation exercise — every surface is derived from Phase 252 outputs. The risk is hand-duplication that creates a second source of truth; every derived surface needs a structural drift guard.

---

## Runtime State Inventory

Not applicable — this is a greenfield surface-exposure and documentation phase. No rename/refactor/migration involved.

---

## Common Pitfalls

### Pitfall 1: Stale "39 built-in components" in json_ui_catalog service.rs description
**What goes wrong:** `service.rs:1303` still reads "39 built-in components" in the `#[tool]` description string. CI passes because it's a string, not a count assertion.
**Why it happens:** The count was updated in ferro-json-ui catalog but the tool description string in service.rs is a hand-written literal, not derived.
**How to avoid:** When updating the `json_ui_catalog` tool for D-05, update the description string from "39" to "47" in the same commit.
**Warning signs:** Grep `"39 built-in"` in service.rs returns a hit.

### Pitfall 2: IN-01 fix breaks `form-default-values` rule scope
**What goes wrong:** Removing `"Textarea"` from `FIELD_TYPES` in `rules.rs:298` is straightforward. BUT `"RichTextEditor"` is a plugin component — the rule must still recognize it when present.
**Why it happens:** The fix removes a non-existent builtin but must preserve the plugin component reference.
**How to avoid:** After removing `"Textarea"`, verify `FIELD_TYPES` = `["Input", "Select", "RichTextEditor"]`. Run `cargo test -p ferro-json-ui design` — all 46 rule tests must still pass.
**Warning signs:** `form-default-values` tests fail or warn on RichTextEditor specs.

### Pitfall 3: generation_context test fails on new field
**What goes wrong:** `test_generation_context_has_all_sections` in `generation_context.rs:181` enumerates every field. Adding `design_system` without updating the test causes `cargo test --all-features` to fail.
**Why it happens:** The test is structured as individual field assertions, not a serialization smoke test.
**How to avoid:** Add `assert!(!context.design_system.tokens.is_empty())` (and similar) to the test in the same commit as the field addition.
**Warning signs:** `cargo test -p ferro-mcp generation_context` fails.

### Pitfall 4: D-09 drift test path is wrong in CI
**What goes wrong:** The drift test uses `CARGO_MANIFEST_DIR` to locate `patterns.md`. From `ferro-json-ui/`, the relative path is `../docs/src/design-system/patterns.md`. In CI, the workspace root is the checkout directory — this works correctly. But the test file must be committed BEFORE the drift test is added; otherwise the test panics on the `expect("patterns.md must exist")`.
**Why it happens:** Wave ordering: write patterns.md (Wave 3 step 1) before the drift test (Wave 3 step 2).
**How to avoid:** Commit patterns.md before adding the drift test, or add both in the same commit with Wave 3.

### Pitfall 5: cargo doc -D warnings fails on new public types
**What goes wrong:** New public types in ferro-mcp (`DesignLintParams`, `DesignVocabulary`, etc.) without `///` doc comments fail the `cargo doc -D warnings` CI step.
**Why it happens:** CI's matrix is wider than the local three-command gate (`feedback_ci_matrix_wider_than_local_gate`).
**How to avoid:** Add `///` doc comments to every new public struct and field in ferro-mcp. Run `cargo doc --no-deps -D warnings` locally before the publish gate.
**Warning signs:** `cargo doc` produces "missing documentation" warnings on new types.

### Pitfall 6: Publish gate false-pass without Cargo.toml change
**What goes wrong:** If the final push to master only touches docs files, the CI publish.yml change-gate sees only `docs/*` changes and skips the publish entirely (see publish.yml:54 — `docs/*` is in the skip list).
**Why it happens:** The publish workflow has a library-change gate that skips when only doc/planning files change.
**How to avoid:** The publish commit MUST include a workspace `Cargo.toml` version bump. Version bump commit = the trigger. Documented in D-11.

### Pitfall 7: Workspace version confusion (local 0.2.83 vs crates.io 0.2.80)
**What goes wrong:** Local master already has three unpushed commits at 0.2.81, 0.2.82, 0.2.83. If CI bumps the version again from 0.2.83 (auto-bump path), the final version is 0.2.84. If the publish commit manually sets 0.2.84 first, CI's "version already tagged" check sees the tag doesn't exist and uses `should_publish=yes` (no extra bump).
**Why it happens:** CI auto-bump logic: if `v$VERSION` tag exists, bump to next patch; if not, publish as-is. With unpushed history, the tag state on remote doesn't match local.
**How to avoid:** Verify via `git tag | grep v0.2.83` and via crates.io/gh API whether 0.2.83 is published before cutting the publish commit. Manual bump to the correct next version in the publish commit.

---

## Code Examples

### CLI --json Envelope (stable contract, MCP tool must mirror)
```rust
// Source: ferro-cli/src/commands/design_lint.rs:23-30 [VERIFIED]
#[derive(Serialize)]
pub struct FileFinding {
    pub file: String,
    #[serde(flatten)]
    pub finding: Finding,
}

// Wire shape of Finding (from ferro-json-ui/src/design/types.rs:25-37):
// {
//   "file": "src/views/orders.json",
//   "rule": "prefer-data-table",
//   "element_id": null,
//   "severity": "warning",
//   "message": "Raw Table used for entity list...",
//   "suggestion": "Replace Table with DataTable..."
// }
```

### Design Rule Registry (source for docs and catalog derivation)
```rust
// Source: ferro-json-ui/src/design/mod.rs:50-52 [VERIFIED]
pub fn rules() -> &'static [DesignRule] {
    rules::RULE_REGISTRY
}

// DesignRule fields available for derivation (from types.rs:41-54):
// pub id: &'static str        -- "prefer-data-table"
// pub title: &'static str     -- "Prefer DataTable for entity lists"
// pub rationale: &'static str -- "DataTable provides..."
// pub intents: &'static [&'static str] -- &["browse"]
// pub check: fn(...) -> Vec<Finding>  -- not serialized
```

### Token Vocabulary (ferro-theme/src/token.rs verified, 30 entries)
```rust
// Source: ferro-theme/src/token.rs:86-117 [VERIFIED]
// ALL_TOKENS: &[&str] — 30 CSS variable names
// Groups: surface (6), role (8), shape (4), shadow (3), typography (2),
//         density (1 - --spacing), motion (4 - fast/base/slow/ease),
//         focus (1 - --color-ring), display font (1 - --font-display)
// Test at token.rs:124 guards ALL_TOKENS.len() == 30
```

### Canonical Enum Sets (source for catalog vocabulary)
```rust
// Source: ferro-json-ui/src/catalog.rs:1229-1231 [VERIFIED]
const CANONICAL_VARIANT: &[&str] = &["primary", "secondary", "outline", "ghost", "destructive"];
const CANONICAL_TONE: &[&str] = &["neutral", "success", "warning", "destructive"];
const CANONICAL_SIZE: &[&str] = &["sm", "md", "lg"];
```

### Existing Generation Context (shape to extend)
```rust
// Source: ferro-mcp/src/tools/generation_context.rs:7-13 [VERIFIED]
pub struct GenerationContext {
    pub naming_conventions: NamingConventions,
    pub file_structure: FileStructure,
    pub common_patterns: CommonPatterns,
    pub avoid: Vec<String>,
    pub imports: ImportTemplates,
    // add: pub design_system: DesignSystemSummary
}
// Existing test at line 181: test_generation_context_has_all_sections
// — must be updated to include design_system field assertions
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No design enforcement at authoring boundary | `design::lint` rule engine (10 rules, intent-keyed) | Phase 252 (2026-07-03) | All Phase 253 derivations are possible |
| Per-component `variant` enum drift | Canonical `Variant`/`Tone`/`Size` enums (47 components) | Phase 251 | Vocabulary derivation in catalog is now a single source |
| 23 token slots | 30 slots (v2: density, motion, focus ring, display font) | Phase 250 | generation_context summary has the full vocabulary |
| `json_ui_catalog` description: "39 built-in" | Must be updated to "47" | Phase 251 (count change) | Stale string in service.rs:1303 |
| No `ferro-theme` dependency in ferro-mcp | Needs addition for count drift guard | Phase 253 | Minor Cargo.toml addition |

**Deprecated/outdated:**
- `"Textarea"` in `FIELD_TYPES` (rules.rs:298): not a registered builtin component; catalog validation rejects it before lint runs — dead entry, IN-01 fix.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `DesignRule` text scan for component names is sufficient for per-component guidance derivation without adding a `components` field to the struct | Architecture Patterns — Pattern 3 | Minor: if the text-scan approach produces poor coverage, the planner should instead add a `components: &'static [&'static str]` field to `DesignRule` in Phase 253 (this is Phase 252's API, but it's not published yet at plan time) |
| A2 | Adding `ferro-theme` to ferro-mcp's Cargo.toml creates no dependency cycle | Standard Stack | Low: ferro-theme has no dependencies on ferro-mcp; cargo tree would confirm |
| A3 | `CARGO_MANIFEST_DIR` resolves correctly for ferro-json-ui in CI to locate `docs/src/design-system/patterns.md` via `../docs/...` relative path | Pattern 5 | Medium: if CI checkout structure differs, the drift test panics rather than fails cleanly — use a `std::fs::read_to_string(...).ok()` check with a skip or an informative panic message |

---

## Open Questions

1. **Does `DesignRule` gain a `components` field, or is text-scan derivation used for per-component catalog guidance?**
   - What we know: `DesignRule` currently has `id/title/rationale/intents/check`; no component field exists.
   - What's unclear: Whether D-05 "derived from design::rules() metadata where a rule references the component" means text scanning or a structural field.
   - Recommendation: Text scanning is simpler and avoids touching Phase 252's shipped type. If coverage is poor (some rules don't mention component names in their text), add a `components` field to `DesignRule` as part of Phase 253.

2. **What version number to use for the publish commit?**
   - What we know: local master is at 0.2.83; crates.io is at 0.2.80; three unpushed commits cover 0.2.81-0.2.83.
   - What's unclear: Whether 0.2.81-0.2.83 have been published (MEMORY.md says crates.io at 0.2.80 but that note may be stale post-Phase 252).
   - Recommendation: At the publish gate, verify via `curl -s https://crates.io/api/v1/crates/ferro-rs | jq .crate.max_version` or `gh api repos/albertogferrario/ferro/releases/latest` before choosing the next version number.

3. **Are the 251 pixel-level visual pass items in the D-14 UAT checklist blocking or advisory?**
   - What we know: Phase 251 Plan 04 notes suggest a pixel-level pass at Phase 253 pre-publish review (not blocking, "suggested").
   - What's unclear: Whether Alberto considers this a hard gate for publish.
   - Recommendation: Include as a UAT checklist item (same pattern as 252's human CLI output check) but not a blocking hard gate.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain 1.88.0 | cargo fmt/clippy/test | ✓ | confirmed in CI workflow | — |
| `ferro-theme` crate | generation_context count drift guard | ✓ (workspace) | workspace version | — |
| `ferro-json-ui` with projections | design_lint tool, catalog extension | ✓ (ferro-mcp/Cargo.toml:24) | workspace version | — |
| mdBook | docs chapter authoring | not checked — pages are markdown files; no build step required in phase | — | n/a: pages are markdown |

**Missing dependencies with no fallback:** None — all needed crates are in the workspace.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (cargo test) |
| Config file | none — workspace `Cargo.toml` `[profile.test]` |
| Quick run command | `cargo test -p ferro-mcp design_lint` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DS-07 | design_lint tool accepts inline spec_json, returns FileFinding[] | unit | `cargo test -p ferro-mcp design_lint` | ❌ Wave 1 |
| DS-07 | design_lint tool accepts path, returns FileFinding[] | unit | `cargo test -p ferro-mcp design_lint` | ❌ Wave 1 |
| DS-07 | design_lint tool returns parse diagnostic for malformed spec (not tool error) | unit | `cargo test -p ferro-mcp design_lint` | ❌ Wave 1 |
| DS-07 | json_ui_catalog returns design_system field with variant/tone/size values | unit | `cargo test -p ferro-mcp json_ui_catalog` | ❌ Wave 2 |
| DS-07 | generation_context returns design_system field with tokens and intent_patterns | unit | `cargo test -p ferro-mcp generation_context` | ❌ Wave 2 (update existing test) |
| DS-07 | Token description count matches ferro_theme::token::ALL_TOKENS count | unit (drift guard) | `cargo test -p ferro-mcp generation_context` | ❌ Wave 2 |
| DS-08 | patterns.md covers all rule IDs from design::rules() | unit (drift guard) | `cargo test -p ferro-json-ui design` | ❌ Wave 3 |
| DS-08 | docs/src/design-system/ has all five pages | manual / file existence | `ls docs/src/design-system/` | ❌ Wave 3 |
| DS-08 | Workspace publishes to crates.io successfully | smoke (post-publish) | `cargo search ferro-rs` or `gh api` | — |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-mcp design_lint` (Wave 1), `cargo test -p ferro-mcp generation_context` (Wave 2), `cargo test -p ferro-json-ui design` (Wave 3)
- **Per wave merge:** `cargo test --all-features`
- **Phase gate (pre-publish):** `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features && cargo doc --no-deps -D warnings`

### Wave 0 Gaps

- [ ] `ferro-mcp/src/tools/design_lint.rs` — new file (DS-07 inline and path tests)
- [ ] `ferro-mcp/Cargo.toml` — add `ferro-theme` dependency
- [ ] `docs/src/design-system/` directory + five placeholder files (required before D-09 drift test)

*(All existing test infrastructure in ferro-mcp, ferro-json-ui, and ferro-cli covers prior phases; only the new surfaces need new test files.)*

---

## Security Domain

This phase adds MCP tooling (read-only design diagnostics), catalog metadata, generation context text, and documentation pages. No authentication, session management, access control, cryptography, or user input processing is involved. The `design_lint` tool operates on spec JSON provided by the calling agent — it calls `Spec::from_json` which is an existing, tested parser; no injection risk. Security enforcement is not applicable to this phase.

---

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/design/mod.rs` — `lint()`, `rules()`, `KNOWN_INTENTS`, drift test structure [VERIFIED]
- `ferro-json-ui/src/design/types.rs` — `Finding`, `Severity`, `DesignRule` field set [VERIFIED]
- `ferro-mcp/src/tools/json_ui_validate_spec.rs` — tool execute() pattern and param struct [VERIFIED]
- `ferro-mcp/src/service.rs:1403-1424` — closest tool registration analog [VERIFIED]
- `ferro-mcp/src/tools/json_ui_catalog.rs:12-51, 280-298` — `JsonUiCatalog` struct and 47-count mirror assertion [VERIFIED]
- `ferro-mcp/src/tools/generation_context.rs:7-57` — `GenerationContext` struct and existing test [VERIFIED]
- `ferro-cli/src/commands/design_lint.rs:22-30` — `FileFinding` stable `--json` contract [VERIFIED]
- `ferro-theme/src/token.rs:86-117` — `ALL_TOKENS` 30-entry constant and count guard test [VERIFIED]
- `ferro-mcp/Cargo.toml:24` — `ferro-json-ui` dependency with `projections` feature confirmed [VERIFIED]
- `ferro-json-ui/src/catalog.rs:1229-1231` — `CANONICAL_VARIANT/CANONICAL_TONE/CANONICAL_SIZE` [VERIFIED]
- `docs/src/SUMMARY.md:63-73` — JSON-UI chapter block for insertion point [VERIFIED]
- `docs/src/features/themes.md:55-132` — token v2 reference table [VERIFIED]
- `docs/src/json-ui/components.md:42-99` — migration table location [VERIFIED]
- `.github/workflows/publish.yml:50-60` — publish gate: `docs/*` changes do not trigger publish [VERIFIED]
- `Cargo.toml:46` — workspace version `0.2.83` [VERIFIED]
- `ferro-json-ui/src/design/rules.rs:298` — IN-01 dead `"Textarea"` in FIELD_TYPES [VERIFIED]

### Secondary (MEDIUM confidence)

- CONTEXT.md 253-CONTEXT.md — all D-01 through D-16 locked decisions
- 252-VERIFICATION.md — IN-01, IN-02 findings and pre-existing flaky test documentation
- 252-CONTEXT.md D-10/D-11 — public rule registry and Finding serialization contract

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all library usage verified against Cargo.toml and source files
- Architecture: HIGH — tool registration, struct extension, and drift-guard patterns verified in existing code
- Pitfalls: HIGH — all identified from concrete code evidence (stale string at service.rs:1303, dead constant at rules.rs:298, test field check at generation_context.rs:181)
- Docs structure: HIGH — SUMMARY.md and cross-link targets verified

**Research date:** 2026-07-04
**Valid until:** 2026-08-04 (stable domain — no external dependencies, all library versions are workspace-pinned)
