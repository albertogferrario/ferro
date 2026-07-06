# Phase 253: MCP surface + docs + publish - Pattern Map

**Mapped:** 2026-07-04
**Files analyzed:** 15 new/modified files
**Analogs found:** 15 / 15

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp/src/tools/design_lint.rs` | tool | request-response | `ferro-mcp/src/tools/json_ui_validate_spec.rs` | exact |
| `ferro-mcp/src/service.rs` | service | request-response | `ferro-mcp/src/service.rs:1403-1424` (json_ui_validate_spec registration) | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | service | request-response | itself (additive field; existing struct is the template) | exact |
| `ferro-mcp/src/tools/generation_context.rs` | service | request-response | itself (additive field; existing struct and test are the template) | exact |
| `ferro-mcp/Cargo.toml` | config | — | itself (existing path+version dep lines at lines 23-25) | exact |
| `ferro-json-ui/src/design/mod.rs` | test | — | itself (existing D-08 drift test at lines 284-314) | exact |
| `ferro-json-ui/src/design/rules.rs` | utility | — | itself (FIELD_TYPES constant at line 298) | exact |
| `ferro-cli/src/commands/design_lint.rs` | CLI command | request-response | itself (`print_human` function at lines 138-155) | exact |
| `docs/src/design-system/principles.md` | doc | — | `docs/src/json-ui/getting-started.md` | role-match |
| `docs/src/design-system/tokens.md` | doc | — | `docs/src/features/themes.md` (token reference table at lines 52-132) | exact |
| `docs/src/design-system/variants.md` | doc | — | `docs/src/json-ui/components.md` (Shared Enum Values section at lines 40-49) | exact |
| `docs/src/design-system/patterns.md` | doc | — | `docs/src/json-ui/components.md` (per-component sections) | role-match |
| `docs/src/design-system/linting.md` | doc | — | `docs/src/features/themes.md` (Quick Start + CLI usage pattern) | role-match |
| `docs/src/SUMMARY.md` | config | — | itself (JSON-UI chapter block at lines 61-73) | exact |
| `Cargo.toml` (workspace root) | config | — | itself (workspace version field) | exact |

---

## Pattern Assignments

### `ferro-mcp/src/tools/design_lint.rs` (new — tool, request-response)

**Analog:** `ferro-mcp/src/tools/json_ui_validate_spec.rs`

**Imports pattern** (analog lines 1-15):
```rust
//! design_lint tool — runs the ferro-json-ui design rule engine on a single spec,
//! either provided inline as JSON or read from a file path.
//!
//! Per Phase 253 D-01/D-04: input is spec_json XOR path; parse errors and XOR
//! violations are returned as Warning-level findings, never as tool errors.
//! The output shape is identical to the CLI `--json` envelope (252 D-11).

use ferro_json_ui::design::{lint, Finding, Severity};
use ferro_json_ui::spec::{Spec, SCHEMA_VERSION};
use serde::Serialize;
```

**FileFinding struct** — reuse from CLI (copy verbatim, `ferro-cli/src/commands/design_lint.rs` lines 23-30):
```rust
/// One finding tagged with the file it originated from.
///
/// This is the stable `--json` / MCP contract consumed by gestiscilo Phase 232.
/// Identical to `ferro_cli::commands::design_lint::FileFinding` by design (D-02).
#[derive(Debug, Serialize)]
pub struct FileFinding {
    /// "<inline>" for spec_json input; the given path for path input.
    pub file: String,
    #[serde(flatten)]
    pub finding: Finding,
}
```

**Core execute pattern** (analog: `json_ui_validate_spec.rs` lines 43-66, adapted for XOR + path):
```rust
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
```

**lint_string helper** — mirrors `ferro_cli::commands::design_lint::lint_content` (CLI lines 41-64):
```rust
fn lint_string(label: &str, content: &str) -> Vec<FileFinding> {
    if !content.contains(SCHEMA_VERSION) {
        // Non-ferro JSON: silently skip (same as CLI WalkDir behaviour).
        return vec![];
    }
    match Spec::from_json(content) {
        Ok(spec) => lint(&spec)
            .into_iter()
            .map(|finding| FileFinding { file: label.to_string(), finding })
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
```

**Test pattern** (analog: `json_ui_validate_spec.rs` lines 68-146 — four test cases):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN: &str = r#"{"$schema":"ferro-json-ui/v2","root":"t","layout":"auth",
        "design":{"intent":"focus"},"elements":{"t":{"type":"Text","props":{"content":"hi"}}}}"#;

    #[test]
    fn inline_clean_spec_returns_empty() {
        assert!(execute(Some(CLEAN), None).is_empty());
    }

    #[test]
    fn inline_malformed_returns_spec_parse_warning() {
        let findings = execute(Some(r#"{"$schema":"ferro-json-ui/v2","root":"missing","elements":{}}"#), None);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding.rule, "spec-parse");
        assert_eq!(findings[0].file, "<inline>");
    }

    #[test]
    fn path_mode_reads_and_lints() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(CLEAN.as_bytes()).unwrap();
        let p = f.path().to_str().unwrap().to_string();
        let findings = execute(None, Some(&p));
        assert!(findings.is_empty(), "clean spec should produce no findings");
    }

    #[test]
    fn both_none_returns_tool_input_warning() {
        let findings = execute(None, None);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].finding.rule, "tool-input");
    }
}
```

---

### `ferro-mcp/src/service.rs` (modified — param struct + tool method registration)

**Analog:** itself, lines 246-250 (param struct pattern) and lines 1403-1424 (tool method pattern)

**Param struct** — insert alongside `JsonUiValidateSpecParams` (lines 246-250):
```rust
// [service.rs:246-250 shows the derive set — copy exactly]
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DesignLintParams {
    /// Inline JSON-UI v2 spec string to lint.
    /// Provide either this or `path`, not both.
    pub spec_json: Option<String>,
    /// Path to a single JSON-UI spec file to lint.
    /// Provide either this or `spec_json`, not both.
    pub path: Option<String>,
}
```

**Tool method** — insert after `json_ui_validate_spec` method (lines 1418-1424):
```rust
/// Lint a JSON-UI spec for design-pattern conformance
#[tool(
    name = "design_lint",
    description = "Run design-pattern rules on a JSON-UI v2 spec and return findings.\n\n\
        Provide EITHER `spec_json` (inline JSON string) OR `path` (path to a single \
        .json file) — not both and not neither.\n\n\
        **When to use:** After writing or editing a spec, validate it against the \
        10 design rules before submitting for review. The tool returns the same \
        `FileFinding[]` shape as `ferro design:lint --json`, so the same \
        rules apply: Warning-level findings trip `--deny`; Info findings are advisory.\n\n\
        **Returns:** JSON array of `{ file, rule, element_id, severity, message, suggestion }`. \
        An empty array means the spec is clean. Parse failures and input errors are \
        returned as Warning-level findings — never as tool errors.\n\n\
        **Combine with:** `json_ui_validate_spec` to check structural + catalog validity, \
        `generation_context` for per-intent pattern expectations before authoring."
)]
pub async fn design_lint(&self, params: Parameters<DesignLintParams>) -> String {
    let result = tools::design_lint::execute(
        params.0.spec_json.as_deref(),
        params.0.path.as_deref(),
    );
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "[]".to_string())
}
```

**Stale string fix** — also update the `json_ui_catalog` description string at line 1303:
```
// Before: "Get a structured reference of all JSON-UI components: 39 built-in components"
// After:  "Get a structured reference of all JSON-UI components: 47 built-in components"
```

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` (modified — additive `design_system` field)

**Analog:** itself (own struct at lines 12-24, own `execute()` at lines 70-152, own tests at lines 281-598)

**Additive struct fields** — append to `JsonUiCatalog` struct (after `directives` field, line 23):
```rust
// New additive field — all existing fields and tests are unchanged.
/// Design system vocabulary derived from canonical enums and design::rules().
pub design_system: DesignVocabulary,
```

**New supporting structs** — insert after `DirectiveInfo` struct (after line 42):
```rust
/// Design vocabulary derived from canonical enum constants and the rule registry.
///
/// Sourced from `CANONICAL_VARIANT/CANONICAL_TONE/CANONICAL_SIZE`
/// (`ferro-json-ui/src/catalog.rs:1229-1231`) and `ferro_json_ui::design::rules()`.
#[derive(Debug, Serialize)]
pub struct DesignVocabulary {
    /// Canonical variant values (visual weight of interactive elements).
    pub variant_values: &'static [&'static str],
    /// Canonical tone values (semantic status color for stateful display components).
    pub tone_values: &'static [&'static str],
    /// Canonical size values.
    pub size_values: &'static [&'static str],
    /// Per-intent design rules: rule id + title + rationale, grouped by intent.
    pub intent_rules: std::collections::HashMap<&'static str, Vec<DesignRuleRef>>,
}

/// Minimal rule metadata for agent consumption (no check fn — not serializable).
#[derive(Debug, Serialize)]
pub struct DesignRuleRef {
    /// Stable rule id used in `allow` lists and finding `rule` fields.
    pub id: &'static str,
    /// Short human title.
    pub title: &'static str,
    /// One-sentence rationale.
    pub rationale: &'static str,
}
```

**execute() extension** — append `design_system` to the `JsonUiCatalog { ... }` literal (around line 143):
```rust
// Import at top of function or file:
use ferro_json_ui::design::rules as design_rules;
use ferro_json_ui::design::KNOWN_INTENTS;

// Constants already in ferro-json-ui/src/catalog.rs — re-export or duplicate here:
const CANONICAL_VARIANT: &[&str] = &["primary", "secondary", "outline", "ghost", "destructive"];
const CANONICAL_TONE: &[&str] = &["neutral", "success", "warning", "destructive"];
const CANONICAL_SIZE: &[&str] = &["sm", "md", "lg"];

// In execute():
let mut intent_rules: std::collections::HashMap<&'static str, Vec<DesignRuleRef>> =
    KNOWN_INTENTS.iter().map(|&i| (i, Vec::new())).collect();
intent_rules.insert("all", Vec::new()); // rules with empty intents apply to all

for rule in design_rules() {
    let entry = if rule.intents.is_empty() {
        intent_rules.entry("all").or_default()
    } else {
        for &intent in rule.intents {
            intent_rules.entry(intent).or_default().push(DesignRuleRef {
                id: rule.id,
                title: rule.title,
                rationale: rule.rationale,
            });
        }
        continue;
    };
    entry.push(DesignRuleRef { id: rule.id, title: rule.title, rationale: rule.rationale });
}

JsonUiCatalog {
    // ... existing fields unchanged ...
    design_system: DesignVocabulary {
        variant_values: CANONICAL_VARIANT,
        tone_values: CANONICAL_TONE,
        size_values: CANONICAL_SIZE,
        intent_rules,
    },
}
```

**New test** — append to `mod tests` (after existing tests):
```rust
#[test]
fn design_system_vocabulary_present() {
    let catalog = execute(None);
    assert!(!catalog.design_system.variant_values.is_empty());
    assert!(catalog.design_system.variant_values.contains(&"primary"));
    assert!(catalog.design_system.tone_values.contains(&"destructive"));
    assert!(catalog.design_system.size_values.contains(&"md"));
    // At least one intent with rules
    assert!(catalog.design_system.intent_rules.values().any(|v| !v.is_empty()));
}
```

---

### `ferro-mcp/src/tools/generation_context.rs` (modified — additive `design_system` field)

**Analog:** itself (struct at lines 7-13, execute() at lines 59-173, test at lines 181-259)

**Additive struct field** — append to `GenerationContext` struct (after `imports` field, line 12):
```rust
/// Design system summary for JSON-UI spec authoring.
pub design_system: DesignSystemSummary,
```

**New supporting structs** — insert after `ImportTemplates` struct (after line 56):
```rust
/// Design system summary for agent-authoring context.
///
/// Token descriptions are maintained as a static array; count is drift-guarded
/// against `ferro_theme::token::ALL_TOKENS.len()` (30 entries).
#[derive(Debug, Serialize)]
pub struct DesignSystemSummary {
    /// Semantic token vocabulary (30 slots). Each entry: CSS variable name + purpose.
    pub tokens: &'static [TokenInfo],
    /// Design rules grouped by intent: rule id + title + rationale.
    pub intent_patterns: std::collections::HashMap<&'static str, Vec<IntentPattern>>,
    /// Canonical variant/tone/size value lists.
    pub canonical_variants: CanonicalVariants,
    /// Pointer to full design system documentation.
    pub docs: &'static str,
}

/// One semantic token slot: CSS variable name + one-line purpose.
#[derive(Debug, Serialize)]
pub struct TokenInfo {
    /// CSS custom property name (e.g. `"--color-primary"`).
    pub name: &'static str,
    /// One-line purpose description.
    pub purpose: &'static str,
}

/// Rule metadata for a specific intent, derived from the rule registry.
#[derive(Debug, Serialize)]
pub struct IntentPattern {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub rationale: &'static str,
}

/// Canonical shared enum values across JSON-UI components.
#[derive(Debug, Serialize)]
pub struct CanonicalVariants {
    pub variant: &'static [&'static str],
    pub tone: &'static [&'static str],
    pub size: &'static [&'static str],
}
```

**Static token table** — define as a module-level constant (with count drift guard):
```rust
use ferro_theme::token::ALL_TOKENS;

// Maintained in parallel with ferro_theme::token::ALL_TOKENS (30 entries).
// The drift guard below asserts these stay in sync.
static DESIGN_TOKEN_DESCRIPTIONS: &[TokenInfo] = &[
    TokenInfo { name: "--color-background", purpose: "Page/canvas background" },
    TokenInfo { name: "--color-surface", purpose: "Component surface (cards, panels)" },
    TokenInfo { name: "--color-card", purpose: "Card background (may differ from surface)" },
    TokenInfo { name: "--color-border", purpose: "Dividers, input borders, separators" },
    TokenInfo { name: "--color-text", purpose: "Primary text" },
    TokenInfo { name: "--color-text-muted", purpose: "Secondary/muted text, placeholders" },
    TokenInfo { name: "--color-primary", purpose: "Primary action color (buttons, links)" },
    TokenInfo { name: "--color-primary-foreground", purpose: "Text on primary-colored surfaces" },
    TokenInfo { name: "--color-secondary", purpose: "Secondary action / subdued UI elements" },
    TokenInfo { name: "--color-secondary-foreground", purpose: "Text on secondary-colored surfaces" },
    TokenInfo { name: "--color-accent", purpose: "Accent highlight (hover, selection)" },
    TokenInfo { name: "--color-destructive", purpose: "Destructive actions and danger states" },
    TokenInfo { name: "--color-success", purpose: "Success / positive states" },
    TokenInfo { name: "--color-warning", purpose: "Warning / caution states" },
    TokenInfo { name: "--radius-sm", purpose: "Small corner radius (badges, chips)" },
    TokenInfo { name: "--radius-md", purpose: "Medium corner radius (buttons, inputs)" },
    TokenInfo { name: "--radius-lg", purpose: "Large corner radius (cards, modals)" },
    TokenInfo { name: "--radius-full", purpose: "Full / pill corner radius (avatars)" },
    TokenInfo { name: "--shadow-sm", purpose: "Small elevation shadow" },
    TokenInfo { name: "--shadow-md", purpose: "Medium elevation shadow (dropdowns)" },
    TokenInfo { name: "--shadow-lg", purpose: "Large elevation shadow (modals)" },
    TokenInfo { name: "--font-sans", purpose: "Body / UI sans-serif font stack" },
    TokenInfo { name: "--font-mono", purpose: "Monospace font stack (code, IDs)" },
    TokenInfo { name: "--spacing", purpose: "Base spacing unit (density scale)" },
    TokenInfo { name: "--motion-duration-fast", purpose: "Fast transitions (100-150 ms)" },
    TokenInfo { name: "--motion-duration-base", purpose: "Standard transitions (200-250 ms)" },
    TokenInfo { name: "--motion-duration-slow", purpose: "Slow transitions (300-400 ms)" },
    TokenInfo { name: "--motion-ease", purpose: "Default easing curve" },
    TokenInfo { name: "--color-ring", purpose: "Focus ring / outline color" },
    TokenInfo { name: "--font-display", purpose: "Display/heading font (defaults to --font-sans)" },
];

// Drift guard: fails compilation if DESIGN_TOKEN_DESCRIPTIONS gets out of sync
// with ferro_theme::token::ALL_TOKENS.
const _: () = {
    // This fires at test time, not compile time, because ALL_TOKENS.len() is runtime.
    // The test below enforces it.
};
```

**Count drift guard test** — in `mod tests`:
```rust
#[test]
fn token_description_count_matches_all_tokens() {
    assert_eq!(
        DESIGN_TOKEN_DESCRIPTIONS.len(),
        ferro_theme::token::ALL_TOKENS.len(),
        "DESIGN_TOKEN_DESCRIPTIONS must have one entry per ALL_TOKENS slot (D-06 drift guard)"
    );
}
```

**Updated all-sections test** — extend `test_generation_context_has_all_sections` (line 181):
```rust
// Append to the existing test after the `imports` assertions:
assert!(!context.design_system.tokens.is_empty());
assert_eq!(context.design_system.tokens.len(), 30);
assert!(!context.design_system.canonical_variants.variant.is_empty());
assert!(!context.design_system.intent_patterns.is_empty());
assert!(!context.design_system.docs.is_empty());
```

---

### `ferro-mcp/Cargo.toml` (modified — new ferro-theme dependency)

**Analog:** itself, lines 23-25 (existing path+version dep pattern)

**Pattern** (lines 23-25 show the exact format to copy):
```toml
# Existing deps for reference:
ferro-ai = { path = "../ferro-ai", version = "0.2" }
ferro-json-ui = { path = "../ferro-json-ui", version = "0.2", features = ["projections"] }
ferro-projections = { path = "../ferro-projections", version = "0.2" }

# New line to insert (alphabetical order after ferro-json-ui):
ferro-theme = { path = "../ferro-theme", version = "0.2" }
```

---

### `ferro-json-ui/src/design/mod.rs` (modified — add D-09 docs drift test)

**Analog:** itself, lines 284-314 (the D-08 intent drift test — same structure: `#[cfg(all(test, feature = "projections"))]` block reading external state and asserting coverage)

**Test pattern** (copy the test block structure from lines 284-314):
```rust
// ── D-09 docs drift test ──────────────────────────────────────────────────────
// Run only when the projections feature is active (same gate as D-08).
// Or run unconditionally in #[cfg(test)] — preferred since patterns.md
// does not require the projections feature:

#[cfg(test)]
mod docs_drift_tests {
    use super::rules;

    /// Assert every rule id from the registry appears in patterns.md.
    ///
    /// Wave ordering: patterns.md must be committed before this test is added
    /// (Pitfall 4 in RESEARCH.md). CARGO_MANIFEST_DIR from ferro-json-ui/
    /// resolves to ../../docs/src/design-system/patterns.md.
    #[test]
    fn patterns_md_covers_all_rule_ids() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set");
        let patterns_path = std::path::Path::new(&manifest_dir)
            .join("../docs/src/design-system/patterns.md");
        let content = std::fs::read_to_string(&patterns_path)
            .unwrap_or_else(|e| {
                panic!("patterns.md not found at {}: {} (D-09 drift guard)", patterns_path.display(), e)
            });

        for rule in rules() {
            assert!(
                content.contains(rule.id),
                "patterns.md is missing rule id `{}` — add a section for it (D-09)",
                rule.id
            );
        }
    }
}
```

**Placement:** append after the D-08 drift test block (line 314), keeping the two test modules separate.

---

### `ferro-json-ui/src/design/rules.rs` (modified — IN-01: remove "Textarea" from FIELD_TYPES)

**Analog:** itself, line 298

**Before** (line 298):
```rust
const FIELD_TYPES: &[&str] = &["Input", "Select", "Textarea", "RichTextEditor"];
```

**After** (IN-01 fix — "Textarea" is not a registered builtin; RichTextEditor is a plugin component):
```rust
const FIELD_TYPES: &[&str] = &["Input", "Select", "RichTextEditor"];
```

**Verification:** Run `cargo test -p ferro-json-ui design` — all existing `form-default-values` rule tests must still pass. The check functions at lines 300-340 use `FIELD_TYPES.contains(...)` unchanged.

---

### `ferro-cli/src/commands/design_lint.rs` (modified — IN-02: fix "No findings" when zero files linted)

**Analog:** itself, `print_human` function at lines 138-155

**Current behavior** (lines 147-154): prints "No findings — all specs are clean." whenever `files_seen.is_empty()`, even if zero JSON files were discovered (no files walked at all).

**IN-02 fix** — distinguish "zero files found" from "files found, all clean":
```rust
fn print_human(all: &[FileFinding]) {
    // Collect files in encounter order.
    let mut files_seen: Vec<&str> = Vec::new();
    for ff in all {
        let f = ff.file.as_str();
        if !files_seen.contains(&f) {
            files_seen.push(f);
        }
    }

    if files_seen.is_empty() {
        // IN-02: "all specs are clean" implies specs were checked.
        // When no files were discovered, say so explicitly.
        // The caller (run()) passes all: Vec<FileFinding> which is empty when:
        //   (a) no JSON files discovered, or (b) all files were non-ferro and skipped.
        // We cannot distinguish (a) from (b) here without a file-count parameter.
        // Simplest correct fix: change the message to be non-misleading in both cases.
        println!(
            "{}",
            style("No findings.").green().bold()
        );
        return;
    }
    // ... rest of function unchanged ...
}
```

**Alternative fix** (cleaner — pass a `files_walked: usize` counter from `run()`):
```rust
// In run(), count files processed before lint:
let mut files_linted: usize = 0;
// Inside the WalkDir loop, after lint_content is called on a ferro-marker file:
//   files_linted += 1;
// Then pass files_linted to print_human or check after the loop.

// Then in print_human or at the end of run():
if all.is_empty() {
    if files_linted == 0 {
        println!("{}", style("No JSON-UI spec files found.").yellow());
    } else {
        println!("{}", style("No findings — all specs are clean.").green().bold());
    }
    return;
}
```

**Recommendation:** The two-counter approach (files_linted) is the cleanest; the simpler "No findings." message change is acceptable if one-liner is the target. Both fix the misleading output.

---

### `docs/src/design-system/principles.md` (new — doc)

**Analog:** `docs/src/json-ui/getting-started.md` (neutral product voice, brief intro + concept sections)

**Structure pattern:**
```markdown
# Design System Principles

JSON-UI specs are validated against a set of design rules at authoring time
and in CI. This chapter describes the three pillars of the system and how
they fit together.

## Semantic Tokens

... one paragraph per pillar ...

## Intent-Keyed Patterns

... link to patterns.md ...

## Lint as Diagnostics

... link to linting.md ...

See [Token Reference](tokens.md), [Variant Vocabulary](variants.md),
[Pattern Catalog](patterns.md), and [Linting Guide](linting.md) for details.
```

---

### `docs/src/design-system/tokens.md` (new — doc)

**Analog:** `docs/src/features/themes.md` lines 52-132 (token reference table format — verbatim heading + table structure to copy)

**Table structure to copy** (themes.md lines 57-60):
```markdown
| Token | Default (light) | Purpose |
|-------|----------------|---------|
| `--color-primary` | `oklch(55% 0.22 250)` | Primary action color (buttons, links, highlights) |
```

**Key guidance:** Cross-link to `features/themes.md` for authoring recipe — never duplicate the `tokens.css` instructions. The `tokens.md` page owns the vocabulary reference; `themes.md` owns the authoring guide. One sentence at the top:

```markdown
For how to write and activate a theme that customizes these tokens, see
[Themes](../features/themes.md).
```

---

### `docs/src/design-system/variants.md` (new — doc)

**Analog:** `docs/src/json-ui/components.md` lines 40-53 (Shared Enum Values section — the canonical definitions that this page expands on)

**Key guidance:** Cross-link to `json-ui/components.md` for the migration table. Never duplicate the rename table. Opening line:

```markdown
These canonical values are enforced at catalog validation time and checked
by the design lint engine. For the full component rename/migration table from
earlier versions, see [Components](../json-ui/components.md#component-specific-enum-values).
```

**Table format to copy** (components.md lines 44-48):
```markdown
**variant** (visual weight of interactive elements) — `"primary"` | `"secondary"` |
`"outline"` | `"ghost"` | `"destructive"`
```

---

### `docs/src/design-system/patterns.md` (new, drift-guarded — doc)

**Analog:** `docs/src/json-ui/components.md` (per-component sections with prose + code blocks)

**Required structure** — the D-09 drift test asserts every rule id from `design::rules()` appears in this file. As of Phase 252 the rule ids are:

```
page-header
prefer-data-table
list-empty-state
row-actions-grouped
breadcrumb-on-subpages
process-kanban
card-actions-in-menu
create-separate-page
form-default-values
destructive-confirmation
```

Each section must contain the rule id as a plain string (for the drift guard `content.contains(rule.id)` check):

```markdown
## `page-header`

**Title:** Dashboard pages start with a PageHeader

**Rationale:** A PageHeader gives every app page a consistent title, breadcrumb,
and action-button slot.

**Intents:** all

### Conforming example

...

### Violating example

...

### How to allow

Add `"allow": ["page-header"]` to the `design` object when the layout is exempt
(e.g., auth, blank, or embedded frames).
```

Repeat this block for every rule id. The `DesignRule.rationale` field is the canonical source for the rationale text — copy it verbatim to keep prose and machine metadata in sync.

---

### `docs/src/design-system/linting.md` (new — doc)

**Analog:** `docs/src/features/themes.md` (Quick Start section + CLI reference pattern)

**Structure:**
```markdown
# Linting Guide

The `ferro design:lint` command and the `design_lint` MCP tool run the same
rule engine against JSON-UI specs.

## CLI Usage

```bash
ferro design:lint              # lint src/views/
ferro design:lint src/views/orders.json   # single file
ferro design:lint --json       # machine-readable output
ferro design:lint --deny       # non-zero exit on warnings (CI gate)
```

## MCP Tool

Use `design_lint` inside an agent session to validate a spec before saving it.
Provide exactly one of `spec_json` (inline) or `path` (file path):

...

## Output Shape

...

## Allowing a Rule

...
```

---

### `docs/src/SUMMARY.md` (modified — chapter registration)

**Analog:** itself, lines 61-73 (the JSON-UI chapter block — exact format to copy)

**JSON-UI block** (lines 61-73, existing):
```markdown
# JSON-UI

- [Getting Started](json-ui/getting-started.md)
- [Components](json-ui/components.md)
...
- [JSON Schema](json-ui/json-schema.md)
```

**New block to insert after line 73** (immediately after the JSON-UI chapter, before `# Agents`):
```markdown
# Design System

- [Principles](design-system/principles.md)
- [Token Reference](design-system/tokens.md)
- [Variant Vocabulary](design-system/variants.md)
- [Pattern Catalog](design-system/patterns.md)
- [Linting Guide](design-system/linting.md)
```

---

### `Cargo.toml` (workspace root — version bump at publish)

**Analog:** itself (the `version` field under `[workspace.package]`)

**Pattern** (current value: `"0.2.83"`):
```toml
[workspace.package]
version = "0.2.XX"   # bump to next available patch before the publish push
```

**Pre-publish verification command:**
```bash
curl -s https://crates.io/api/v1/crates/ferro-rs | jq .crate.max_version
```
Set XX = (crates.io max_version patch + 1). Do not guess — the local 0.2.83 may or may not be pushed.

---

## Shared Patterns

### `#[tool]` Registration (all new MCP tools)
**Source:** `ferro-mcp/src/service.rs` lines 1403-1424
**Apply to:** `design_lint` tool registration in service.rs

```rust
// 1. Param struct derives (service.rs:246):
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SomeToolParams { /* Option<String> fields for optional params */ }

// 2. Tool method signature (service.rs:1418):
pub async fn tool_name(&self, params: Parameters<SomeToolParams>) -> String {
    let result = tools::module_name::execute(params.0.field.as_deref());
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "[]".to_string())
}
```

### Execute() Function Shape (all tool modules)
**Source:** `ferro-mcp/src/tools/json_ui_validate_spec.rs` lines 43-66
**Apply to:** `design_lint::execute()`

```rust
// Always a pure function; all I/O (file read) happens inside execute(), not in service.rs
pub fn execute(input: Option<&str>) -> SomeResponseType {
    // Early return on error → wrap as finding, never panic
    // Return serializable type (derives Serialize)
}
```

### Drift Guard Test Pattern
**Source:** `ferro-json-ui/src/design/mod.rs` lines 284-314 (D-08 drift test)
**Apply to:** D-09 docs drift test in `mod.rs`, token count drift guard in `generation_context.rs`

```rust
// In ferro-json-ui/src/design/mod.rs:
#[cfg(test)]
mod docs_drift_tests {
    use super::rules;
    #[test]
    fn patterns_md_covers_all_rule_ids() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = std::path::Path::new(&manifest_dir).join("../docs/src/design-system/patterns.md");
        let content = std::fs::read_to_string(&path).expect("patterns.md must exist");
        for rule in rules() {
            assert!(content.contains(rule.id), "missing: {}", rule.id);
        }
    }
}

// In generation_context.rs:
#[test]
fn token_description_count_matches_all_tokens() {
    assert_eq!(DESIGN_TOKEN_DESCRIPTIONS.len(), ferro_theme::token::ALL_TOKENS.len());
}
```

### Additive Struct Extension Pattern
**Source:** `ferro-mcp/src/tools/json_ui_catalog.rs` lines 12-24 and `generation_context.rs` lines 7-13
**Apply to:** `design_system` field additions to both structs

```rust
// Rule: new field appended last; existing tests left unchanged; new tests added.
// Serialization test updated to check new field name appears in JSON output.
#[derive(Debug, Serialize)]
pub struct ExistingStruct {
    // ... existing fields unchanged ...
    pub design_system: NewSummaryType,  // <— append last
}
```

### Docs Page Voice
**Source:** `docs/src/json-ui/getting-started.md` lines 1-40, `docs/src/features/themes.md` lines 1-55
**Apply to:** all five `design-system/*.md` pages

- Neutral product documentation voice — no "v2 vs legacy" comparisons, no "we" or personal pronouns
- Cross-link rather than duplicate: `features/themes.md` owns token authoring; `json-ui/components.md` owns the migration table
- Code blocks for all CLI commands, JSON examples, and Rust snippets

---

## No Analog Found

All files have a clear analog. No novel patterns are required.

---

## Metadata

**Analog search scope:** `ferro-mcp/src/`, `ferro-json-ui/src/design/`, `ferro-cli/src/commands/`, `docs/src/`
**Files scanned:** 12 source files + 3 docs pages read directly
**Pattern extraction date:** 2026-07-04
