# Phase 258: MCP Surface + Docs + Publish — Research

**Researched:** 2026-07-06
**Domain:** ferro-mcp generation_context extension, json_ui_catalog audit, docs/src authoring, crates.io publish
**Confidence:** HIGH (all claims verified in-tree on `feat/billable-return-url-seam`)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**json_ui_catalog surface (POS-12) — verification-first**
- D-01: SC-1 is pre-satisfied (world-state correction 1). Work = run `test_all_components_present` + both count assertions (`catalog.rs:1296` canonical, `json_ui_catalog.rs:405` mirror) and record them as pre-existing evidence. No re-implementation, no count churn.
- D-02: Per-component design guidance for the five new components rides the 253 D-05 derived mapping (`design::rules()` registry / `RULE_COMPONENTS`). Audit the `json_ui_catalog` output for the five components; fix only additive gaps found. Any static supplement carries a drift test. All changes additive — existing output shape stays backward-compatible.

**generation_context register guidance (POS-12)**
- D-03: Content contract (SC-2, all six items): (a) when to use Register layout template vs. form-only Collect spec; (b) form-state selection contract — hidden-input quantity accumulation (`data-qty-input`), ONE confirm POST, SelectionPanel as live client-side VIEW of form state; (c) filter/numpad data attributes (`data-filter-tokens`, `data-filter-text`, numpad target-field wiring); (d) `fill_viewport` dependency + supported shell layouts (app/dashboard); (e) four `register-*` lint rule ids agents check via `design_lint`; (f) pointer to `register_template()` at `ferro-json-ui/src/projection/intent_layout.rs:50`.
- D-04: Style: compact — ids and one-liners with a pointer to `docs/src` for depth.
- D-05: Derive what is derivable (rule ids/rationale from `design::rules()`, component names from BUILTIN registry). Hand-written register prose drift-guarded: a test asserts every component name, rule id, and data attribute mentioned in the register guidance exists in its authoritative source.
- D-06: Numpad guidance documents it as an author-composable addition — NOT part of the v1 register template. Same for a standalone FilterTabs outside the TileGrid integrated strip.

**docs/src updates (POS-12)**
- D-07: Five new component sections in `docs/src/json-ui/components.md` (TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad) each with props table + at minimum one usage example, following the `Tile` section at :1411 as format anchor.
- D-08: Register projection surface documented in existing pages first (layouts.md for Register template + fill_viewport chain; spec-construction.md for builder API; components.md cross-links). New page only if warranted; if added, wire into SUMMARY.md.
- D-09: Docs cover: tap-to-add interaction model, `disable_on_submit` double-submit guard + idempotency-key pattern pointer (255 D-16/D-18), `Form` common-ancestor scoping requirement for hidden-input contract.
- D-10: mdBook docs build exits 0 (SC-3 gate). Neutral product documentation voice; no internal-strategy framing.

**Publish + gate (POS-13)**
- D-11: ONE final workspace bump **0.2.88 → 0.2.89** as the publish commit. Manual bump.
- D-12: Branch topology: `feat/billable-return-url-seam` — 140+ commits ahead of remote master; clean fast-forward. Land 258 work on this branch, then fast-forward local master **from main repo root with HEAD=master asserted** (feedback_worktree_merge_cwd_trap), then push master via gh HTTPS credential helper.
- D-13: The branch base carries ferro-payments **0.1.6** already committed (`4477e394`). Publish verification MUST confirm BOTH ferro-rs 0.2.89 AND ferro-payments 0.1.6 on crates.io.
- D-14: `ferro-a2ui` stays `publish = false` and out of `publish.yml`. No new crates → no publish.yml wave changes.
- D-15: CI-exact gate: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`, `cargo test --all-features`, `cargo doc --no-deps --all-features -D warnings`. Re-run fmt after ANY hand-edit. Serialize CPU-heavy runs. Check disk space before full test run.
- D-16: Publish step is operator-gated. Present pre-publish checklist (gate results, version bumps, what ships including ferro-payments 0.1.6 rider). Post-publish: verify via crates.io / gh API.
- D-17: gestiscilo handoff is a brief only — never edit the consumer tree.
- D-18: Stage specific files only — stray planning artifacts (phases 209/212/214/231/232/238/251/252/253 and `app/tmp/`) and the phantom `planning/phases/158-…` deletion stay OUT of Phase 258 commits.

### Claude's Discretion
- Exact docs placement within D-08 constraint and section ordering inside components.md.
- Exact `generation_context` section naming/structure for register guidance and how much rule rationale is embedded verbatim vs. trimmed.
- Whether any catalog-guidance gap found under D-02 is fixed in ferro-json-ui or ferro-mcp.
- Pre-publish checklist composition details at the D-16 gate.
- Test organization for the D-05 drift guards.

### Deferred Ideas (OUT OF SCOPE)
- Numpad in the register template (258 only documents manual composition)
- Register template knobs (pane ratios, order, search toggle)
- Sibling FilterTabs↔TileGrid pairing (`data-filter-for`)
- Category strip derivation hint
- Barcode wedge, payment flow, receipts, shift close
- v16.6 milestone archival (`/gsd-complete-milestone`) — after this phase
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| POS-12 | `json_ui_catalog` entries/count for new components, `generation_context` register composition guidance, `docs/src` updates | SC-1 pre-satisfied (count=52, all names verified); SC-2 gap confirmed (zero register content in generation_context.rs); SC-3 format anchor, props ground truth, and docs placement all researched below |
| POS-13 | `/cassa` flip to projection-derived spec, full CI-exact gate green, single crates.io publish closes the milestone | `/cassa` flip shipped in Phase 257 (world-state correction 3); this phase verifies it stands + publishes 0.2.89 |
</phase_requirements>

---

## Summary

Phase 258 closes the v16.6 POS Component Suite milestone. The scope is narrowly defined: extend `generation_context` with register composition guidance (six D-03 content items), fix additive gaps in `json_ui_catalog` per-component guidance, write five component documentation sections in `docs/src/json-ui/components.md` plus register projection surface docs in `layouts.md` and `spec-construction.md`, run the CI-exact gate, and perform the single operator-gated crates.io publish (0.2.88 → 0.2.89) that unblocks gestiscilo's register phase.

**World-state confirmed:** SC-1 is pre-satisfied — `ferro-mcp/src/tools/json_ui_catalog.rs:403` already asserts `catalog.components.len() == 52` naming all five components (TileGrid, FilterTabs, QuantityStepper, Numpad, SelectionPanel). No count or name work is needed; verification only. `generation_context.rs` is 498 lines with zero "register" content — SC-2 is the real scope. The `/cassa` projection-derived flip already shipped in Phase 257 (UAT passed).

**Two additive gaps found in `json_ui_catalog`:** (1) The `BUILDER_API` static string does not mention `fill_viewport(bool)` or `.each(path, as_)` — Phase 257 additions now absent from MCP context. (2) `RULE_COMPONENTS` maps `register-fill-viewport` to `["Grid", "TileGrid"]` but the rule's trigger set (`REGISTER_TRIGGER_TYPES`) includes `TileGrid`, `SelectionPanel`, and `Numpad` — so SelectionPanel and Numpad receive no guidance about this rule.

**Primary recommendation:** Add `register_composition: RegisterCompositionGuidance` as a new field on `GenerationContext`; populate it from derived sources (rule registry, runtime attribute contract) with drift-guard tests; fix the two BUILDER_API and RULE_COMPONENTS gaps additively; write the five component sections + register layout/builder docs; then operator-gated publish.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `generation_context` register guidance | ferro-mcp (MCP output crate) | ferro-json-ui (authoritative sources) | MCP tool owns the agent context surface; ferro-json-ui owns rule registry, attribute contract, and BUILTIN_TYPES that the guidance derives from |
| `json_ui_catalog` per-component design guidance | ferro-mcp (RULE_COMPONENTS mapping) | ferro-json-ui (design::rules() registry) | RULE_COMPONENTS lives in json_ui_catalog.rs; derivation pulls from design::rules() at runtime |
| BUILDER_API string | ferro-mcp (json_ui_catalog.rs) | ferro-json-ui (spec.rs SpecBuilder/ElementBuilder) | Static string in json_ui_catalog.rs; must mirror the public API that lives in ferro-json-ui/src/spec.rs |
| docs/src component sections | docs/src (mdBook) | ferro-json-ui (Props structs, render behavior) | Authoring surface; ground truth comes from Props structs and rustdoc in ferro-json-ui |
| register layout template docs | docs/src/json-ui/layouts.md | ferro-json-ui/src/projection/intent_layout.rs | The `register_template()` helper and `fill_viewport` chain behavior live in ferro-json-ui |
| builder API docs (`fill_viewport`, `each`) | docs/src/json-ui/spec-construction.md | ferro-json-ui/src/spec.rs | Builder methods live in spec.rs; docs extend the existing spec-construction page |
| Publish trigger | Cargo.toml (workspace version) | .github/workflows/publish.yml | Version bump commit is the CI publish trigger; no publish.yml changes needed |

---

## Standard Stack

### Already in Use (no new dependencies)

| Library | Purpose | Source |
|---------|---------|--------|
| `ferro-json-ui` | Component registry, design rules, props schemas, runtime attribute contract | `[VERIFIED: grep -rn "ferro-json-ui" ferro-mcp/Cargo.toml]` |
| `ferro-theme` | Token constants (ALL_TOKENS drift guard) | `[VERIFIED: generation_context.rs:449]` |
| `strum` | VariantArray for Variant/Tone/Size enum derivation | `[VERIFIED: generation_context.rs:228]` |
| `schemars` | Props JSON schema generation for catalog | `[VERIFIED: catalog.rs:33–38]` |
| `mdBook` | Docs build (book.toml, docs/src/) | `[VERIFIED: docs/book.toml]` |

**Installation:** No new crates. All dependencies already in Cargo.toml.

---

## generation_context Extension — Detailed Facts

### Current Structure [VERIFIED: ferro-mcp/src/tools/generation_context.rs]

`GenerationContext` currently has these top-level fields:
- `naming_conventions: NamingConventions`
- `file_structure: FileStructure`
- `common_patterns: CommonPatterns`
- `avoid: Vec<String>`
- `imports: ImportTemplates`
- `design_system: DesignSystemSummary` (added in Phase 253)

**Zero "register" content in the file.** A `grep -n "register" ferro-mcp/src/tools/generation_context.rs` returns no hits.

### What to Add

A new field `register_composition: RegisterCompositionGuidance` on `GenerationContext`. The struct contains the six D-03 content items. Everything derivable is derived; prose is drift-guarded.

```rust
// New field on GenerationContext:
pub register_composition: RegisterCompositionGuidance,

// New struct:
#[derive(Debug, Serialize)]
pub struct RegisterCompositionGuidance {
    /// When to use Register vs. form-only Collect.
    pub when_to_use: &'static str,
    /// Form-state selection contract (hidden-input accumulation, one confirm POST).
    pub form_state_contract: &'static str,
    /// Runtime data attributes for filter + numpad wiring.
    pub data_attributes: &'static [DataAttributeInfo],
    /// fill_viewport requirement and supported shell layouts.
    pub fill_viewport_requirement: &'static str,
    /// Four register-* lint rule ids derived from design::rules().
    pub lint_rules: Vec<RegisterRuleRef>,
    /// Pointer to register_template() helper.
    pub template_helper: &'static str,
}
```

### D-03 Content Items — Authoritative Facts [VERIFIED in-tree]

**(a) When to use Register vs. form-only Collect:**
- Register layout template = Collect intent + `layout: "Register"` override via `register_template()`
- Use Register when: the screen has both a browseable items pane (TileGrid) and a running-selection pane (SelectionPanel) that pins and scrolls internally
- Use plain Collect when: standard create/edit form without an adjacent selection pane

**(b) Form-state selection contract [VERIFIED: ferro-json-ui/src/projection/builder.rs:640–673]:**
- A single `Form` element (with HTML `id`, e.g. `"sale_form"`) is the common ancestor of both the TileGrid pane and the SelectionPanel pane
- TileGrid prop `form_id` and SelectionPanel prop `form_id` must both equal the Form's `id`
- Hidden inputs (`data-qty-input="{field}"`) accumulate per-tile quantity; the SelectionPanel is a live CLIENT-SIDE VIEW of that form state — it is NOT a second source of truth
- ONE confirm POST button submits the entire Form; `disable_on_submit: true` on the Button prevents double-submission
- Data contract: handler-supplied rows include `field: "qty_{id}"` (or projection-derived equivalent), `price_cents: <integer>` (integer cents, never float), and the fields Tile's `$each` template binds

**(c) Filter and numpad data attributes [VERIFIED: ferro-json-ui/src/runtime/filters.rs, tiles.rs, numpad.rs]:**

Filter runtime (`setupFilters` — Phase 255):
| Attribute | Who emits | Role |
|-----------|-----------|------|
| `data-filter-scope` | TileGrid root | Scoping container for the filter group |
| `data-filter-tab="<token>"` | FilterTabs items | Filter tab; empty value = "All" |
| `data-filter-search` | TileGrid search input | Optional text search input |
| `data-filter-text="<name>"` | Tile root | Search source (emitted from `Tile.name`) |
| `data-filter-tokens="t1 t2 ..."` | Tile root | Space-separated category tokens (spaces in names normalize to hyphens) |

Tile quantity runtime (`setupTiles` — Phase 254/255):
| Attribute | Who emits | Role |
|-----------|-----------|------|
| `data-qty-inc="{field}"` | Tile tap button | Increments the named hidden input |
| `data-qty-dec="{field}"` | QuantityStepper − button | Decrements the named hidden input |
| `data-qty-input="{field}"` | Hidden input | The form field the runtime writes to |
| `data-qty-display="{field}"` | SelectionPanel line count | Display element updated on change |
| `data-qty-step`, `data-qty-min`, `data-qty-max` | QuantityStepper | Stepper bounds/step size |
| `data-unit-price` | Tile root | Integer cents for SelectionPanel running total |
| `data-selection-form="{form_id}"` | TileGrid root + SelectionPanel root | Pairing attribute |
| `data-selection-remove` | SelectionPanel remove button | Sets qty=0 (client-side removal) |

Numpad runtime (`setupNumpad` — Phase 255):
| Attribute | Who emits | Role |
|-----------|-----------|------|
| `data-numpad` | Numpad root | Container selector |
| `data-numpad-target="<field>"` | Numpad root | Names the hidden input to update |
| `data-numpad-mode="price|quantity"` | Numpad root (from `NumpadProps.mode`) | Entry mode (default: quantity) |
| `data-numpad-display` | Display element inside container | Shows current value |
| `data-numpad-key="0".."9"|"backspace"|"clear"` | Key buttons | Digit/action keys |
| `data-numpad-input="<field>"` | Hidden input matching target | Written on each tap |

Form guard (`setupFormGuards` — Phase 255):
| Attribute | Who emits | Role |
|-----------|-----------|------|
| `data-disable-on-submit` | Confirm Button (when `disable_on_submit: true`) | Disables all non-qty buttons on form submit |

**(d) fill_viewport dependency [VERIFIED: design/rules.rs, render output]:**
- `fill_viewport: true` at the Spec level is REQUIRED when a spec contains TileGrid, SelectionPanel, or Numpad (lint rule `register-fill-viewport`)
- The root Grid MUST have `fill: true` (lint rule `register-grid-fill`)
- The supported shell layouts for `fill_viewport` are `"app"` and `"dashboard"` ONLY — the `ferro-fill` CSS chain supports only these two. Using any other layout causes silent whole-page scroll (lint rule `fill-viewport-layout-unknown`)
- `register_template()` emits layout `"dashboard"` and `fill_viewport: true` in the projection path

**(e) Four register-* lint rule ids [VERIFIED: ferro-json-ui/src/design/rules.rs:85–111]:**
| Rule ID | Fires when |
|---------|------------|
| `register-fill-viewport` | TileGrid/SelectionPanel/Numpad present but `fill_viewport` not set |
| `register-grid-fill` | `fill_viewport: true` but root Grid lacks `fill: true` |
| `register-selection-present` | TileGrid present but no SelectionPanel anywhere |
| `fill-viewport-layout-unknown` | `fill_viewport: true` but layout is not `"app"` or `"dashboard"` |

**(f) register_template() helper [VERIFIED: ferro-json-ui/src/projection/intent_layout.rs:50–66]:**
```rust
// ferro-json-ui/src/projection/intent_layout.rs:50
pub fn register_template() -> ThemeTemplates
```
Returns a `ThemeTemplates` that overrides the Collect intent's display layout to `"Register"`. Pass via `VisualContext { templates: Some(register_template()), .. }` — the built-in `default_template(Intent::Collect)` remains Form (existing Collect projections unaffected).

The projection-derived `/cassa` sample in `app/src/controllers/cassa.rs` is the reference composition.

### D-05 Drift Guard Pattern

The drift-guard test for the register guidance section must assert:
1. Every component name mentioned in the guidance (TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad, Tile) exists in `ferro_json_ui::global_catalog()` (BUILTIN_TYPES)
2. Every rule id mentioned (`register-fill-viewport`, `register-grid-fill`, `register-selection-present`, `fill-viewport-layout-unknown`) exists in `ferro_json_ui::design::rules()` (RULE_REGISTRY)
3. Key attribute names (`data-qty-input`, `data-filter-tokens`, `data-filter-text`, `data-numpad-target`, `data-disable-on-submit`) appear in `FERRO_RUNTIME_JS` bundle (the same pattern used in `runtime/mod.rs:285–322`)

The test follows the 253 D-09 pattern: `assert!(FERRO_RUNTIME_JS.contains("data-numpad-target"), ...)`.

---

## json_ui_catalog Audit — Additive Gaps Found [VERIFIED: ferro-mcp/src/tools/json_ui_catalog.rs]

### SC-1 Status: Pre-Satisfied

`json_ui_catalog.rs:403–408` already asserts `catalog.components.len() == 52` with the error message naming all five components. All five names appear in the expected list at :411–464. No work needed beyond recording this as verification evidence.

### Gap 1: BUILDER_API string missing Phase 257 additions

The `BUILDER_API` static string at lines 347–370 currently shows:

```
Spec::builder() -> SpecBuilder
  .title(...) .layout(...) .data(...) .element(id, Element) .build()
Element::new(type_name) -> ElementBuilder
  .prop(...) .child(...) .action(...) .visible(...)
```

**Missing from the string:**
- `SpecBuilder::fill_viewport(bool) -> Self` (added in Phase 257, `spec.rs:359`)
- `ElementBuilder::each(path: impl Into<String>, as_: impl Into<String>) -> Self` (added in Phase 257, `spec.rs:471`)

**Fix:** Extend the `BUILDER_API` const string additively. These are the only two Phase 257 builder additions.

### Gap 2: RULE_COMPONENTS misses SelectionPanel/Numpad for register-fill-viewport

Current mapping at `json_ui_catalog.rs:97`:
```rust
("register-fill-viewport", &["Grid", "TileGrid"]),
```

The rule's trigger set `REGISTER_TRIGGER_TYPES` in `rules.rs:443` is `&["TileGrid", "SelectionPanel", "Numpad"]`. SelectionPanel and Numpad both trigger `register-fill-viewport` but receive zero guidance about it in the catalog.

**Fix:** Add SelectionPanel and Numpad to the `register-fill-viewport` entry additively:
```rust
("register-fill-viewport", &["Grid", "TileGrid", "SelectionPanel", "Numpad"]),
```

### Gap 3: fill-viewport-layout-unknown has no component entries

Current mapping at `json_ui_catalog.rs:103`:
```rust
("fill-viewport-layout-unknown", &[]),
```

No component receives guidance about this rule. The most natural home is Grid (the root element in register compositions). Consider adding `["Grid"]` so agents authoring a fill_viewport spec get the layout constraint surfaced.

**Fix (Claude's discretion):** Either add `&["Grid"]` or leave `&[]` if the rationale is that this is a spec-level rule rather than a component rule. Either choice should be documented in code comments.

### Gap 4: FilterTabs and QuantityStepper have zero design rule guidance

Neither FilterTabs nor QuantityStepper appears in any RULE_COMPONENTS entry. These components have no specific lint rules that fire on their presence (by design — they are composable atoms). No fix required; record as "by design" in the research.

### Existing drift-guard tests that cover the above

`json_ui_catalog.rs:738–790` (`design_system_component_guidance_drift_guarded`) already:
- Asserts every RULE_COMPONENTS rule id exists in `design::rules()` (Direction 1)
- Asserts every rule id in `design::rules()` is mapped in RULE_COMPONENTS (Direction 2)
- Asserts every component in RULE_COMPONENTS is a valid builtin (Direction 3)

Adding entries to RULE_COMPONENTS is automatically covered by these tests.

---

## Props Ground Truth [VERIFIED: ferro-json-ui/src/component.rs:1412–1529]

### Tile (existing section in docs, verify for drift)

Props confirmed unchanged in Phases 256/257 (no new fields added post-254):
| Prop | Type | Required |
|------|------|----------|
| `item_id` | `string` | yes |
| `name` | `string` | yes |
| `price` | `string` | yes |
| `field` | `string` | yes |
| `default_quantity` | `number \| null` | no |
| `categories` | `string[]` | no |
| `image_url` | `string \| null` | no |
| `color` | `tone \| null` | no |
| `stock_badge` | `string \| null` | no |
| `price_cents` | `number \| null` | no |

### TileGrid (new docs section)
| Prop | Type | Required | Notes |
|------|------|----------|-------|
| `data_path` | `string` | yes | JSON pointer to the items array iterated via `$each` |
| `form_id` | `string` | yes | HTML `id` of the Form element owning this grid's hidden inputs; emitted as `data-selection-form` |
| `categories_path` | `string \| null` | no | JSON pointer to a string array for the integrated category strip; absent → no strip |
| `columns` | `number \| null` | no | Base viewport column count (default: 2) |
| `search` | `boolean \| null` | no | Enable client-side text search input |
| `search_placeholder` | `string \| null` | no | Placeholder for search input (default: "Search"); ignored when `search` absent/false |
| `all_label` | `string \| null` | no | "Show all" tab label for the integrated strip (default: "All"); ignored when `categories_path` absent |

### SelectionPanel (new docs section)
| Prop | Type | Required | Notes |
|------|------|----------|-------|
| `form_id` | `string` | yes | Must match the paired TileGrid `form_id`; emitted as `data-selection-form` |
| `empty_message` | `string \| null` | no | Placeholder text when panel has no line items |
| `currency` | `string \| null` | no | Currency symbol (e.g. `"€"`) prepended to the integer-cents total; no locale tables |
| `total_label` | `string \| null` | no | Running-total row label (default: "Total") |

### FilterTabs (new docs section)
| Prop | Type | Required | Notes |
|------|------|----------|-------|
| `items` | `string[]` | no (default `[]`) | Category labels rendered as filter tabs; may be `$data`-bound |
| `all_label` | `string \| null` | no | "Show all" tab label (default: "All") |

Zero-prop FilterTabs renders as an All-only strip.

### QuantityStepper (new docs section)
| Prop | Type | Required | Notes |
|------|------|----------|-------|
| `field` | `string` | yes | Name of the hidden input this stepper drives via `data-qty-inc`/`data-qty-dec` |
| `min` | `number \| null` | no | Lower bound (default: 0) |
| `max` | `number \| null` | no | Upper bound; unbounded when absent |
| `step` | `number \| null` | no | Increment size (default: 1) |

### Numpad (new docs section)
| Prop | Type | Required | Notes |
|------|------|----------|-------|
| `target_field` | `string` | yes | Name of the input this numpad writes into; emitted as `data-numpad-target` |
| `mode` | `"quantity" \| "price"` | no (default: `"quantity"`) | `quantity` = integer entry; `price` = two-decimal-place monetary entry |

---

## Docs Placement [VERIFIED: docs/src/]

### docs/src/SUMMARY.md current structure

JSON-UI chapter (lines 63–73):
```
- [Getting Started](json-ui/getting-started.md)
- [Components](json-ui/components.md)
- [Actions](json-ui/actions.md)
- [Data Binding & Visibility](json-ui/data-binding.md)
- [Form Validation](json-ui/forms.md)
- [Layouts](json-ui/layouts.md)
- [Plugins](json-ui/plugins.md)
- [Runtime Primitives](json-ui/runtime-primitives.md)
- [Spec construction](./json-ui/spec-construction.md)
- [Expressions](json-ui/expressions.md)
- [JSON Schema](json-ui/json-schema.md)
```

No new pages required if the planner extends existing files. **Critical constraint:** `create-missing = false` in `docs/book.toml` — any new page listed in SUMMARY.md MUST have the corresponding file or `mdbook build` fails.

### components.md placement

File is 1673 lines. Section structure:
- Line 116: `## Layout Components` (Grid, etc.)
- Line 773: `## Form Components`
- Line 1182: `## Navigation Components`
- Line 1409: `## Commerce Components` — contains `### Tile` at :1411
- Line 1445: `## Kanban Components` — contains KanbanBoard, KanbanColumn
- Line 1530: Extensible Components (RawHtml, StreamText, etc.)

**Placement recommendation:** Extend `## Commerce Components` (after the Tile section, before `## Kanban Components`). Add `### TileGrid`, `### SelectionPanel`, `### FilterTabs`, `### QuantityStepper`, `### Numpad` in that order. Each section follows the Tile format anchor: description paragraph, props table, JSON usage example, notes paragraph.

### layouts.md additions [VERIFIED: docs/src/json-ui/layouts.md — no register/fill_viewport content]

Current layouts.md covers `dashboard`, `app`, `auth`, omit, and Custom Layouts. No `fill_viewport` or Register template content exists.

**Add two new subsections:**

1. `## fill_viewport` — explain the `fill_viewport: true` spec-level flag, what it does (internal scroll per pane instead of whole-page scroll), which shell layouts support it (`"app"`, `"dashboard"` only), and the required `fill: true` on the root Grid.

2. `## Register Layout Template` — explain `register_template()` as the ready-made helper that overrides Collect → Register layout, how to pass it via `VisualContext`, what the projection emits (fill_viewport Grid + Form + SelectionPanel + TileGrid), and that the seven-intent vocabulary is unchanged (Register is a layout template name, not an intent).

### spec-construction.md additions [VERIFIED: docs/src/json-ui/spec-construction.md — no fill_viewport/each builder content]

Current spec-construction.md covers the four strategies (static, $each, $if, SpecBuilder) and the decision rubric. The `SpecBuilder` section shows old API without Phase 257 additions.

**Add a new subsection** under the `SpecBuilder` section (or alongside it):

`### Builder API additions (Phase 257)`

Cover:
- `SpecBuilder::fill_viewport(bool) -> Self` — sets `fill_viewport` on the built Spec; default is `false`; required for register layouts
- `ElementBuilder::each(path: impl Into<String>, as_: impl Into<String>) -> Self` — public consuming setter over the `$each` directive field; replaces manual JSON construction of the `{"path": ..., "as": ...}` object; used in register compositions to iterate the items array for Tile templates

---

## Publish Mechanics [VERIFIED: 253-05-PLAN.md + 253-05-SUMMARY.md]

This phase mirrors the 253-05 publish choreography exactly. Key lessons from 253:

### CI-exact gate commands (in order, never parallelized)
```bash
cargo fmt --all -- --check
cargo clippy --all --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features -D warnings
```

The CI workflow at `.github/workflows/ci.yml:72` runs `cargo doc --no-deps --all-features`.

### Known ENOSPC risk
253-05-SUMMARY documented that `target/debug/deps` grew to 6.3G and hit ENOSPC during `cargo test --all-features`. Check `df -h` before running tests. If space is low: `rm -rf target/debug/deps` (safe, rebuilt on demand) frees 5–7G.

### Schema-export drift
`cargo test --all-features` regenerates `docs/protocol/schemas/*.json`. In Phase 253 these files changed (Track A CRUD fields). In Phase 258 no new ServiceDef fields are expected — discard the regen churn unless a real diff appears.

### Remote divergence risk
During Phase 253's gate-review pause, the remote received a CI auto-bump to 0.2.84. Before pushing, always fetch remote and check for divergence:
```bash
git fetch https://github.com/albertogferrario/ferro.git master
git log HEAD..FETCH_HEAD --oneline
```
If diverged: `git merge FETCH_HEAD` (resolve Cargo.toml version conflict, keeping the higher local version).

### Version determination
```bash
curl -s https://crates.io/api/v1/crates/ferro-rs | jq -r .crate.max_version
git tag | grep -E "^v0\.2\."
```
Next version = max(crates.io max_version, current Cargo.toml) + 1 patch. Do not trust local `origin/master` refs.

### Push command (SSH denied; HTTPS only)
```bash
git -c credential.helper='!gh auth git-credential' push https://github.com/albertogferrario/ferro.git master
git update-ref refs/remotes/origin/master HEAD
```

### Post-publish verification (both crates)
```bash
curl -s https://crates.io/api/v1/crates/ferro-rs | jq -r .crate.max_version         # must == 0.2.89
curl -s https://crates.io/api/v1/crates/ferro-payments | jq -r .crate.max_version    # must == 0.1.6
gh api repos/albertogferrario/ferro/releases/latest --jq .tag_name                   # must == v0.2.89
```

### publish.yml: no changes needed
Wave structure confirmed:
- Wave 1a: `ferro-json-ui` (leaf crate, no internal deps)
- Wave 1b: `ferro-stripe`, `ferro-projections`, etc.
- Wave 1c: `ferro-payments`
- Wave 2: `ferro-rs ferro-mcp ferro-mcp-oauth ferro-mcp-server`
- Wave 3: `ferro-cli`

`ferro-a2ui` is `publish = false` (intentionally absent). No new crates → no wave changes needed.

---

## Common Pitfalls

### Pitfall 1: Claiming SC-1 as Phase 258 work
**What goes wrong:** The plan or executor re-implements the count assertion or names work on it.
**Why it happens:** World-state correction 1 (CONTEXT.md) states SC-1 is pre-satisfied, but it's easy to miss this.
**How to avoid:** The first task MUST be to run `cargo test -p ferro-mcp -- test_all_components_present` and record the result as "pre-existing evidence" — not as work completed by this phase.

### Pitfall 2: Schema-export churn included in phase commit
**What goes wrong:** `cargo test --all-features` regenerates `docs/protocol/schemas/*.json`; these get staged and committed.
**Why it happens:** Standard `git add` includes them; 253-05-SUMMARY shows this happened in 253.
**How to avoid:** After `cargo test`, run `git diff docs/protocol/schemas/` — commit only if real content changed (not just formatting). In Phase 258 no new ServiceDef fields are expected.

### Pitfall 3: BUILDER_API string treated as requiring a prose rewrite vs. additive extend
**What goes wrong:** The BUILDER_API string is rewritten, breaking existing agents that parse its format.
**Why it happens:** It's a static string, easy to replace wholesale.
**How to avoid:** Append `fill_viewport(bool) -> Self` after the existing `.build()` line for SpecBuilder, and `.each(path, as_) -> Self` after `.visible()` for ElementBuilder. Add a test asserting both strings appear.

### Pitfall 4: mdBook build fails due to missing page file
**What goes wrong:** A new page is added to SUMMARY.md but the file doesn't exist.
**Why it happens:** `book.toml` sets `create-missing = false` — mdBook will fail rather than create placeholder files.
**How to avoid:** Either don't add new pages (extend existing files per D-08), or create the file before wiring SUMMARY.md.

### Pitfall 5: Remote divergence during gate-review pause
**What goes wrong:** Remote master advances (CI auto-bump from independent push) while the operator reviews. Push is rejected.
**Why it happens:** crates.io has CI auto-bump patterns that can fire on any remote push.
**How to avoid:** Always fetch and check for divergence immediately before pushing. See exact commands in Publish Mechanics above.

### Pitfall 6: Disk-full (ENOSPC) blocking `cargo test --all-features`
**What goes wrong:** Test run fails with ENOSPC, not a real test failure.
**Why it happens:** `target/debug/deps` grows to 5–7G and the data volume is near capacity.
**How to avoid:** `df -h` before running the test gate. If <10G free: `rm -rf target/debug/deps`.

### Pitfall 7: Using `origin/master` refs to check crates.io version
**What goes wrong:** Published version appears to be 0.2.86 or another stale value.
**Why it happens:** Local `origin/master` refs are chronically stale on this repo (per project memory).
**How to avoid:** Always verify version via `curl -s https://crates.io/api/v1/crates/ferro-rs | jq -r .crate.max_version`.

### Pitfall 8: Forgetting to verify ferro-payments 0.1.6
**What goes wrong:** Only ferro-rs 0.2.89 is verified; ferro-payments 0.1.6 is already bumped in-tree but unverified on crates.io.
**Why it happens:** ferro-payments is independently versioned; easy to treat it as a rider and skip verification.
**How to avoid:** D-13 explicitly requires confirming both. The wave 1c publish step publishes ferro-payments.

---

## Code Examples

### Adding register_composition to GenerationContext [ASSUMED — pattern extrapolated from the 253 design_system field addition]

```rust
// In generation_context.rs — new struct:
#[derive(Debug, Serialize)]
pub struct RegisterCompositionGuidance {
    pub when_to_use: &'static str,
    pub form_state_contract: &'static str,
    pub data_attributes: &'static [DataAttributeInfo],
    pub fill_viewport_requirement: &'static str,
    pub lint_rules: Vec<RegisterRuleRef>,   // derived from design_rules()
    pub template_helper: &'static str,
}

// In execute() — derive lint_rules from design_rules() (same pattern as intent_patterns):
let register_rule_ids = ["register-fill-viewport", "register-grid-fill",
                          "register-selection-present", "fill-viewport-layout-unknown"];
let lint_rules: Vec<RegisterRuleRef> = design_rules()
    .iter()
    .filter(|r| register_rule_ids.contains(&r.id))
    .map(|r| RegisterRuleRef { id: r.id, title: r.title, rationale: r.rationale })
    .collect();
```

### Extending BUILDER_API string [VERIFIED: pattern from json_ui_catalog.rs:347]

```rust
// Additive addition to the BUILDER_API const:
// After "  .build() -> Result<Spec, SpecError>" in SpecBuilder block:
"  .fill_viewport(bool) -> Self  (sets fill_viewport on the built Spec; required for register layouts)\n"

// After "  .visible(Visibility) -> Self" in ElementBuilder block:
"  .each(path, as_) -> Self  (emit one element per row in the data array; see $each directive)\n"
```

### TileGrid usage example for docs

```json
"tiles": {
  "type": "TileGrid",
  "props": {
    "data_path": "/data/products",
    "form_id": "sale_form",
    "search": true,
    "columns": 3
  },
  "children": ["tile_tmpl"]
}
```

### SelectionPanel usage example for docs

```json
"cart": {
  "type": "SelectionPanel",
  "props": {
    "form_id": "sale_form",
    "currency": "€",
    "empty_message": "No items selected"
  },
  "children": ["confirm_btn"]
}
```

### Numpad usage example for docs

```json
"keypad": {
  "type": "Numpad",
  "props": {
    "target_field": "amount",
    "mode": "price"
  }
}
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`#[cfg(test)]`) |
| Config file | None (integrated into crate) |
| Quick run — mcp only | `cargo test -p ferro-mcp` |
| Quick run — json-ui only | `cargo test -p ferro-json-ui` |
| Full suite | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| POS-12 (SC-1) | catalog count=52, all five names present | unit | `cargo test -p ferro-mcp -- test_all_components_present` | YES (json_ui_catalog.rs:396) |
| POS-12 (SC-2) | generation_context has register guidance | unit | `cargo test -p ferro-mcp -- test_generation_context_has_all_sections` | YES (generation_context.rs:404) — currently doesn't check register field; must be extended |
| POS-12 (SC-2 drift) | register component names, rule ids, attrs are in authoritative sources | unit | `cargo test -p ferro-mcp -- register_composition_drift_guard` | NO — Wave 0 gap |
| POS-12 (D-02 gaps) | BUILDER_API mentions fill_viewport/each | unit | `cargo test -p ferro-mcp -- builder_api_mentions_fill_viewport` | NO — Wave 0 gap |
| POS-12 (D-02 gaps) | RULE_COMPONENTS updated (SelectionPanel/Numpad in register-fill-viewport) | unit | `cargo test -p ferro-mcp -- design_system_component_guidance_drift_guarded` | YES (json_ui_catalog.rs:738) — auto-covered after RULE_COMPONENTS update |
| POS-12 (SC-3) | docs/src build exits 0 | build | `mdbook build docs/` | YES (book.toml exists) |
| POS-13 | CI-exact gate green | integration | (full gate commands above) | — |
| POS-13 | publish verified on crates.io | external verify | curl + gh API commands | — |

### Sampling Rate
- Per task commit: `cargo fmt --all -- --check && cargo test -p ferro-mcp`
- Per wave merge: full CI-exact gate
- Phase gate: full suite green before operator approval + publish push

### Wave 0 Gaps

- [ ] `ferro-mcp/src/tools/generation_context.rs` — extend `test_generation_context_has_all_sections` to assert `register_composition` field is non-empty; add `register_composition_drift_guard` test
- [ ] `ferro-mcp/src/tools/json_ui_catalog.rs` — add `builder_api_mentions_fill_viewport` and `builder_api_mentions_each` assertions
- [ ] Five component sections in `docs/src/json-ui/components.md` (TileGrid, SelectionPanel, FilterTabs, QuantityStepper, Numpad)
- [ ] Register layout template section in `docs/src/json-ui/layouts.md`
- [ ] Builder API additions section in `docs/src/json-ui/spec-construction.md`

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The struct field approach for `register_composition` (adding a new top-level field to `GenerationContext`) is the right extension point | generation_context Extension | Low risk — the 253 pattern for `design_system` field is identical; the planner can choose an alternative struct layout |
| A2 | `fill-viewport-layout-unknown` should map to `["Grid"]` in RULE_COMPONENTS | json_ui_catalog Audit | Low risk — Claude's discretion per D-02; leaving `[]` is also valid |

---

## Open Questions

1. **Should `fill-viewport-layout-unknown` map to `["Grid"]` in RULE_COMPONENTS?**
   - What we know: Currently `&[]` — no component gets this rule guidance
   - What's unclear: Whether this is intentional (spec-level rule) or an oversight
   - Recommendation: Add `["Grid"]` as Grid is the required root element in register compositions; the rule fires when fill_viewport + non-supported layout is combined on any spec

2. **Does `cassa.rs` reference `register_template()` by the exact function path, making it a reliable cross-link target for docs?**
   - What we know: The function exists at `intent_layout.rs:50` with rustdoc; the cassa controller calls it
   - What's unclear: Whether the controller's exact call site style is the right docs example or whether a cleaner snippet should be written
   - Recommendation: Lift the controller code as the docs example; it's already lint-clean and UAT-passed

---

## Environment Availability

Step 2.6: SKIPPED — Phase 258 is code/docs/publish changes with no new external tools. The only external dependency is the crates.io API (verified reachable via curl in Phase 253).

---

## Sources

### Primary (HIGH confidence)
- `ferro-mcp/src/tools/generation_context.rs` — confirmed 498 lines, zero register content, struct field layout for extension
- `ferro-mcp/src/tools/json_ui_catalog.rs:81–104` — RULE_COMPONENTS mapping (gaps verified)
- `ferro-mcp/src/tools/json_ui_catalog.rs:347–370` — BUILDER_API string (fill_viewport/each gap verified)
- `ferro-mcp/src/tools/json_ui_catalog.rs:403–468` — SC-1 count assertion pre-verified
- `ferro-json-ui/src/component.rs:1412–1529` — TileGridProps, SelectionPanelProps, FilterTabsProps, QuantityStepperProps, NumpadProps (all fields verified)
- `ferro-json-ui/src/design/rules.rs:85–111` — four register-* rule ids, rationale text, trigger types
- `ferro-json-ui/src/projection/intent_layout.rs:50–66` — `register_template()` function signature and behavior
- `ferro-json-ui/src/runtime/tiles.rs` — data-qty-* attribute vocabulary
- `ferro-json-ui/src/runtime/numpad.rs` — data-numpad-* attribute vocabulary
- `ferro-json-ui/src/runtime/filters.rs` — data-filter-* attribute vocabulary
- `ferro-json-ui/src/runtime/form_guards.rs` — data-disable-on-submit
- `docs/src/json-ui/components.md:1409–1441` — Tile section format anchor + Commerce Components placement
- `docs/src/json-ui/layouts.md` — confirmed no register/fill_viewport content; add locations identified
- `docs/src/json-ui/spec-construction.md` — confirmed no fill_viewport/each builder content
- `docs/src/SUMMARY.md` — chapter structure verified; no new pages required for D-08 baseline
- `docs/book.toml` — `create-missing = false` confirmed (CRITICAL constraint)
- `.planning/phases/253-mcp-surface-docs-publish/253-05-PLAN.md` — publish choreography source
- `.planning/phases/253-mcp-surface-docs-publish/253-05-SUMMARY.md` — publish execution record (ENOSPC, remote divergence lessons)
- `.github/workflows/ci.yml:72` — CI docs job confirmed as `cargo doc --no-deps --all-features`
- `Cargo.toml:47` — workspace version `0.2.88` confirmed
- `ferro-payments/Cargo.toml:3` — ferro-payments `0.1.6` confirmed

### Secondary (MEDIUM confidence)
- `ferro-json-ui/src/projection/builder.rs:640–733` — emit_register_root form_id convention (`"sale_form"`) and Tile `$each` binding pattern

---

## Metadata

**Confidence breakdown:**
- SC-1 catalog count: HIGH — verified in-tree
- generation_context gaps: HIGH — zero register content confirmed by direct inspection
- json_ui_catalog gaps: HIGH — BUILDER_API and RULE_COMPONENTS gaps confirmed by direct inspection
- Props ground truth: HIGH — verified from component.rs struct fields
- Runtime attribute vocabulary: HIGH — verified from runtime/*.rs sources
- Docs placement: HIGH — verified from current docs structure
- Publish mechanics: HIGH — verified from 253-05 plan/summary execution record

**Research date:** 2026-07-06
**Valid until:** 2026-08-06 (stable codebase; CI/publish mechanics are stable)
