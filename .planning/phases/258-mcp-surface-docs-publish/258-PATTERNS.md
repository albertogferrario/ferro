# Phase 258: MCP Surface + Docs + Publish — Pattern Map

**Mapped:** 2026-07-06
**Files analyzed:** 6
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-mcp/src/tools/generation_context.rs` | mcp-tool | request-response | Same file — `design_system: DesignSystemSummary` field (Phase 253) | exact |
| `ferro-mcp/src/tools/json_ui_catalog.rs` | mcp-tool | request-response | Same file — `BUILDER_API` const + `RULE_COMPONENTS` static | exact |
| `docs/src/json-ui/components.md` | documentation | N/A | Same file — `### Tile` section at line 1411 | exact |
| `docs/src/json-ui/layouts.md` | documentation | N/A | Same file — existing layout subsections (`### "dashboard"`, etc.) | exact |
| `docs/src/json-ui/spec-construction.md` | documentation | N/A | Same file — `## Heterogeneous runtime construction — SpecBuilder` section | role-match |
| `Cargo.toml` (workspace version bump + publish) | config + publish | N/A | `.planning/phases/253-mcp-surface-docs-publish/253-05-PLAN.md` | exact |

---

## Pattern Assignments

### `ferro-mcp/src/tools/generation_context.rs` — add `register_composition` field

**Analog:** Same file, `design_system: DesignSystemSummary` field added in Phase 253.

**Struct declaration pattern** (lines 7–15 + 60–94):
```rust
// Top-level field on GenerationContext — copy this pattern verbatim
pub struct GenerationContext {
    pub naming_conventions: NamingConventions,
    // ...existing fields...
    /// Design system summary for JSON-UI spec authoring (D-06).
    pub design_system: DesignSystemSummary,
    // NEW field follows the same convention: doc comment with decision ref, concrete struct
    /// Register composition guidance for POS-style sale screens (D-03).
    pub register_composition: RegisterCompositionGuidance,
}

// Companion struct pattern from DesignSystemSummary (lines 61–71):
/// Design system summary for agent-authoring context (D-06).
#[derive(Debug, Serialize)]
pub struct DesignSystemSummary {
    /// Semantic token vocabulary (30 slots). Each entry: CSS variable name + one-line purpose.
    pub tokens: &'static [TokenInfo],
    /// Design rules grouped by intent key: rule id + title + rationale.
    pub intent_patterns: std::collections::HashMap<String, Vec<IntentPattern>>,
    /// Canonical variant/tone/size value lists.
    pub canonical_variants: CanonicalVariants,
    /// Pointer to full design system documentation.
    pub docs: &'static str,
}
```

**Sub-struct pattern for rule references** (lines 80–86):
```rust
/// Rule metadata for a specific intent, derived from the rule registry.
#[derive(Debug, Serialize)]
pub struct IntentPattern {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub rationale: &'static str,
}
// For RegisterCompositionGuidance, copy this pattern for RegisterRuleRef:
// pub struct RegisterRuleRef { pub id: &'static str, pub title: &'static str, pub rationale: &'static str }
```

**derive/execute() pattern — derivation from design_rules()** (lines 224–268):
```rust
// In execute() — copy this derivation block for register rules:
use ferro_json_ui::design::rules as design_rules;

// Group rules by intent (existing pattern):
for rule in design_rules() {
    if rule.intents.is_empty() {
        intent_patterns.entry("all".to_string()).or_default()
            .push(IntentPattern { rule_id: rule.id, title: rule.title, rationale: rule.rationale });
    } else {
        for &intent in rule.intents {
            intent_patterns.entry(intent.to_string()).or_default()
                .push(IntentPattern { rule_id: rule.id, title: rule.title, rationale: rule.rationale });
        }
    }
}

// For register_composition, filter by id instead of grouping by intent:
let register_rule_ids = ["register-fill-viewport", "register-grid-fill",
                          "register-selection-present", "fill-viewport-layout-unknown"];
let lint_rules: Vec<RegisterRuleRef> = design_rules()
    .iter()
    .filter(|r| register_rule_ids.contains(&r.id))
    .map(|r| RegisterRuleRef { id: r.id, title: r.title, rationale: r.rationale })
    .collect();
```

**Static string field pattern** (lines 278–280):
```rust
// docs pointer field — compact one-liner, pointer to docs/src/ for depth:
docs: "See docs/src/design-system/ for the full design system \
       (principles, tokens, variants, patterns, linting).",
// Register equivalent:
docs: "See docs/src/json-ui/layouts.md#register-layout-template and \
       docs/src/json-ui/components.md#tilegrid for depth.",
```

**Drift-guard test pattern** (lines 445–451):
```rust
// The token count test is the canonical drift-guard model:
#[test]
fn token_description_count_matches_all_tokens() {
    assert_eq!(
        DESIGN_TOKEN_DESCRIPTIONS.len(),
        ferro_theme::token::ALL_TOKENS.len(),
        "DESIGN_TOKEN_DESCRIPTIONS must have one entry per ALL_TOKENS slot (D-06 drift guard)"
    );
}
// For register guidance, the parallel pattern asserts that every name/id/attr mentioned
// in the guidance string exists in its authoritative source (registry, BUILTIN_TYPES, runtime JS).
```

**Existing test to extend** (lines 404–443):
```rust
#[test]
fn test_generation_context_has_all_sections() {
    let context = execute();
    // ...existing assertions...
    // Verify design system summary populated (D-06)
    assert_eq!(context.design_system.tokens.len(), 30);
    assert!(!context.design_system.intent_patterns.is_empty());
    // NEW: add parallel assertion for register_composition field:
    // assert!(!context.register_composition.when_to_use.is_empty());
    // assert!(!context.register_composition.lint_rules.is_empty());
}
```

---

### `ferro-mcp/src/tools/json_ui_catalog.rs` — BUILDER_API string additions + RULE_COMPONENTS fix

**Analog:** Same file — the `BUILDER_API` const (lines 347–370) and `RULE_COMPONENTS` static (lines 81–104).

**BUILDER_API const pattern** (lines 347–370):
```rust
const BUILDER_API: &str = "\
Spec::builder() -> SpecBuilder
  .title(impl Into<String>) -> Self
  .layout(impl Into<String>) -> Self
  .data(serde_json::Value) -> Self
  .element(id, Element) -> Self
  .build() -> Result<Spec, SpecError>

Element::new(type_name: impl Into<String>) -> ElementBuilder
  .prop(key, value) -> Self (accumulates into props: serde_json::Value)
  .child(id: impl Into<String>) -> Self (child element id reference)
  .action(Action) -> Self (click/submit handler)
  .visible(Visibility) -> Self (show/hide based on data path)

Spec { $schema, root, elements: HashMap<String, Element>, title?, layout?, data? }
  ...
Element { type: String, props: Value, children: Vec<String>, action?, visible? }
  ...";
```

Two lines are missing from this string (Phase 257 additions). Extend additively:
- After `.build() -> Result<Spec, SpecError>` in the SpecBuilder block, add:
  `.fill_viewport(bool) -> Self  (sets fill_viewport on the Spec; required for register layouts)`
- After `.visible(Visibility) -> Self` in the ElementBuilder block, add:
  `.each(path: impl Into<String>, as_: impl Into<String>) -> Self  (public setter for the $each directive)`

**RULE_COMPONENTS static pattern** (lines 81–104):
```rust
static RULE_COMPONENTS: &[(&str, &[&str])] = &[
    ("page-header", &["PageHeader"]),
    // ...
    // POS rules (Phase 254/256). TileGrid added in Phase 256-02.
    ("register-fill-viewport", &["Grid", "TileGrid"]),          // GAP: missing SelectionPanel, Numpad
    ("register-grid-fill", &["Grid", "TileGrid"]),
    ("register-selection-present", &["Grid", "TileGrid", "Numpad", "SelectionPanel"]),
    ("fill-viewport-layout-unknown", &[]),                       // GAP: consider &["Grid"]
];
```

Fix: change `("register-fill-viewport", &["Grid", "TileGrid"])` to:
```rust
("register-fill-viewport", &["Grid", "TileGrid", "SelectionPanel", "Numpad"]),
```

**Drift-guard test that auto-covers RULE_COMPONENTS changes** (lines 738–781):
```rust
#[test]
fn design_system_component_guidance_drift_guarded() {
    use std::collections::HashSet;
    let catalog = execute(None);
    // ...
    let registry_ids: HashSet<&str> = ferro_json_ui::design::rules()
        .iter().map(|r| r.id).collect();
    let mapped_ids: HashSet<&str> = RULE_COMPONENTS.iter().map(|(id, _)| *id).collect();
    // Direction 1: every mapped rule id exists in the registry.
    for id in &mapped_ids {
        assert!(registry_ids.contains(id), "RULE_COMPONENTS references unknown rule id `{id}`");
    }
    // Direction 2: every registry rule id is mapped (no silent drift when a rule is added).
    for id in &registry_ids {
        assert!(mapped_ids.contains(id), "design rule `{id}` is not mapped in RULE_COMPONENTS");
    }
    // Direction 3: every component name is a real builtin.
    let builtins: HashSet<&str> = catalog.components.iter().map(|c| c.name.as_str()).collect();
    for (_, comps) in RULE_COMPONENTS {
        for &c in *comps { assert!(builtins.contains(c), "non-builtin `{c}`"); }
    }
}
```

This test runs automatically after the RULE_COMPONENTS edit — no new test needed for the RULE_COMPONENTS fix itself. New tests ARE needed for the BUILDER_API additions:
```rust
// New test (follows test_builder_api_present pattern at lines 573–583):
#[test]
fn builder_api_mentions_fill_viewport_and_each() {
    let catalog = execute(None);
    assert!(catalog.builder_api.contains("fill_viewport"),
        "BUILDER_API must document fill_viewport(bool) (Phase 257 addition)");
    assert!(catalog.builder_api.contains(".each("),
        "BUILDER_API must document .each(path, as_) (Phase 257 addition)");
}
```

---

### `docs/src/json-ui/components.md` — five new component sections

**Analog:** `### Tile` section at line 1411. This is the exact format anchor. Copy it verbatim for each new section.

**Format pattern** (lines 1411–1443):
```markdown
### Tile

Touch-first tap-to-add tile. [One descriptive paragraph. States what the component
does, its primary interaction model, and any compositional constraints.]

| Prop | Type | Description |
|------|------|-------------|
| `item_id` | `string` | Item identifier |
| `name` | `string` | Item name — also emitted as `data-filter-text` ... |
| `price` | `string` | Formatted display price (e.g., `"€29.00"`) |
| `field` | `string` | Form field name the selected quantity is written to |
| `default_quantity` | `number \| null` | Initial quantity (default: 0) |
| `categories` | `string[]` | Category memberships, emitted as ... |
| `image_url` | `string \| null` | Item image, lazy-loaded ...; absent renders a text-only tile |
| `color` | `tone \| null` | Accent tone for the tile border ... |
| `stock_badge` | `string \| null` | Badge-styled chip text ... |
| `price_cents` | `number \| null` | Machine-readable unit price in integer cents ... |

```json
"tile": {
  "type": "Tile",
  "props": {
    "item_id": { "$data": "/product/id" },
    "name": { "$data": "/product/name" },
    "price": { "$data": "/product/price_formatted" },
    "price_cents": { "$data": "/product/price_cents" },
    "field": "quantities[1]"
  }
}
```

Place Tile elements inside a `Form` — [one-line usage note tying back to compositional constraints].

---
```

**Placement:** Extend `## Commerce Components` (line 1409), inserting `### TileGrid`, `### SelectionPanel`, `### FilterTabs`, `### QuantityStepper`, `### Numpad` after `### Tile` (line 1411) and before `## Kanban Components` (line 1445).

**Type annotation convention** (from the Tile section, line 1422):
- Optional fields: `number \| null` (backslash-escaped pipe in markdown tables)
- Array fields: `string[]`
- Enum fields: `"quantity" \| "price"` (quoted values, escaped pipe)
- Required fields have no null suffix in the type column

**Notes paragraph rule:** Every section closes with a usage note that states the compositional constraint (what must the component be placed inside, what must match what). For the `### Tile` format anchor this is: "Place Tile elements inside a `Form`…". For new sections, state the `form_id`-pairing constraint, the `fill_viewport` requirement, or the `Form` common-ancestor scoping rule as appropriate.

---

### `docs/src/json-ui/layouts.md` — fill_viewport section + Register Layout Template section

**Analog:** Existing layout subsections in the same file (`### "dashboard" layout` at line 39, etc.).

**Section heading pattern** (lines 39–66):
```markdown
### `"dashboard"` layout

[One-sentence description.]

```json
{
  "$schema": "ferro-json-ui/v2",
  ...
}
```
```

**New sections to add after `## Custom Layouts`** (or between `## Built-in Layouts` and `## Custom Layouts`):

`## fill_viewport` — explain the spec-level `fill_viewport: true` flag, what it does (internal per-pane scroll instead of whole-page scroll via `ferro-fill` CSS chain), which layouts support it (`"app"` and `"dashboard"` only — using any other causes silent whole-page scroll; lint rule `fill-viewport-layout-unknown` fires), and the required `fill: true` on the root Grid element (`register-grid-fill` lint rule).

`## Register Layout Template` — explain `register_template()` as the helper that overrides the Collect intent's display layout to `"Register"`. State: pass via `VisualContext { templates: Some(register_template()), .. }`. State what the projection emits (`fill_viewport: true` Grid + Form + TileGrid + SelectionPanel). State that the seven-intent vocabulary is unchanged (`Register` is a layout template name, not an intent). Reference the `cassa.rs` sample.

Format: use the same code block style as the rest of the file. Keep `## fill_viewport` before `## Register Layout Template` (dependency order).

---

### `docs/src/json-ui/spec-construction.md` — builder API additions section

**Analog:** `## Heterogeneous runtime construction — SpecBuilder` section (lines 109–130). This section follows the same pattern as the other four strategy sections: description paragraph + code block.

**Section pattern** (lines 109–130):
```markdown
## Heterogeneous runtime construction — `SpecBuilder`

The element graph is computed from complex domain state... [description paragraph]

```rust
use ferro::json_ui::{Spec, SpecBuilder, Element};

let spec: Spec = SpecBuilder::new()
    .title("Order detail")
    .layout("dashboard")
    .element_nested("root", Element::new("Card")
        .prop("title", "Order #1042")
        .child_nested(Element::new("Text").prop("content", "Status: confirmed"))
        .child_nested(Element::new("Button").prop("label", "Advance")))
    .build()?;
```

[Closing guidance paragraph on when to use vs. alternatives.]
```

**New subsection to add under or after the SpecBuilder section:**

`### Builder API additions`

Cover two methods added in Phase 257:
- `SpecBuilder::fill_viewport(bool) -> Self` — sets `fill_viewport` on the built Spec; `false` by default; required when the spec contains TileGrid, SelectionPanel, or Numpad (lint rule `register-fill-viewport` fires otherwise).
- `ElementBuilder::each(path, as_) -> Self` — consuming setter for the `$each` directive field; equivalent to hand-constructing `{"path": ..., "as": ...}` JSON; used in register compositions to iterate the items array for the Tile template.

Include a short Rust code block showing both methods in context (e.g., building the TileGrid element via `.each()`). Cross-link to the `$each` directive section in `expressions.md`.

---

### `Cargo.toml` + publish choreography

**Analog:** `.planning/phases/253-mcp-surface-docs-publish/253-05-PLAN.md` — the executed publish plan. Pattern is identical; Phase 258 mirrors it with adjusted version numbers and the ferro-payments rider.

**Version bump pattern** (253-05-PLAN.md, task 1 interface block):
```
Workspace version lives at Cargo.toml:46 (`[workspace.package] version = "0.2.83"` at plan time).

Version check (do not trust origin/master or memory):
  curl -s https://crates.io/api/v1/crates/ferro-rs | jq -r .crate.max_version
  git tag | grep -E "^v0\.2\.(8[0-9]|9[0-9])$"
Next version = crates.io max_version patch + 1
```

For Phase 258: the target bump is `0.2.88 → 0.2.89` (D-11). Verify `0.2.88` is actually crates.io max before bumping. `ferro-payments/Cargo.toml:3` is already at `0.1.6` (committed in `4477e394`) — do not bump it again.

**Push pattern** (253-05-PLAN.md):
```bash
# SSH is denied; always use the gh HTTPS credential helper:
git -c credential.helper='!gh auth git-credential' push https://github.com/albertogferrario/ferro.git master
git update-ref refs/remotes/origin/master HEAD
```

**Post-publish verification pattern** (253-05-PLAN.md):
```bash
curl -s https://crates.io/api/v1/crates/ferro-rs | jq -r .crate.max_version       # → 0.2.89
curl -s https://crates.io/api/v1/crates/ferro-payments | jq -r .crate.max_version  # → 0.1.6
gh api repos/albertogferrario/ferro/releases/latest --jq .tag_name                 # → v0.2.89
```

**CI-exact gate command order** (253-05-PLAN.md, interfaces block):
```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features -D warnings
```
CPU-serialized — one at a time, never chained. Re-run fmt after ANY hand-edit.

**ENOSPC prevention** (253-05-SUMMARY pattern): check `df -h` before `cargo test --all-features`. If < 10G free: `rm -rf target/debug/deps`.

**Remote divergence check before push** (253-05-PLAN.md lesson):
```bash
git fetch https://github.com/albertogferrario/ferro.git master
git log HEAD..FETCH_HEAD --oneline
# If diverged: git merge FETCH_HEAD (keep the higher Cargo.toml version)
```

**Operator gate pattern** (236/253 practice): present a pre-publish checklist with:
- gate results (fmt/clippy/test/doc all green)
- version bumps (ferro-rs 0.2.89, ferro-payments 0.1.6 rider)
- staged files list (specific files only — stray planning artifacts excluded)

Wait for explicit approval before the irreversible push.

---

## Shared Patterns

### Drift-guard test pattern
**Source:** `ferro-mcp/src/tools/generation_context.rs` lines 445–451 + `ferro-json-ui/src/runtime/mod.rs` lines 283–329
**Apply to:** All new hand-written MCP content that mirrors an authoritative registry source

The two flavors in use:
```rust
// Flavor 1: count/length assertion against a live registry (generation_context.rs:446–451)
#[test]
fn token_description_count_matches_all_tokens() {
    assert_eq!(
        DESIGN_TOKEN_DESCRIPTIONS.len(),
        ferro_theme::token::ALL_TOKENS.len(),
        "DESIGN_TOKEN_DESCRIPTIONS must have one entry per ALL_TOKENS slot (D-06 drift guard)"
    );
}

// Flavor 2: substring assertion against the assembled JS bundle (runtime/mod.rs:283–328)
#[test]
fn runtime_wires_disable_on_submit() {
    assert!(
        FERRO_RUNTIME_JS.contains("data-disable-on-submit"),
        "bundle must contain data-disable-on-submit for the double-submit guard (SC-4)"
    );
}
```

For Phase 258's `register_composition_drift_guard`, use Flavor 2's pattern:
- Assert every component name mentioned in the guidance exists in `ferro_json_ui::global_catalog()` (by name lookup against `catalog.components`).
- Assert every rule id mentioned exists in `ferro_json_ui::design::rules()` (by id lookup).
- Assert every key attribute string (`data-qty-input`, `data-filter-tokens`, etc.) appears in `FERRO_RUNTIME_JS` (same `contains()` pattern as Flavor 2).

### Serialization + population smoke test pattern
**Source:** `ferro-mcp/src/tools/generation_context.rs` lines 404–443 (`test_generation_context_has_all_sections`)
**Apply to:** Any new top-level field on `GenerationContext`

```rust
#[test]
fn test_generation_context_has_all_sections() {
    let context = execute();
    // Each field gets a non-empty check:
    assert_eq!(context.design_system.tokens.len(), 30);
    assert!(!context.design_system.intent_patterns.is_empty());
    // Pattern to follow for register_composition:
    // assert!(!context.register_composition.when_to_use.is_empty());
    // assert!(!context.register_composition.lint_rules.is_empty(), "must derive ≥1 lint rule");
}
```

### `register_template()` / `VisualContext` composition pattern
**Source:** `app/src/controllers/cassa.rs` lines 73–87
**Apply to:** All docs examples and `generation_context` prose that references the register projection path

```rust
// The canonical one-call pattern (cassa.rs:73–86):
let service = cassa_service_def();
let intents = derive_intents(&service);
let ctx = VisualContext {
    templates: Some(register_template()),
    ..Default::default()
};
let spec = JsonUiRenderer.render(&service, &intents, &ctx)
    .map_err(|e| ferro::error_response!(500, format!("projection failed: {e}")))?;
let data = serde_json::json!({ "data": { "cassa": products() } });
JsonUi::render(&spec, &data)
```

Data payload convention: rows nested under `data.{service_name}` (matching the `$each` data_path `/data/{service}`).

---

## No Analog Found

None. All six files have a direct in-tree analog.

---

## Metadata

**Analog search scope:** `ferro-mcp/src/tools/`, `ferro-json-ui/src/runtime/`, `ferro-json-ui/src/projection/`, `docs/src/json-ui/`, `app/src/controllers/`, `.planning/phases/253-mcp-surface-docs-publish/`
**Files scanned:** 10 (generation_context.rs, json_ui_catalog.rs, runtime/mod.rs, intent_layout.rs, components.md, layouts.md, spec-construction.md, cassa.rs, 253-05-PLAN.md, SUMMARY.md)
**Pattern extraction date:** 2026-07-06
