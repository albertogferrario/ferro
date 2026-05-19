# Phase 117: Catalog & JSON Schema - Research

**Researched:** 2026-04-18
**Domain:** JSON Schema derivation, compiled validation, machine-readable component catalogs for SDUI
**Confidence:** HIGH (downstream constraints and upstream code are fully known; external dependencies verified)

## Summary

Phase 117 replaces `COMPONENT_CATALOG` — a 4.5 KB hand-maintained Markdown const in `ferro-json-ui/src/lib.rs` (lines 88–174) — with a machine-readable `Catalog` struct that derives per-component JSON Schema from existing `#[derive(JsonSchema)]` on every `*Props` struct (Phase 115 shipped those derives). The Catalog pre-computes five artifacts at build time: (1) `components: HashMap<String, ComponentSpec>`, (2) `plugin_components: HashMap<String, ComponentSpec>`, (3) `per_component_schemas: HashMap<String, Value>`, (4) `full_schema: Value` (Spec envelope with `oneOf` over every component), and (5) `validator: jsonschema::Validator` compiled once from the full schema. It exposes `prompt()` (≤ 8 KB text for LLM context), `json_schema()` (full document for external tooling), `component_schema(name)` (Props-only for targeted AI generation), and `validate(&Spec)` (two-stage: pre-dispatch on `type_name` then full schema validation).

Two structural challenges drive every design decision. **First**, the 39-variant `oneOf` in the full spec schema is linear-time worst-case inside `jsonschema` — the crate does not optimize discriminated unions. The fix is a pre-dispatch pass that checks `el.type_name ∈ (BUILTIN_TYPES ∪ plugin_registry)` in O(1) before invoking the full validator, collapsing the hot path. **Second**, the schema validator must be compiled once at first access (not per-request) and the 30+ `schema_for!` invocations must run once at Catalog::build time and be cached — 30 `validator_for` compilations per request would dominate the request budget.

**Primary recommendation:** Ship the single-file `catalog.rs` (≤ 1200 LOC) with the static `BUILTIN_SPECS` table + two-stage validator + hand-assembled `oneOf`. Pin `jsonschema = "0.46"` (current stable), NOT 0.28 as CONTEXT D-09 speculates — 0.46 is what the ecosystem is on and the API is stable. Plan the work as 7 plans (117-01 through 117-07), with Plan 03 (full-schema assembly) and Plan 04 (validation pipeline) being the two architecturally interesting ones; the others are mechanical.

## Architectural Responsibility Map

Phase 117 is single-tier (pure Rust library + CLI subcommand); the tier map is thin but worth stating explicitly so the planner does not over-decompose.

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Component schema derivation | `ferro-json-ui` lib | — | schemars derives are already on the Props structs; catalog is a reflection layer |
| Plugin schema ingestion | `ferro-json-ui` lib | `JsonUiPlugin` trait | Plugin authors provide `props_schema() -> Value`; Catalog consumes |
| Full spec schema assembly | `ferro-json-ui` lib | — | Hand-assembled `oneOf` — no framework dep |
| Spec validation (Props) | `ferro-json-ui` lib | `jsonschema` crate | Compiled validator stored on Catalog |
| CLI schema export | `ferro-cli` → `framework` binary | `ferro-json-ui` | Shell-out to `cargo run -- json-ui:schema`; subcommand dispatches to `global_catalog().json_schema()` |
| MCP tool wiring | `ferro-mcp` tools | `ferro-json-ui` | Rewire `json_ui_catalog.rs`, `json_ui_generate.rs`, `json_ui_inspect.rs` to consume `global_catalog()` |

## User Constraints (from CONTEXT.md)

### Locked Decisions (35)

**D-01: Crate location.** Catalog lives in `ferro-json-ui/src/catalog.rs`. Same crate as renderer, Props, `BUILTIN_TYPES`. No new crate.

**D-02: Public API.** `pub use catalog::{Catalog, CatalogError, ComponentSpec, global_catalog};` in `lib.rs`.

**D-03: Eager build.** `Catalog` struct pre-computes `components`, `plugin_components`, `full_schema`, `per_component_schemas`, `validator`.

**D-04: `OnceLock<Catalog>` global singleton.** `pub fn global_catalog() -> &'static Catalog`. Plugin state at first-access time is frozen; late-registered plugins do NOT propagate. No hot-swap.

**D-05: Static `BUILTIN_SPECS` table.** `&[(type_name, description, schema_fn, slot_fields)]`. No macro — auditable, grep-friendly.

**D-06: Descriptions inline.** Authored in `BUILTIN_SPECS`, one-sentence imperative, matching today's `COMPONENT_CATALOG` voice.

**D-07: Drift guard.** Assert `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` at `build()` + unit test; mismatch panics with clear message.

**D-08: Plugin discovery.** `global_plugin_registry().read()` iteration; `is_plugin = true`; generic description fallback; `slot_fields = &[]`.

**D-09: `jsonschema` crate.** `jsonschema = "0.28"` — **Research note: the current stable is 0.46; see Risks section. Planner should confirm version at Plan 01 time.**

**D-10: Two-stage validation.** (1) `type_name` whitelist (O(1) HashMap lookup), (2) per-element Props validation via cached `per_component_schema`, (3) optional full-spec validation.

**D-11: `CatalogError` variants** with `thiserror`: `UnknownType`, `PropsInvalid`, `SpecInvalid`, `BuildFailed(String)`, `SchemaSerialization(#[from] serde_json::Error)`.

**D-12: Validator reuse.** Full spec validator compiled once at `build()`. Per-component validators compiled on-demand in `validate()` for Phase 117; escape hatch to precompile into `HashMap<String, Validator>` if profiling shows overhead > 1 ms × N elements.

**D-13: Full schema shape.** Root has `$schema` / `root` / `elements` (HashMap of `Element` refs). `$defs` holds `Element`, `Action`, `Visibility`. `Element.props` is a `oneOf` over per-component Props schemas each with `"type": { "const": "X" }` pinned.

**D-14: Hand-assemble `oneOf`.** ~40 LOC iteration over `BUILTIN_SPECS + plugin_components`. Deterministic output order.

**D-15: Cache assembled schema.** `catalog.full_schema: Value`; `json_schema() -> &Value`.

**D-16: `prompt()` format.** Markdown-like text matching today's `COMPONENT_CATALOG` style (`### ComponentName\nDescription\nProps: ...`).

**D-17: Prompt size budget.** ≤ 8 KB soft target; overflow logged, not errored.

**D-18: Deterministic sort.** `BUILTIN_SPECS + plugin_components` sorted by name.

**D-19: Per-component export.** `component_schema(name) -> Option<&Value>` returns Props schema only (NOT wrapped in Element).

**D-20: Plugin schemas opaque.** Phase 117 does NOT meta-validate plugin schemas; malformed schemas surface as `BuildFailed` during `build()`.

**D-21: CLI subcommand.** `ferro json-ui:schema` with `--output <path>`, `--pretty`, `--component <name>`.

**D-22: Binary dispatch.** `framework/src/app.rs` (NOT `framework/src/bin/ferro.rs` — does not exist; see Canonical Refs) adds the `json-ui:schema` subcommand arm.

**D-23: Delete `COMPONENT_CATALOG`.** `ferro-json-ui/src/lib.rs` lines 88–174 removed; consumers use `global_catalog().prompt()`.

**D-24: ferro-mcp `json_ui_catalog.rs` rewrite.** Public struct shape preserved (`JsonUiCatalog { components, plugin_components, builder_api, action_api }`); body pulls from `global_catalog()`; hand-maintained `BUILDER_API` / `ACTION_API` strings stay.

**D-25: ferro-mcp `json_ui_generate.rs`.** System prompt source `COMPONENT_CATALOG` → `global_catalog().prompt()`.

**D-26: ferro-cli `make_json_view.rs` / `ai.rs`.** Grep confirmed — only `ferro-cli/src/ai.rs` references `COMPONENT_CATALOG` (line 7 import + line 103 interpolation); `make_json_view.rs` has zero hits.

**D-27: Inline unit tests.** Catalog build, every BUILTIN_TYPES present, schema shape, prompt length, component_schema isolation.

**D-28: Positive validation tests.** Minimal valid Element per type passes `validate`.

**D-29: Negative validation tests.** Each CatalogError variant has a dedicated failing fixture.

**D-30: Integration test.** Phase 115 fixture round-trips through `catalog.validate`.

**D-31: Slot-ID graph validation NOT in scope.** Semantic check of `CardProps.footer[i] ∈ spec.elements` deferred to Phase 117.5 or a follow-up.

**D-32–D-35: Out of scope.** Spec/Element shape frozen (115). Walker stays catalog-unaware (116). `$data`/`$template` is Phase 118. No plugin hot-swap.

### Claude's Discretion

- Single `catalog.rs` vs. `catalog/` dir split — start single-file, split above ~1200 LOC.
- Prompt sort (alphabetical vs. atoms/containers/form/data bucketed) — pick what reads cleaner.
- Per-component validator caching strategy — start on-demand, upgrade if profiling demands.
- `jsonschema` version within 0.x.y range — research says 0.46; CONTEXT says 0.28. Planner chooses (see Risks).
- `Catalog::build()` returning `Result` vs. panicking — CONTEXT recommends `Result` for clean CLI failure reporting.
- File split of `BUILTIN_SPECS` — inline in `catalog.rs` OR separate `catalog/builtin_specs.rs` if it grows large (~40 entries with inline descriptions is ~200 LOC — keep inline).

### Deferred Ideas (OUT OF SCOPE)

- Phase 117.5 slot-ID graph validation.
- Per-component validator precompilation HashMap (CONTEXT D-12 escape hatch).
- Runtime plugin hot-swap / Catalog rebuild.
- Plugin schema meta-validation (is the plugin's schema itself valid JSON Schema).
- Catalog diff tool for release notes.
- IDE plugin / LSP consuming exported JSON Schema.
- Schema `$id` URL hosting.
- Two-tier AI generation (Phase 120).
- Docs rewrite (Phase 121).

## Phase Requirements

From ROADMAP §"Phase 117: Catalog & JSON Schema". REQUIREMENTS.md does not enumerate these IDs explicitly — the 8 ROADMAP success criteria are the enforceable contract.

| ID | Description | Research Support |
|----|-------------|------------------|
| CAT-01 | `Catalog::build()` auto-discovers all Component variants with descriptions and JSON Schema per props struct | Props Inventory table + `BUILTIN_SPECS` design in §2 + plugin discovery via `global_plugin_registry()` |
| CAT-02 | `catalog.prompt()` generates concise text system prompt — NOT raw JSON Schema (too large for AI context) | §7 Prompt Generation Algorithm — 8 KB budget, Markdown format matching today's `COMPONENT_CATALOG` voice |
| CAT-03 | `catalog.validate(&spec)` returns typed errors; pre-dispatches by `"type"` string before full schema validation | §5 Pre-Dispatch Validation Flow + `CatalogError` variants in §4 |
| CAT-04 | `catalog.component_schema("Card")` returns per-component Props schema (NOT wrapped in Element) | §4 jsonschema crate audit + D-19 locked; per-component schemas live in `per_component_schemas: HashMap<String, Value>` |
| SCHEMA-01 | `catalog.json_schema()` exports complete spec schema document (root + elements + `oneOf`) | §3 Full Spec Schema Structure — hand-assembled oneOf skeleton + `$defs/{Element,Action,Visibility}` |
| SCHEMA-02 | `ferro json-ui:schema` CLI exports schema (stdout or file) | §8 CLI Command Wiring — `db_status.rs` pattern + `framework/src/app.rs` subcommand |
| SCHEMA-03 | Schema validator compiled once in `Catalog::build()`, reused — no per-validation compilation | §6 Per-Component Schema Caching Strategy — full-spec validator cached; per-component on-demand with escape hatch |
| (implied) | `COMPONENT_CATALOG` replaced by `catalog.prompt()` output | §9 Consumer Migration Plan — delete const, rewire `ferro-mcp/tools/json_ui_generate.rs` + `ferro-cli/src/ai.rs` |

## 1. Problem Framing

**What's hard here (two structural challenges):**

### (a) Assembling a discriminated-union `oneOf` from 30+ Props schemas

`schemars` produces a schema for each `*Props` type in isolation. Nothing in the schemars output connects them into a `oneOf` where each variant is pinned by a type discriminator. Phase 117 must hand-assemble that `oneOf`:

- Iterate every `(type_name, props_schema)` pair in `BUILTIN_SPECS + plugin_components`.
- For each, produce `{ "allOf": [ {"properties": {"type": {"const": "X"}}}, <props_schema> ] }` OR (cleaner) nest the discriminator alongside the Props schema's `properties` map.
- Assemble all into `$defs/Element.properties.props.oneOf: [ … ]`.

This is the idiomatic JSON Schema pattern for sum types (e.g., OpenAPI's `discriminator`, jsonschema crate accepts it, all modern validators handle it). It's ~40 LOC of deterministic iteration but must be stable — sort by name, preserve exact schema shape so diffs are meaningful.

### (b) Validator-compilation cost budget

Naive approach — call `jsonschema::validator_for(&full_schema)` inside every `validate()` invocation — compiles 30+ sub-schemas per request. With the cached-once approach (CONTEXT D-12):

- Full-spec validator: compiled once at `build()` time, stored on Catalog. Cost amortized over lifetime.
- Per-component validators: Phase 117 compiles on-demand in the two-stage path (see §5). Each is small (single Props schema, ~50 LOC of JSON). If real-world profiling shows compile cost × N elements > 1 ms, upgrade to `HashMap<String, Validator>` precompiled at build.

The validation path must short-circuit — fail fast on type_name whitelist before attempting full-schema validation, which is the expensive case. This is what makes the two-stage design non-negotiable.

**8 success criteria (enforceable):**

1. `Catalog::build()` auto-discovers all Component variants with descriptions and JSON Schema per Props struct. [VERIFIED: ROADMAP §117 SC-1]
2. `catalog.prompt()` generates concise text (NOT raw JSON Schema). [VERIFIED: ROADMAP §117 SC-2]
3. `catalog.validate(&spec)` — typed errors for unknown types, invalid props, missing required fields; pre-dispatches by `"type"` string before full schema validation. [VERIFIED: ROADMAP §117 SC-3]
4. `catalog.json_schema()` exports the complete JSON Schema document. [VERIFIED: ROADMAP §117 SC-4]
5. `catalog.component_schema("Card")` returns JSON Schema for single component's props. [VERIFIED: ROADMAP §117 SC-5]
6. `ferro json-ui:schema` CLI command exports schema to stdout or file. [VERIFIED: ROADMAP §117 SC-6]
7. `COMPONENT_CATALOG` const string is replaced by `catalog.prompt()` output. [VERIFIED: ROADMAP §117 SC-7]
8. Schema validator compiled once in `Catalog::build()`, reused. [VERIFIED: ROADMAP §117 SC-8]

## 2. Props Inventory

All 39 built-in types from `ferro-json-ui/src/render/mod.rs` lines 41–85, paired with the matching Props struct from `ferro-json-ui/src/component.rs` and slot fields from Phase 116.

| # | type_name | Props struct | Slot fields | One-sentence description (author) |
|---|-----------|--------------|-------------|-----------------------------------|
| 1 | `Text` | `TextProps` | `[]` | Semantic text element (p / h1 / h2 / h3 / span / div / section). |
| 2 | `Button` | `ButtonProps` | `[]` | Interactive button with variant, size, optional icon, and disabled state. |
| 3 | `Badge` | `BadgeProps` | `[]` | Small variant-styled label. |
| 4 | `Alert` | `AlertProps` | `[]` | Inline notice with info / success / warning / error variants. |
| 5 | `Separator` | `SeparatorProps` | `[]` | Horizontal or vertical divider between content sections. |
| 6 | `Progress` | `ProgressProps` | `[]` | Progress bar with 0–100 percentage value and optional label. |
| 7 | `Avatar` | `AvatarProps` | `[]` | Circular user image with fallback initials and size variants. |
| 8 | `Image` | `ImageProps` | `[]` | Image with optional aspect ratio and skeleton fallback on load error. |
| 9 | `Skeleton` | `SkeletonProps` | `[]` | Loading placeholder with configurable width / height / rounding. |
| 10 | `Breadcrumb` | `BreadcrumbProps` | `[]` | Navigation trail of label + optional URL items. |
| 11 | `Pagination` | `PaginationProps` | `[]` | Page navigation for paginated data (current / per_page / total). |
| 12 | `DescriptionList` | `DescriptionListProps` | `[]` | Key-value pairs displayed as a description list with optional format. |
| 13 | `EmptyState` | `EmptyStateProps` | `[]` | Standardized empty view with title, description, and optional CTA. |
| 14 | `StatCard` | `StatCardProps` | `[]` | Live-updatable metric card with label, value, icon, SSE target. |
| 15 | `Checklist` | `ChecklistProps` | `[]` | Onboarding-style checklist with dismissal and server-side state. |
| 16 | `Toast` | `ToastProps` | `[]` | Declarative notification intent consumed by the runtime JS via data attributes. |
| 17 | `NotificationDropdown` | `NotificationDropdownProps` | `[]` | Dropdown listing notification items with icons, timestamps, read state. |
| 18 | `Sidebar` | `SidebarProps` | `[]` | Dashboard sidebar with fixed top / bottom items and collapsible nav groups. |
| 19 | `Header` | `HeaderProps` | `[]` | Dashboard top bar with business name, notification badge, user menu. |
| 20 | `DropdownMenu` | `DropdownMenuProps` | `[]` | Trigger button with an absolutely-positioned kebab-style action panel. |
| 21 | `CalendarCell` | `CalendarCellProps` | `[]` | Single day in a month grid with today highlight, out-of-month muting, event dots. |
| 22 | `ActionCard` | `ActionCardProps` | `[]` | Clickable row with icon, title, description, chevron, and variant-colored border. |
| 23 | `ProductTile` | `ProductTileProps` | `[]` | Touch-friendly POS tile with name, price, and +/- quantity controls. |
| 24 | `Card` | `CardProps` | `["footer"]` | Content container with title, description, body children, and optional footer slot. |
| 25 | `Modal` | `ModalProps` | `["footer"]` | Dialog overlay with title, description, body children, and optional footer slot. |
| 26 | `Tabs` | `TabsProps` | `[]` (slots live per-tab inside `TabsProps.tabs[i].children`) | Tabbed content; per-tab children live in `TabsProps.tabs[i].children`. |
| 27 | `KanbanBoard` | `KanbanBoardProps` | `[]` (slots live per-column inside `KanbanBoardProps.columns[i].children`) | Horizontally scrollable kanban columns on desktop, tab-switched on mobile. |
| 28 | `PageHeader` | `PageHeaderProps` | `["actions"]` | Page title with optional breadcrumb and action button slot. |
| 29 | `Grid` | `GridProps` | `[]` (uses `Element.children`) | Responsive multi-column grid with configurable breakpoint columns, gap, scroll. |
| 30 | `Collapsible` | `CollapsibleProps` | `[]` (uses `Element.children`) | Expandable `<details>` / `<summary>` section. |
| 31 | `FormSection` | `FormSectionProps` | `[]` (uses `Element.children`) | Visual grouping within a form with title, description, and layout variant. |
| 32 | `ButtonGroup` | `ButtonGroupProps` | `[]` (uses `Element.children`) | Horizontal button row with configurable gap. |
| 33 | `Form` | `FormProps` | `[]` (fields come from `Element.children`) | Form container with action binding and field components. |
| 34 | `Input` | `InputProps` | `[]` | Text input with type variants, validation error, data_path pre-fill. |
| 35 | `Select` | `SelectProps` | `[]` | Dropdown select with options, error, data_path pre-fill. |
| 36 | `Checkbox` | `CheckboxProps` | `[]` | Boolean checkbox with label, description, data binding. |
| 37 | `Switch` | `SwitchProps` | `[]` | Toggle switch (visual alternative to Checkbox); auto-submit when `action` set. |
| 38 | `Table` | `TableProps` | `[]` | Data table with columns, row_actions, sorting, empty_message. |
| 39 | `DataTable` | `DataTableProps` | `[]` | Stripe-style alternating-row table with per-row DropdownMenu and mobile card fallback. |

**Drift guard note:** `render/mod.rs::builtin_types_count_matches_dispatch` asserts `BUILTIN_TYPES.len() == 39`. `BUILTIN_SPECS.len() == 39` must match. Plan 01 adds this as a `const _: () = …` compile-time assertion if possible, or a `#[test]` at minimum.

**Tab / KanbanBoard slot handling:** `Tab.children` and `KanbanColumnProps.children` are slot fields on nested types, NOT on the outer Props. The `slot_fields` column in `BUILTIN_SPECS` is for the OUTER Props only. Tab children and Kanban column children are still slots but they're discovered by the walker at the nested-type level. Document this in Plan 02: `slot_fields` doesn't need entries for nested-type slots — the prompt / json_schema must mention them via the nested Tab / KanbanColumnProps schemas already being embedded in the full `oneOf`.

Sources:
- [VERIFIED: `ferro-json-ui/src/component.rs`] — Props struct list and field definitions
- [VERIFIED: `ferro-json-ui/src/render/mod.rs::BUILTIN_TYPES`] — canonical type_name list (39 entries)
- [VERIFIED: `ferro-json-ui/src/lib.rs::COMPONENT_CATALOG`] — authorial voice for descriptions

## 3. Full Spec Schema Structure

Concrete skeleton `Catalog::build()` assembles. Draft 2020-12 (matches schemars 1.x default per [CITED: https://graham.cool/schemars/]).

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "ferro-json-ui/v2",
  "type": "object",
  "required": ["$schema", "root", "elements"],
  "properties": {
    "$schema":  { "const": "ferro-json-ui/v2" },
    "root":     { "type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_-]{0,127}$" },
    "elements": {
      "type": "object",
      "additionalProperties": { "$ref": "#/$defs/Element" }
    },
    "title":    { "type": ["string", "null"] },
    "layout":   { "type": ["string", "null"] },
    "data":     true
  },
  "$defs": {
    "Element": {
      "type": "object",
      "required": ["type"],
      "properties": {
        "type":     { "type": "string" },
        "props":    { "oneOf": [ /* 39 built-in variants + N plugin variants */ ] },
        "children": { "type": "array", "items": { "type": "string" } },
        "action":   { "$ref": "#/$defs/Action" },
        "visible":  { "$ref": "#/$defs/Visibility" }
      }
    },
    "Action":     { /* serde_json::to_value(schema_for!(Action)) */ },
    "Visibility": { /* serde_json::to_value(schema_for!(Visibility)) */ }
  }
}
```

**Each `oneOf` variant** is assembled from the per-component Props schema with a sibling discriminator:

```json
{
  "allOf": [
    {
      "type": "object",
      "required": ["type"],
      "properties": { "type": { "const": "Card" } }
    },
    { /* schema_for!(CardProps) — just the schema, no wrapper */ }
  ]
}
```

Alternative (cleaner, prefer this): merge the type constraint into the existing Props schema's `properties` map:

```json
{
  "type": "object",
  "required": ["type", "title"],  /* unions CardProps required + ["type"] */
  "properties": {
    "type":        { "const": "Card" },
    "title":       { "type": "string" },
    "description": { "type": ["string", "null"] },
    /* … rest of CardProps schema properties … */
  }
}
```

The merge approach keeps the schema lean (no nested `allOf`) but requires careful merge logic. Plan 03 chooses — CONTEXT D-13 shows the `allOf` variant; the merge variant is smaller output. Either satisfies downstream tooling.

**Action and Visibility via schemars.** Both `Action` and `Visibility` carry `#[derive(JsonSchema)]` [VERIFIED: `ferro-json-ui/src/action.rs:10`, `ferro-json-ui/src/visibility.rs:8`]. Fetch with:

```rust
let action_schema     = serde_json::to_value(schemars::schema_for!(Action))?;
let visibility_schema = serde_json::to_value(schemars::schema_for!(Visibility))?;
```

Extract the root schema object and place under `$defs/Action` / `$defs/Visibility`. Note that `schema_for!` produces a root schema with `$schema` / `$id` / `definitions` — you want just the object shape, so extract `.get("properties")`, `.get("required")`, `.get("type")` or consume the whole root and delete the meta-keys.

**Schema-normalization gotchas:**

- schemars 1.x default draft is 2020-12 [CITED: https://graham.cool/schemars/]. If output references `definitions` vs. `$defs`, normalize to `$defs` at assembly time.
- schemars wraps enums with `oneOf` over `const` variants already — that's desired; keep as-is.
- `#[serde(rename_all = "snake_case")]` on enums produces lowercase-snake in the schema; this matches every ferro Props enum.
- `Option<T>` produces `{ "type": ["T", "null"] }` in 2020-12. Matches expected shape.
- `#[serde(skip_serializing_if = ...)]` doesn't appear in the schema (schemars only sees serde's presence requirement — field may or may not be required depending on `#[serde(default)]`).

Sources:
- [VERIFIED: `ferro-json-ui/src/spec.rs`] — Spec/Element field shape
- [CITED: https://graham.cool/schemars/] — schemars 1.x uses Draft 2020-12 by default
- [CITED: https://docs.rs/jsonschema/latest/jsonschema/] — jsonschema crate accepts 2020-12 schemas

## 4. jsonschema Crate Audit

**Current stable version:** `jsonschema = "0.46"` [VERIFIED: crates.io via `cargo search jsonschema` 2026-04-18]. CONTEXT D-09 specifies `0.28` — this is a **research correction**: the CONTEXT was written against outdated training data. Use 0.46 (see Risks §11 for migration notes).

**Key API surface** (from [CITED: https://docs.rs/jsonschema/latest/jsonschema/]):

```rust
// Build a validator from a schema. Auto-detects draft.
pub fn validator_for(schema: &serde_json::Value) -> Result<Validator, ReferencingError>;

// Draft 2020-12 explicit variant, if pinning is desired.
pub mod draft202012 {
    pub fn new(schema: &Value) -> Result<Validator, ReferencingError>;
    pub fn is_valid(schema: &Value, instance: &Value) -> bool;
    pub fn validate(schema: &Value, instance: &Value) -> Result<(), ValidationError>;
    pub mod meta { pub fn is_valid(schema: &Value) -> bool; }
}

impl Validator {
    pub fn is_valid(&self, instance: &Value) -> bool;
    pub fn validate(&self, instance: &Value) -> Result<(), ValidationError>;
    pub fn iter_errors<'i>(&'i self, instance: &'i Value) -> impl Iterator<Item = ValidationError<'i>>;
    pub fn evaluate(&self, instance: &Value) -> OutputFormat;  // detailed structural diagnostic
}
```

**Error iteration pattern** (multi-error per instance):

```rust
for error in validator.iter_errors(&instance_value) {
    errors.push(CatalogError::PropsInvalid {
        element_id: id.clone(),
        type_name: el.type_name.clone(),
        errors: vec![format!("{}: {}", error.instance_path(), error)],
    });
}
```

**Feature flags:** Default features include resolvers for `http`, `https`, `file` references. For Phase 117 (schemas are all in-process, no remote refs), `default-features = false` is safer and smaller. Revisit if remote `$ref` resolution is ever needed.

**Pinned version recommendation:** `jsonschema = { version = "0.46", default-features = false }`. No specific Cargo feature flag needed — the base package supports Draft 2020-12 via `draft202012` module and auto-detection via `validator_for`.

**`oneOf` + `const` discriminator quirks:** Documentation does not mention optimization of discriminated unions [CITED: docs.rs fetch, 2026-04-18]. The linear sub-schema check is a real cost — this validates the pre-dispatch approach in §5.

Sources:
- [VERIFIED: crates.io via cargo search] — current version 0.46.0
- [CITED: https://docs.rs/jsonschema/latest/jsonschema/] — API surface
- [CITED: https://github.com/Stranger6667/jsonschema] — maintained repo (Stranger6667)

## 5. Pre-Dispatch Validation Flow

Concrete pseudo-code for `catalog.validate(&spec) -> Result<(), Vec<CatalogError>>`. Follows CONTEXT D-10.

```rust
impl Catalog {
    pub fn validate(&self, spec: &Spec) -> Result<(), Vec<CatalogError>> {
        let mut errors = Vec::new();

        // ── Stage 1: type_name whitelist (O(n), O(1) per element) ──
        // This collapses the oneOf worst case. Every type_name must resolve to
        // either a built-in or a registered plugin.
        for (id, el) in &spec.elements {
            let is_builtin = self.components.contains_key(&el.type_name);
            let is_plugin  = self.plugin_components.contains_key(&el.type_name);
            if !is_builtin && !is_plugin {
                errors.push(CatalogError::UnknownType {
                    element_id: id.clone(),
                    type_name: el.type_name.clone(),
                });
            }
        }
        // Fast-fail: if a type_name is unknown, don't try per-element Props
        // validation — we'd fail with noisy "oneOf has no match" errors anyway.
        if !errors.is_empty() {
            return Err(errors);
        }

        // ── Stage 2: per-element Props validation (O(n), small schema) ──
        // For each element, look up its per-component Props schema and validate
        // just the props Value against it. We do NOT wrap in Element here — we
        // want isolated Props errors, not envelope errors.
        for (id, el) in &spec.elements {
            if let Some(schema) = self.per_component_schemas.get(&el.type_name) {
                // Per-component Validator is compiled on-demand in Phase 117
                // (CONTEXT D-12). Upgrade to cached HashMap if profiling demands.
                let validator = match jsonschema::validator_for(schema) {
                    Ok(v) => v,
                    Err(e) => {
                        errors.push(CatalogError::BuildFailed(format!(
                            "compiling per-component schema for '{}': {}",
                            el.type_name, e
                        )));
                        continue;
                    }
                };
                for err in validator.iter_errors(&el.props) {
                    errors.push(CatalogError::PropsInvalid {
                        element_id: id.clone(),
                        type_name: el.type_name.clone(),
                        errors: vec![format!("{}: {}", err.instance_path(), err)],
                    });
                }
            }
            // If the type_name is a plugin and we don't have a schema, skip
            // per-element validation — plugin schemas are opaque (D-20).
        }

        // ── Stage 3 (optional): full spec envelope validation ──
        // Catches $schema mismatch, root missing from elements, malformed
        // top-level structure. The compiled full-schema validator is reused.
        let spec_value = serde_json::to_value(spec)
            .map_err(|e| vec![CatalogError::SchemaSerialization(e)])?;
        for err in self.validator.iter_errors(&spec_value) {
            errors.push(CatalogError::SpecInvalid {
                errors: vec![format!("{}: {}", err.instance_path(), err)],
            });
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

**Edge cases** (Plan 04 tests):

- **Empty spec** (`elements` empty, `root` points at missing ID): Stage 1 passes (no elements to iterate); Stage 3 fails with `SpecInvalid` because `root` doesn't resolve (the full schema includes this check via `additionalProperties` on `elements`… actually no, it doesn't — full schema only checks structural shape, not the "root exists in elements" semantic constraint). Phase 115's `Spec::from_json` catches this structurally. **Decision:** Document that `validate` is layered ON TOP of `Spec::from_json` — callers that already parsed via `from_json` are guaranteed root-exists + no-dangling + no-cycles. `catalog.validate` adds type-name + Props-shape + envelope validation.
- **Element with `props: null`**: `Element.props` is `serde_json::Value` with `#[serde(default, skip_serializing_if = "serde_json::Value::is_null")]`. A null props value will fail per-component Props schema validation if required fields exist. Stage 2 surfaces this as `PropsInvalid`.
- **Plugin-only spec (no built-ins)**: Stage 1 passes; Stage 2 skips per-component validation (plugin schemas are opaque); Stage 3 runs full-schema validation — the `oneOf` must include plugin variants assembled at `build()` time. This is why Plugin schemas get woven into the full-schema `oneOf` in Plan 03.
- **Duplicate type discriminator in `oneOf`**: Can't happen if Stage 1 passes (type_name is unique-per-element, `oneOf` matches exactly one variant by const).

## 6. Per-Component Schema Caching Strategy

**Phase 117 starts on-demand (CONTEXT D-12):**

```rust
// In validate() — compiled fresh each time this element validates.
let validator = jsonschema::validator_for(schema)?;
```

**Benchmark target:** <1 ms per element of validator-compile overhead. Each per-component schema is small (50–200 LOC of JSON), so compile should be well under 1 ms. At 10–50 elements per spec, total overhead is 10–50 ms — acceptable for Phase 117.

**Escape hatch** (if profiling shows > 1 ms × N):

```rust
pub struct Catalog {
    // … existing fields …
    per_component_validators: HashMap<String, jsonschema::Validator>,  // precompiled
}

impl Catalog {
    pub fn build() -> Result<Self, CatalogError> {
        // … existing build logic …
        let mut per_component_validators = HashMap::new();
        for (name, schema) in &per_component_schemas {
            let v = jsonschema::validator_for(schema)
                .map_err(|e| CatalogError::BuildFailed(format!("{name}: {e}")))?;
            per_component_validators.insert(name.clone(), v);
        }
        // … rest of build …
    }
}
```

This moves 30 compilations from per-request to once-at-startup — same total CPU but amortized. Trade-off: Catalog size grows (compiled validators retain internal state). Recommend: Plan 04 implements on-demand; a follow-up task measures real-world cost and decides.

**Why not eagerly precompile in Phase 117:** KISS principle + CONTEXT D-12 says "simple, correct." The full-spec validator IS cached (that's the expensive one). Per-component on-demand is cheap.

## 7. Prompt Generation Algorithm

**Format** — derived from today's `COMPONENT_CATALOG` [VERIFIED: `ferro-json-ui/src/lib.rs:88-174`]:

```markdown
## Component Catalog

### Text
Semantic text element (p / h1 / h2 / h3 / span / div / section).
Props: content (String), element (p|h1|h2|h3|span|div|section, default p)

### Button
Interactive button with variant, size, optional icon, and disabled state.
Props: label (String), variant (default|secondary|destructive|outline|ghost|link, default default), size (xs|sm|default|lg, default default), disabled (Option<bool>), icon (Option<String>), icon_position (Option<left|right>), button_type (Option<button|submit>)

### Card
Content container with title, description, body children, and optional footer slot.
Props: title (String), description (Option<String>), max_width (Option<default|narrow|wide>), footer (Vec<String> of element IDs — see Slots below)
Slots: footer (Vec<String>) — body children come from Element.children.

### Tabs
Tabbed content; per-tab children live in TabsProps.tabs[i].children.
Props: default_tab (String), tabs (Vec<Tab {value: String, label: String, children: Vec<String> of element IDs}>)
```

**Sorting order:** Alphabetical is simplest and deterministic (CONTEXT D-18 requires deterministic). Bucketed (atoms / containers / form / data) reads better for humans. Recommend: alphabetical within buckets:

```
## Atoms
### Alert
### Avatar
### Badge
…
## Containers
### ButtonGroup
### Card
…
## Form Controls
### Checkbox
### Input
…
## Data Displays
### DataTable
### Table
```

Buckets derived from `BUILTIN_TYPES` comments (`// Leaves`, `// Containers`, `// Form controls`, `// Data displays`). Plan 05 chooses — both are deterministic, both fit the 8 KB budget.

**Enum handling:** Inline variants when count ≤ 8:

- `ButtonVariant` (6 variants) → inline: `variant (default|secondary|destructive|outline|ghost|link)`
- `InputType` (11 variants) → still inline (well under the 8 KB budget for the whole prompt)
- `ActionCardVariant` (3 variants) → inline
- If any enum grows past 8–10 variants in the future, emit `variant (one of N — see schema)`

**Slot-field documentation:** Explicit. For CardProps:
```
Props: title (String), description (Option<String>), max_width (Option<default|narrow|wide>), footer (Vec<String> of element IDs)
Body children come from Element.children. Footer IDs live in CardProps.footer, not in Element.children.
```

This tells the LLM: slot IDs live in typed Props, NOT in `element.children`. Critical for Phase 120 generation correctness.

**Size budget ≤ 8 KB (CONTEXT D-17):** Measure in Plan 05 tests. Current `COMPONENT_CATALOG` is 4.5 KB with 23 hand-authored entries. 39 auto-generated entries at similar verbosity should land ~6–8 KB. If it exceeds 8 KB, log a warning (not an error).

**Deterministic output:** Sort keys; sort enum variants (within their serde-derived order, which is declaration order — stable across schemars versions).

## 8. CLI Command Wiring

### Binary subcommand (unified `framework/src/app.rs`)

The unified binary is NOT at `framework/src/bin/ferro.rs` (that path does not exist). It's in `framework/src/app.rs` — the user's app declares `Application::new()...run().await` which parses CLI via `clap`. Subcommands are added to the `Commands` enum at lines 43–83.

**Add:**

```rust
// In framework/src/app.rs Commands enum (after DbSeed):
/// Export the JSON-UI v2 schema (full spec schema or a single component)
#[command(name = "json-ui:schema")]
JsonUiSchema {
    /// Write to file instead of stdout
    #[arg(long, short = 'o')]
    output: Option<String>,

    /// Pretty-print JSON output (default: compact)
    #[arg(long)]
    pretty: bool,

    /// Export only the Props schema for a single component (e.g., "Card")
    #[arg(long)]
    component: Option<String>,
},
```

**In the `match cli.command` arm (after `DbSeed`):**

```rust
Some(Commands::JsonUiSchema { output, pretty, component }) => {
    Self::run_json_ui_schema(output, pretty, component).await;
}
```

**Handler:**

```rust
async fn run_json_ui_schema(
    output: Option<String>,
    pretty: bool,
    component: Option<String>,
) {
    use ferro_json_ui::global_catalog;

    // global_catalog() returns &'static Catalog, panics on BuildFailed
    // per CONTEXT D-04 design. For CLI UX we want graceful error — wrap
    // in a catch_unwind or have global_catalog() return Result. Per
    // CONTEXT discretion, build() returns Result; the global wrapper
    // panics on first access. The CLI handler should probe via a local
    // Catalog::build() call so it can surface errors cleanly.

    let catalog = match ferro_json_ui::Catalog::build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error building catalog: {e}");
            std::process::exit(1);
        }
    };

    let value: &serde_json::Value = match &component {
        Some(name) => match catalog.component_schema(name) {
            Some(v) => v,
            None => {
                eprintln!("error: unknown component '{name}'");
                std::process::exit(1);
            }
        },
        None => catalog.json_schema(),
    };

    let serialized = if pretty {
        serde_json::to_string_pretty(value).expect("schema serializes")
    } else {
        serde_json::to_string(value).expect("schema serializes")
    };

    match output {
        Some(path) => {
            std::fs::write(&path, serialized).unwrap_or_else(|e| {
                eprintln!("error writing to {path}: {e}");
                std::process::exit(1);
            });
        }
        None => println!("{serialized}"),
    }
}
```

**Dispatch note:** `global_catalog()` vs. `Catalog::build()` — the CLI should favor a local `build()` call so errors surface as exit codes, not panics. In production handlers, `global_catalog()` is the normal path.

### ferro-cli shell-out command (`ferro-cli/src/commands/json_ui_schema.rs`)

Follows `db_status.rs` pattern [VERIFIED: `ferro-cli/src/commands/db_status.rs`]:

```rust
use console::style;
use std::path::Path;
use std::process::Command;

pub fn run(output: Option<String>, pretty: bool, component: Option<String>) {
    // Check we're in a Ferro project
    if !Path::new("Cargo.toml").exists() {
        eprintln!(
            "{} This command must be run from a Ferro project root",
            style("Error:").red().bold()
        );
        std::process::exit(1);
    }

    let mut args = vec!["run".to_string(), "--quiet".to_string(), "--".to_string(), "json-ui:schema".to_string()];
    if let Some(o) = &output  { args.push("--output".into()); args.push(o.clone()); }
    if pretty                 { args.push("--pretty".into()); }
    if let Some(c) = &component { args.push("--component".into()); args.push(c.clone()); }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("Failed to execute cargo command");

    if !status.success() {
        eprintln!("{} Schema export failed", style("Error:").red().bold());
        std::process::exit(1);
    }
}
```

**And in `ferro-cli/src/main.rs`** add the clap variant and dispatch:

```rust
/// Export the JSON-UI v2 schema (full or single component)
#[command(name = "json-ui:schema")]
JsonUiSchema {
    #[arg(long, short = 'o')]
    output: Option<String>,
    #[arg(long)]
    pretty: bool,
    #[arg(long)]
    component: Option<String>,
},
```

With dispatch: `Commands::JsonUiSchema { output, pretty, component } => commands::json_ui_schema::run(output, pretty, component),`

## 9. Consumer Migration Plan

Concrete per-file diffs for Plan 06.

### `ferro-json-ui/src/lib.rs`

Delete lines 88–174 entirely (the `COMPONENT_CATALOG` const). Add after the existing `spec::*` re-export block:

```rust
pub use catalog::{Catalog, CatalogError, ComponentSpec, global_catalog};
```

Add module declaration at the `pub mod` block:

```rust
pub mod catalog;
```

### `ferro-mcp/src/tools/json_ui_catalog.rs`

Rewrite body. Public API shape preserved (per CONTEXT D-24 — `JsonUiCatalog { components, plugin_components, builder_api, action_api }`). Replace the two hand-maintained `build_catalog()` and `build_plugin_catalog()` functions with wrappers over `ferro_json_ui::global_catalog()`:

```rust
pub fn execute(component: Option<&str>) -> JsonUiCatalog {
    let cat = ferro_json_ui::global_catalog();

    let to_catalog_component = |spec: &ComponentSpec| -> CatalogComponent {
        CatalogComponent {
            name: spec.name.clone(),
            description: spec.description.clone(),
            props: derive_prop_infos_from_schema(&spec.props_schema),
            variants: derive_variants_from_schema(&spec.props_schema),
        }
    };

    let all: Vec<CatalogComponent> = cat.components_sorted().map(to_catalog_component).collect();
    let all_plugins: Vec<CatalogComponent> = cat.plugin_components_sorted().map(to_catalog_component).collect();

    // filter logic per CONTEXT D-24 unchanged
    // …
}
```

The `BUILDER_API` and `ACTION_API` const strings stay — they document DSL idioms (CONTEXT D-24).

`derive_prop_infos_from_schema` is a new helper that walks `spec.props_schema["properties"]` to produce `Vec<PropInfo>`. It reads `required: Vec<String>` array from the schema to set `required: bool` per field; pulls `description` from schema's `description` field; pulls type name from `type`/`enum`/`$ref`. Test extensively — this is the bridge between schemars output and the hand-authored `PropInfo` shape. Estimated ~80 LOC.

`derive_variants_from_schema` walks enum variants when the schema is a `oneOf` of `const` strings.

**Alternative:** Since CONTEXT preserves the public struct shape, you could also just populate `name + description + empty props + None variants` and let MCP callers consume `prompt()` or `json_schema()` for detail. Plan 06 picks — full parity with today's detail is probably what callers expect; invest in the helpers.

### `ferro-mcp/src/tools/json_ui_generate.rs`

Line 6: `use ferro_json_ui::COMPONENT_CATALOG;` → `use ferro_json_ui::global_catalog;`

Line 103 (inside the const string): `{COMPONENT_CATALOG}` → `{catalog_prompt}` where `catalog_prompt` is pulled from `global_catalog().prompt()` at call-site into a local variable:

```rust
let catalog_prompt = global_catalog().prompt();
let system_prompt = format!(
    "… {catalog_prompt} …",
    catalog_prompt = catalog_prompt,
);
```

Note: the current file uses a `const` string with `{COMPONENT_CATALOG}` embedded — that can't interpolate a runtime value. You must convert to runtime `format!()`. Audit the exact structure during Plan 06 — it may require pulling the template into a `fn view_generation_prompt() -> String`.

### `ferro-mcp/src/tools/json_ui_inspect.rs`

**No migration required.** This file has its own local `BUILTIN_TYPES` const (lines 57–78) that is stale — only 20 entries, pre-Phase 116. It's used by v1 regex-based scanning (clearly marked `TODO(Phase 120)` at line 6). Phase 117 should NOT touch this — it's Phase 120's surface.

**Optional improvement** (Claude's discretion): the `ComponentSchemaInfo` struct returns `catalog_entry: Option<super::json_ui_catalog::CatalogComponent>` — it will automatically pick up the rewired data once `json_ui_catalog.rs` is migrated. No change needed inside `json_ui_inspect.rs` itself.

### `ferro-cli/src/ai.rs`

Line 7: `use ferro_json_ui::COMPONENT_CATALOG;` → `use ferro_json_ui::global_catalog;`

Line 103 (system prompt f-string interpolates `{COMPONENT_CATALOG}`): replace with `{global_catalog().prompt()}` — but note that f-strings in Rust don't evaluate expressions, only identifiers. Must assign to a local first:

```rust
let catalog_prompt = global_catalog().prompt();
let prompt = format!(
    "You are generating a Ferro JSON-UI view.\n\
     Rules:\n\
     - Use only components from the catalog below\n\
     {catalog_prompt}\n\n\
     …",
);
```

Check the exact interpolation pattern at line ~100 — there's a `format!` block that needs `COMPONENT_CATALOG` replaced with a local variable.

### `ferro-cli/src/commands/make_json_view.rs`

**No migration required.** Grep confirms zero `COMPONENT_CATALOG` references. Included in CONTEXT D-26 for completeness; Plan 06 confirms the grep result and moves on.

### External consumers (workspace-wide grep required)

Plan 06 Task 1 must run: `rg "COMPONENT_CATALOG" --type rust` across the whole workspace. Expected hits: `ferro-json-ui/src/lib.rs` (definition — deleted), `ferro-cli/src/ai.rs` (migrated), `ferro-mcp/src/tools/json_ui_generate.rs` (migrated). Any other hits are unexpected and must be audited before completing the plan.

## 10. Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in, no additional deps) |
| Config file | None (workspace-default) |
| Quick run command | `cargo test -p ferro-json-ui --lib catalog::` |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CAT-01 | `Catalog::build()` produces non-empty components map with every BUILTIN_TYPES entry | unit | `cargo test -p ferro-json-ui --lib catalog::build_populates_all_builtins` | ❌ Wave 0 — new file |
| CAT-01 | Plugin components discovered from `global_plugin_registry()` | unit | `cargo test -p ferro-json-ui --lib catalog::build_discovers_plugins` | ❌ Wave 0 |
| CAT-01 | Drift guard: `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` | unit | `cargo test -p ferro-json-ui --lib catalog::builtin_specs_len_matches_dispatch` | ❌ Wave 0 |
| CAT-02 | `catalog.prompt()` ≤ 8 KB | unit | `cargo test -p ferro-json-ui --lib catalog::prompt_under_size_budget` | ❌ Wave 0 |
| CAT-02 | `catalog.prompt()` contains every built-in name | unit | `cargo test -p ferro-json-ui --lib catalog::prompt_mentions_every_builtin` | ❌ Wave 0 |
| CAT-02 | `catalog.prompt()` is deterministic (repeatable byte-for-byte) | unit | `cargo test -p ferro-json-ui --lib catalog::prompt_is_deterministic` | ❌ Wave 0 |
| CAT-03 | `validate(&spec)` passes on minimal valid spec | unit | `cargo test -p ferro-json-ui --lib catalog::validate_positive_per_type` | ❌ Wave 0 |
| CAT-03 | `validate(&spec)` → `UnknownType` on unknown type_name | unit | `cargo test -p ferro-json-ui --lib catalog::validate_unknown_type` | ❌ Wave 0 |
| CAT-03 | `validate(&spec)` → `PropsInvalid` on missing required prop | unit | `cargo test -p ferro-json-ui --lib catalog::validate_missing_required_prop` | ❌ Wave 0 |
| CAT-03 | `validate(&spec)` → `SpecInvalid` on malformed $schema | unit | `cargo test -p ferro-json-ui --lib catalog::validate_bad_schema_version` | ❌ Wave 0 |
| CAT-03 | `validate(&spec)` → `BuildFailed` when plugin returns invalid schema | unit | `cargo test -p ferro-json-ui --lib catalog::build_fails_on_invalid_plugin_schema` | ❌ Wave 0 |
| CAT-03 | Pre-dispatch wins: unknown types do NOT trigger full-schema validation | unit | `cargo test -p ferro-json-ui --lib catalog::validate_pre_dispatch_short_circuits` | ❌ Wave 0 |
| CAT-04 | `component_schema("Card")` returns Props-only (no Element wrapper) | unit | `cargo test -p ferro-json-ui --lib catalog::component_schema_returns_props_only` | ❌ Wave 0 |
| CAT-04 | `component_schema("Unknown")` returns `None` | unit | `cargo test -p ferro-json-ui --lib catalog::component_schema_none_for_unknown` | ❌ Wave 0 |
| SCHEMA-01 | `catalog.json_schema()` is a valid JSON Schema (meta-validates) | unit | `cargo test -p ferro-json-ui --lib catalog::json_schema_is_valid` | ❌ Wave 0 |
| SCHEMA-01 | `catalog.json_schema()` contains every built-in as a `const` in the `oneOf` | unit | `cargo test -p ferro-json-ui --lib catalog::json_schema_oneof_covers_all_builtins` | ❌ Wave 0 |
| SCHEMA-01 | `catalog.json_schema()` includes `$defs/Action` and `$defs/Visibility` | unit | `cargo test -p ferro-json-ui --lib catalog::json_schema_has_action_and_visibility_defs` | ❌ Wave 0 |
| SCHEMA-02 | `ferro json-ui:schema` to stdout (no args) prints valid JSON | integration | `cargo run -- json-ui:schema \| jq .` | ❌ Wave 0 manual smoke |
| SCHEMA-02 | `ferro json-ui:schema --output /tmp/s.json` writes file | integration | shell smoke test in CI; assert file exists and parses | ❌ Wave 0 |
| SCHEMA-02 | `ferro json-ui:schema --component Card --pretty` prints single Props | integration | shell smoke test | ❌ Wave 0 |
| SCHEMA-03 | Full-schema validator compiled once (not per-call) | unit — observable by constructing Catalog, calling `validate` 100× and asserting no measurable compile-cost growth | `cargo test -p ferro-json-ui --lib catalog::validator_is_cached_not_recompiled` | ❌ Wave 0 |
| (SC-7) | Workspace-wide `COMPONENT_CATALOG` grep returns zero | integration | CI check: `rg "COMPONENT_CATALOG" --type rust \| wc -l` must be 0 after Plan 06 | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-json-ui --lib catalog::` (should be < 10 s once catalog tests are in)
- **Per wave merge:** `cargo test -p ferro-json-ui --lib && cargo test -p ferro-mcp --lib && cargo test -p ferro-cli --lib` (scoped to touched crates)
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features`

### Wave 0 Gaps

- [ ] `ferro-json-ui/src/catalog.rs` — new file, hosts all unit tests inline under `#[cfg(test)] mod tests`
- [ ] `ferro-json-ui/Cargo.toml` — add `jsonschema = "0.46"` dep
- [ ] `ferro-cli/src/commands/json_ui_schema.rs` — new file (Plan 06)
- [ ] Framework smoke test for `cargo run -- json-ui:schema` — may go in `framework/tests/` or stay as manual verification in Plan 07

*(No new test framework install needed — `cargo test` is workspace-default.)*

## 11. Risks & Caveats

### HIGH Priority

**H-1: `jsonschema` crate version drift (CONTEXT says 0.28, current stable is 0.46).**

CONTEXT D-09 specifies `jsonschema = "0.28"`. Research confirms the current stable is 0.46.0 [VERIFIED: `cargo search jsonschema` 2026-04-18]. The API differs between these versions:

- 0.28: `jsonschema::JSONSchema::compile(schema)` returned older-style validator
- 0.46: `jsonschema::validator_for(schema) -> Result<Validator, ReferencingError>` + dedicated `draft202012::` module

**Recommendation:** Plan 01 MUST pin `jsonschema = "0.46"` (or whatever is current at Plan 01 time, verified via `cargo search jsonschema`), NOT 0.28. Update CONTEXT D-09 inline with a note "superseded by research — see 117-RESEARCH.md §11 H-1". The API shift is net-positive (cleaner trait surface, better error types in 0.46). This is the single most important planner override needed.

**H-2: schemars 1.x output quality for complex Props.**

Some Props structs are non-trivial: `KanbanBoardProps` contains `Vec<KanbanColumnProps>` with nested `Vec<String>` children; `DataTableProps` contains `Vec<DropdownMenuAction>` which contains `Action` (a `JsonSchema`-derived type with `ActionOutcome` enum inside). Plan 02 should spot-check the generated schema for at least three representative types:

- `KanbanBoardProps` (nested slot containers)
- `DataTableProps` (nested Action-containing items)
- `Tab` (struct nested inside TabsProps.tabs)

If schemars emits surprising shape (e.g., `$ref: "#/definitions/..."` instead of `$ref: "#/$defs/..."`, or fails to inline a referenced type), Plan 02 adds a normalization pass: walk the schema JSON tree and rewrite `definitions` → `$defs`. This is a common schemars-1.x migration chore [CITED: https://graham.cool/schemars/].

**Remediation if schemas are malformed:** Plan 02 includes a "schema sanitizer" helper that post-processes `schema_for!` output into a canonical shape. ~100 LOC.

**H-3: Plugin `props_schema()` returns untrusted `serde_json::Value`.**

`JsonUiPlugin::props_schema() -> serde_json::Value` [VERIFIED: `ferro-json-ui/src/plugin.rs:77`]. Phase 117 weaves plugin schemas into the full-schema `oneOf` without meta-validation (CONTEXT D-20). If a plugin returns `serde_json::json!({"this is not a valid schema": true})`, `Catalog::build()` calls `jsonschema::validator_for(&full_schema)` which will likely FAIL on the full schema compile, NOT on the individual plugin schema.

**Mitigation:** Plan 02 `build()` wraps each plugin schema ingestion in a `validator_for()` check:

```rust
for plugin_type in registered_plugin_types() {
    let schema = with_plugin(&plugin_type, |p| p.props_schema()).unwrap_or(Value::Null);
    // Meta-validate: is this at least a valid JSON Schema?
    if let Err(e) = jsonschema::validator_for(&schema) {
        return Err(CatalogError::BuildFailed(format!(
            "plugin '{plugin_type}' returned an invalid JSON Schema: {e}"
        )));
    }
    plugin_components.insert(plugin_type.clone(), ComponentSpec {
        name: plugin_type,
        description: String::from("Plugin component."),
        props_schema: schema,
        is_plugin: true,
        slot_fields: Vec::new(),
    });
}
```

This fails fast with a clear error instead of mysteriously corrupting the full-schema validator. Add a negative test: register a plugin whose `props_schema()` returns garbage, assert `Catalog::build()` → `Err(BuildFailed)`.

### MEDIUM Priority

**M-1: `BUILTIN_SPECS` table size.**

39 entries × (type_name + description + schema_fn + slot_fields) ≈ 200 LOC. Keep inline in `catalog.rs` — splitting into a separate file adds ceremony without benefit. If it grows past ~300 LOC, split to `catalog/builtin_specs.rs` (Claude's discretion per CONTEXT).

**M-2: CLI unified-binary integration.**

The unified binary is `framework/src/app.rs`, NOT `framework/src/bin/ferro.rs`. CONTEXT D-22 said `framework/src/bin/ferro.rs` — this file does not exist. [VERIFIED: `ls framework/src/bin/` returns empty]. Plan 06 must add the `json-ui:schema` subcommand to `framework/src/app.rs` Commands enum + match arm (see §8 for exact diff).

**M-3: Slot-field documentation in prompt (LLM comprehension).**

Phase 120 will consume `catalog.prompt()` as the LLM system context. If the LLM doesn't understand that `CardProps.footer` is `Vec<String>` (element IDs) and NOT inline component structures, it will generate broken specs. The prompt format in §7 explicitly documents this ("Footer IDs live in CardProps.footer, not in Element.children"). Plan 05 must include a concrete prompt snapshot test that compares against a fixed reference — regressions in prompt wording that confuse the LLM must show up as test failures.

**M-4: `Catalog::build()` `Result` vs. panic.**

CONTEXT recommends `Result<Catalog, CatalogError>` for clean CLI failure reporting. `global_catalog()` wraps with `OnceLock::get_or_init` which takes a panicking closure. The pattern:

```rust
pub fn global_catalog() -> &'static Catalog {
    static GLOBAL_CATALOG: OnceLock<Catalog> = OnceLock::new();
    GLOBAL_CATALOG.get_or_init(|| {
        Catalog::build().expect("failed to build global catalog — plugin registry may have invalid schemas")
    })
}
```

CLI handlers should favor local `Catalog::build()` to surface errors as exit codes, not panics. Plan 06 implements both — framework `json-ui:schema` handler calls `build()` directly; production handlers use `global_catalog()`.

### LOW Priority

**L-1: Catalog build cost at first access.**

30 `schema_for!` calls + 1 full-schema `validator_for` compile. Estimated < 100 ms on a modern machine. First-hit latency on a cold process. Plan 07 SUMMARY should note the measured cost.

**L-2: `COMPONENT_CATALOG` as public API.**

It's `pub const COMPONENT_CATALOG: &str` [VERIFIED: `ferro-json-ui/src/lib.rs:88`]. Deleting it is a breaking change for any external consumer. The grep over the workspace catches internal uses; external crates depending on `ferro-json-ui` would break. Since the crate is pre-1.0, this is acceptable per project norm (see memory: `project_ferro_publication.md` — "not in production, breaking changes OK"). No deprecation shim (CONTEXT D-23).

**L-3: Nested-type slots (Tab.children, KanbanColumnProps.children) not in `slot_fields`.**

The outer `slot_fields` field on `ComponentSpec` doesn't mention `Tab.children` (that lives inside `TabsProps.tabs[i].children`). The prompt and schema still document them via nested type schemas. Document in Plan 02 / Plan 05 — this is a known modeling choice, not a gap.

## 12. Plan Split Recommendation

Seven plans, each sized to fit a single executor session. LOC estimates include tests.

| Plan | Title | LOC | Waves | Focus |
|------|-------|-----|-------|-------|
| **117-01** | Prep & scaffolding | ~300 | 1 | Add `jsonschema = "0.46"` to `Cargo.toml`. Scaffold `catalog.rs` with empty `Catalog` struct, `CatalogError` enum (thiserror), `ComponentSpec` struct, `global_catalog()` OnceLock skeleton (returns placeholder). Drift-check test asserting `BUILTIN_TYPES.len() == 39`. All public API visible but not functional. Workspace builds green. |
| **117-02** | `Catalog::build()` discovery | ~600 | 2 | Static `BUILTIN_SPECS` table with all 39 entries (authored descriptions, slot_fields, schema_fn). Implement `Catalog::build() -> Result<Catalog, CatalogError>` populating `components`, `plugin_components`, `per_component_schemas`. Plugin schema meta-validation (H-3). Drift guard enforced (`BUILTIN_SPECS.len() == BUILTIN_TYPES.len()`). Schema sanitizer helper if H-2 materializes. Positive tests: every type_name present, every schema is a non-empty object. |
| **117-03** | Full spec schema assembly | ~400 | 2 | Implement `catalog.json_schema() -> &Value`. Hand-assemble root + `$defs/Element` + `$defs/Action` + `$defs/Visibility` + `oneOf` over all Props schemas with const discriminator. Cache as `catalog.full_schema`. Compile `catalog.validator: jsonschema::Validator` from `full_schema` in `build()` (SCHEMA-03 satisfied here). Tests: meta-validation passes, every type appears as const, Action/Visibility defs present. |
| **117-04** | Validation pipeline | ~500 | 2 | Implement `catalog.validate(&Spec) -> Result<(), Vec<CatalogError>>`. Two-stage: type_name whitelist + per-component Props schema + full-schema envelope. On-demand per-component validators (escape hatch documented). Positive tests per built-in type. Negative tests for every `CatalogError` variant. Pre-dispatch short-circuit test. |
| **117-05** | Prompt + component_schema export | ~400 | 2 | Implement `catalog.prompt() -> String` (Markdown, buckets, ≤ 8 KB, deterministic). Implement `catalog.component_schema(name) -> Option<&Value>`. Snapshot test for prompt (locks the LLM-facing contract). Size budget test. Every-type-mentioned test. Deterministic-output test (run prompt() twice, assert byte-equal). |
| **117-06** | CLI + consumer migration | ~600 | 3 | Add `ferro-cli/src/commands/json_ui_schema.rs` (shell-out). Add `JsonUiSchema` variant to `ferro-cli/src/main.rs` Commands enum. Add `JsonUiSchema` variant to `framework/src/app.rs` Commands enum + handler (§8). Delete `COMPONENT_CATALOG` from `lib.rs` + add `pub use catalog::*`. Migrate `ferro-mcp/tools/json_ui_generate.rs`, `ferro-cli/src/ai.rs`. Rewrite `ferro-mcp/tools/json_ui_catalog.rs` body preserving public shape (derive PropInfos from schemas). End-to-end: `cargo run -- json-ui:schema | jq . | head` works. Zero `COMPONENT_CATALOG` hits in workspace grep. |
| **117-07** | Integration + phase gate | ~200 | 4 | Framework-level integration test: load a Phase 115 fixture, call `global_catalog().validate(&spec)`, assert OK. Full CI-parity gate: `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` — all green. Phase SUMMARY with per-criterion PASS/FAIL. Document validator-compile cost measurement (L-1). |

**Wave layout recommendation:**
- Wave 1: Plan 01 (scaffold)
- Wave 2: Plans 02, 03, 04, 05 (parallelizable — each builds on Plan 01 scaffold; Plans 03/04 both consume Plan 02's populated `BUILTIN_SPECS`, so 02 must merge before 03/04 start)
- Wave 3: Plan 06 (consumer migration — depends on 05's `prompt()` API)
- Wave 4: Plan 07 (phase gate)

Planner may choose to merge 03+04 (both are validator-adjacent; ~900 LOC combined fits an executor session) or split 06 into 06a (framework + CLI) + 06b (mcp migration + COMPONENT_CATALOG delete). 7 plans is a conservative upper bound — 5 is achievable if the executor is aggressive.

## Architecture Patterns

### System Architecture Diagram

```
                ┌────────────────────────────────────────────────────┐
                │           Application Boot (first-access)          │
                └─────────────────────────┬──────────────────────────┘
                                          │
                                          ▼
                ┌────────────────────────────────────────────────────┐
                │  global_catalog() → OnceLock<Catalog>.get_or_init()│
                │         ─────────────────────────────────          │
                │  Catalog::build() -> Result<Catalog, CatalogError> │
                └─────────────────────────┬──────────────────────────┘
                                          │
                   ┌──────────────────────┼────────────────────────┐
                   ▼                      ▼                        ▼
        ┌──────────────────┐  ┌──────────────────────┐  ┌──────────────────────┐
        │  BUILTIN_SPECS   │  │  global_plugin_      │  │  schemars::          │
        │  static table    │  │  registry().read()   │  │  schema_for!(T)      │
        │  (39 entries)    │  │  (0..N plugins)      │  │  per Props struct    │
        └────────┬─────────┘  └──────────┬───────────┘  └──────────┬───────────┘
                 │                       │                         │
                 └───────────────────────┼─────────────────────────┘
                                         ▼
                ┌────────────────────────────────────────────────────┐
                │                 Catalog state                      │
                │ • components:            HashMap<String, Spec>     │
                │ • plugin_components:     HashMap<String, Spec>     │
                │ • per_component_schemas: HashMap<String, Value>    │
                │ • full_schema:           Value (oneOf assembled)   │
                │ • validator:             jsonschema::Validator     │
                │                            (COMPILED ONCE)         │
                └─────────────────────────┬──────────────────────────┘
                                          │
          ┌────────────┬──────────────────┼──────────────────┬──────────────┐
          ▼            ▼                  ▼                  ▼              ▼
     prompt()   json_schema()     validate(&Spec)   component_schema()  (CLI export)
     ─────────  ──────────────    ──────────────    ──────────────────  ──────────────
     Markdown    &Value           two-stage:        Option<&Value>      cargo run --
     ≤ 8 KB     zero-copy         ① type whitelist  Props only          json-ui:schema
     for LLM    for tooling       ② Props schema
                                  ③ envelope
                                     validator
```

### Recommended File Structure

```
ferro-json-ui/src/
├── catalog.rs              # Catalog, CatalogError, ComponentSpec, BUILTIN_SPECS,
│                           #  global_catalog, build, validate, prompt, json_schema
│                           #  component_schema. ≤ 1200 LOC target.
└── lib.rs                  # pub mod catalog; pub use catalog::{Catalog, ...};
                            #  COMPONENT_CATALOG DELETED.

ferro-cli/src/commands/
└── json_ui_schema.rs       # NEW — shell-out to cargo run -- json-ui:schema

framework/src/
└── app.rs                  # Commands enum + JsonUiSchema variant + handler (~30 LOC added)

ferro-mcp/src/tools/
├── json_ui_catalog.rs      # REWRITTEN body, public shape preserved
└── json_ui_generate.rs     # COMPONENT_CATALOG → global_catalog().prompt()

ferro-cli/src/
├── ai.rs                   # COMPONENT_CATALOG → global_catalog().prompt()
└── main.rs                 # JsonUiSchema variant added to Commands enum
```

### Pattern 1: OnceLock Global + Build-Once

**What:** `OnceLock<Catalog>` with lazy first-access build. Matches `global_plugin_registry` [VERIFIED: `ferro-json-ui/src/plugin.rs:147-158`].

**When:** Any pre-computed artifact that does not change after startup.

**Example:**

```rust
static GLOBAL_CATALOG: OnceLock<Catalog> = OnceLock::new();

pub fn global_catalog() -> &'static Catalog {
    GLOBAL_CATALOG.get_or_init(|| {
        Catalog::build().expect("catalog build failed — see CatalogError for details")
    })
}
```

### Pattern 2: Static BUILTIN_SPECS Table

**What:** `&[(name, description, schema_fn, slot_fields)]` array. No macro. Matches CONTEXT D-05.

**When:** Small fixed set (< 100) of related entries, each with a derivation function.

```rust
type SchemaFn = fn() -> serde_json::Value;

static BUILTIN_SPECS: &[(&str, &str, SchemaFn, &[&str])] = &[
    ("Text",
        "Semantic text element (p / h1 / h2 / h3 / span / div / section).",
        || serde_json::to_value(schema_for!(TextProps)).unwrap(),
        &[]),
    ("Card",
        "Content container with title, description, body children, and optional footer slot.",
        || serde_json::to_value(schema_for!(CardProps)).unwrap(),
        &["footer"]),
    // … 37 more entries
];
```

### Anti-Patterns to Avoid

- **Lazy per-method computation.** Don't make `json_schema()` build the schema each call. `Catalog::build()` is the one place that computes; method calls are `&self` accessors.
- **Trait-object catalog.** Don't introduce `Box<dyn ComponentSchemaProvider>`. The static table is clearer and faster.
- **Validator compiled per request.** Defeats SCHEMA-03. The full-schema validator lives on Catalog.
- **Embedding `COMPONENT_CATALOG` references anywhere post-migration.** Phase 117's success means zero grep hits after Plan 06.
- **Re-exporting `COMPONENT_CATALOG` for backwards compat.** Clean break per CONTEXT D-23.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema generation from Rust types | Custom reflection / derive macro | `schemars = "1"` (already in deps) | `schema_for!` handles recursion, enums, tagged unions |
| JSON Schema validation | Custom validator | `jsonschema = "0.46"` | Draft 2020-12, compiled validators, multi-error iteration |
| Plugin registry | Custom lookup map | `global_plugin_registry()` (Phase 116) | Already shipped, Send+Sync-correct, sort-by-name |
| Global singleton | Static mut / Mutex dance | `std::sync::OnceLock` | Thread-safe, one-shot init, `get_or_init` API |
| Error enum | Raw `String` + `.to_string()` errors | `thiserror` derive | Workspace convention; structured payloads (CONTEXT D-11, D-16) |
| CLI arg parsing | Hand-rolled `env::args()` | `clap` (already used everywhere in app.rs) | Subcommand composition, `--help` for free |

**Key insight:** This phase is almost entirely gluing existing primitives. The interesting work is the `oneOf` assembly (Plan 03) and the two-stage validate pipeline (Plan 04). Everything else is mechanical wire-up.

## Common Pitfalls

### Pitfall 1: schemars `definitions` vs. `$defs` drift

**What goes wrong:** schemars 0.8.x emitted `definitions`; schemars 1.x emits `$defs` (Draft 2020-12). A leftover reference inside an inlined schema (e.g., nested `Vec<Tab>` producing `$ref: "#/definitions/Tab"`) won't resolve in the assembled full-schema that uses `$defs`.

**Why it happens:** Mixed schemars versions in transitive deps, or schemars outputting unexpected keys.

**How to avoid:** Plan 02 adds a schema-sanitizer pass that rewrites `definitions` → `$defs` across every ingested schema. ~20 LOC of tree-walking.

**Warning signs:** `jsonschema::validator_for(&full_schema)` fails with "Unresolved reference" errors at build time.

### Pitfall 2: `oneOf` matches more than one variant (validation "ambiguous")

**What goes wrong:** If two component schemas have overlapping shape (e.g., both accept `{"title": "..."}`) and the discriminator isn't pinned, `oneOf` semantics require EXACTLY ONE match. If two match, validation fails with a confusing "oneOf has multiple matches" error.

**Why it happens:** Forgetting to pin `"type": { "const": "X" }` in the assembled variant. Without it, `CardProps` and `ModalProps` (both have `title + description + footer`) are indistinguishable.

**How to avoid:** Plan 03 test explicitly constructs two specs with overlapping prop sets but different `type_name`s, validates each, asserts SUCCESS for both. This proves the discriminator is doing its job.

**Warning signs:** Tests fail with `ValidationError: oneOf — multiple matches`; or the `validate()` pipeline reports ambiguous schema errors instead of targeted `PropsInvalid`.

### Pitfall 3: Prompt drift between snapshot test and actual output

**What goes wrong:** Plan 05 writes a snapshot test for `catalog.prompt()`. Someone adds a new Props struct without updating the snapshot. CI fails on an innocuous change.

**Why it happens:** Snapshot tests are fragile.

**How to avoid:** Two tests, not one. (a) **Fragile snapshot** asserting byte-equal to a reference file — catches ALL changes. (b) **Robust invariants** — length ≤ 8 KB, every type_name present, deterministic across two calls — catches only meaningful regressions. Plan 05 includes both; snapshot is easily regenerated via `--nocapture` workflow.

**Warning signs:** CI fails after adding a new component even though the new component works correctly.

### Pitfall 4: Plugin registration ordering with `OnceLock`

**What goes wrong:** Plugin registered AFTER `global_catalog()` first access is ignored. No warning.

**Why it happens:** `OnceLock::get_or_init` freezes on first read. The `RwLock<PluginRegistry>` can still accept new plugins, but the Catalog snapshot is stuck.

**How to avoid:** Document in rustdoc on `global_catalog()`: "Call once all plugins have registered. Subsequent plugin registrations are not reflected." Framework `app.rs` bootstrap order must register plugins BEFORE any handler calls `global_catalog()` (renderers don't — they don't need the catalog; but ferro-mcp tools might).

**Warning signs:** Plugin renders fine at runtime (walker has its own plugin fallback) but `catalog.validate()` returns `UnknownType` for it.

### Pitfall 5: CLI binary vs. library dispatch

**What goes wrong:** `ferro-cli`'s `json-ui:schema` command shells out to `cargo run -- json-ui:schema`. But that only works inside a Ferro app directory (where a main.rs calls `Application::new().run().await`). If run outside a project, `cargo run --` doesn't know what to build.

**Why it happens:** CONTEXT D-22 treats the unified binary as a per-project app entry point, not a global ferro binary. `ferro-cli` is the global scaffolder; subcommands it exposes for introspecting a user's app use the shell-out.

**How to avoid:** `ferro-cli/src/commands/json_ui_schema.rs` must detect "am I in a Ferro project?" via `Cargo.toml` presence (mirrors `db_status.rs`). Error clearly if not. Plan 06 adds the check.

**Warning signs:** `ferro json-ui:schema` run from `/tmp` emits `error: failed to parse manifest at '/tmp/Cargo.toml'` or similar cargo-level error.

## Code Examples

### Building the Catalog

```rust
// Source: Phase 117 design, derived from ferro-json-ui/src/plugin.rs OnceLock pattern
use schemars::schema_for;
use serde_json::{to_value, Value};

pub struct Catalog {
    pub(crate) components: std::collections::HashMap<String, ComponentSpec>,
    pub(crate) plugin_components: std::collections::HashMap<String, ComponentSpec>,
    pub(crate) full_schema: Value,
    pub(crate) per_component_schemas: std::collections::HashMap<String, Value>,
    pub(crate) validator: jsonschema::Validator,
}

impl Catalog {
    pub fn build() -> Result<Self, CatalogError> {
        // 1. Populate built-ins from BUILTIN_SPECS.
        let mut components = HashMap::with_capacity(BUILTIN_SPECS.len());
        let mut per_component_schemas = HashMap::with_capacity(BUILTIN_SPECS.len() * 2);
        for (name, desc, schema_fn, slots) in BUILTIN_SPECS {
            let schema = schema_fn();
            per_component_schemas.insert((*name).to_string(), schema.clone());
            components.insert((*name).to_string(), ComponentSpec {
                name: (*name).to_string(),
                description: (*desc).to_string(),
                props_schema: schema,
                is_plugin: false,
                slot_fields: slots.iter().map(|s| (*s).to_string()).collect(),
            });
        }

        // Drift guard.
        if components.len() != crate::render::BUILTIN_TYPES.len() {
            return Err(CatalogError::BuildFailed(format!(
                "BUILTIN_SPECS has {} entries but BUILTIN_TYPES has {}",
                components.len(),
                crate::render::BUILTIN_TYPES.len(),
            )));
        }

        // 2. Populate plugins.
        let mut plugin_components = HashMap::new();
        for plugin_type in crate::plugin::registered_plugin_types() {
            let schema = crate::plugin::with_plugin(&plugin_type, |p| p.props_schema())
                .unwrap_or(Value::Null);
            // Meta-validate plugin schema (H-3).
            if jsonschema::validator_for(&schema).is_err() {
                return Err(CatalogError::BuildFailed(format!(
                    "plugin '{plugin_type}' returned an invalid JSON Schema"
                )));
            }
            per_component_schemas.insert(plugin_type.clone(), schema.clone());
            plugin_components.insert(plugin_type.clone(), ComponentSpec {
                name: plugin_type.clone(),
                description: String::from("Plugin component."),
                props_schema: schema,
                is_plugin: true,
                slot_fields: Vec::new(),
            });
        }

        // 3. Hand-assemble full spec schema.
        let full_schema = assemble_full_schema(&per_component_schemas)?;

        // 4. Compile once.
        let validator = jsonschema::validator_for(&full_schema)
            .map_err(|e| CatalogError::BuildFailed(format!("compiling full schema: {e}")))?;

        Ok(Catalog {
            components,
            plugin_components,
            full_schema,
            per_component_schemas,
            validator,
        })
    }
}
```

### Assembling the Full Schema

```rust
// Source: Phase 117 design
fn assemble_full_schema(
    per_component: &HashMap<String, Value>,
) -> Result<Value, CatalogError> {
    let action_schema = serde_json::to_value(schema_for!(crate::action::Action))?;
    let visibility_schema = serde_json::to_value(schema_for!(crate::visibility::Visibility))?;

    // Build oneOf — sorted for determinism.
    let mut names: Vec<&String> = per_component.keys().collect();
    names.sort();
    let one_of: Vec<Value> = names
        .into_iter()
        .map(|name| {
            let props_schema = &per_component[name];
            // Merge type const into properties map, OR use allOf wrapper.
            // Here: allOf for clarity.
            serde_json::json!({
                "allOf": [
                    {
                        "type": "object",
                        "required": ["type"],
                        "properties": { "type": { "const": name } }
                    },
                    props_schema
                ]
            })
        })
        .collect();

    Ok(serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "ferro-json-ui/v2",
        "type": "object",
        "required": ["$schema", "root", "elements"],
        "properties": {
            "$schema":  { "const": "ferro-json-ui/v2" },
            "root":     { "type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_-]{0,127}$" },
            "elements": {
                "type": "object",
                "additionalProperties": { "$ref": "#/$defs/Element" }
            },
            "title":    { "type": ["string", "null"] },
            "layout":   { "type": ["string", "null"] },
            "data":     true
        },
        "$defs": {
            "Element": {
                "type": "object",
                "required": ["type"],
                "properties": {
                    "type":     { "type": "string" },
                    "props":    { "oneOf": one_of },
                    "children": { "type": "array", "items": { "type": "string" } },
                    "action":   { "$ref": "#/$defs/Action" },
                    "visible":  { "$ref": "#/$defs/Visibility" }
                }
            },
            "Action": action_schema,
            "Visibility": visibility_schema
        }
    }))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-maintained const string `COMPONENT_CATALOG` | Machine-derived `Catalog` from `schemars::JsonSchema` | Phase 117 | Source of truth = Props types; zero drift |
| `jsonschema::JSONSchema::compile` (0.x) | `jsonschema::validator_for` / `jsonschema::draft202012::new` (0.46) | jsonschema 0.30 migration | Cleaner API, explicit draft support |
| schemars 0.8 `definitions` | schemars 1.x `$defs` (Draft 2020-12) | schemars 1.0 (~Oct 2024) | Modern JSON Schema draft; post-process may be needed |
| v1 `JsonUiView` with nested `Component` enum | v2 `Spec` with flat `elements: HashMap<String, Element>` | Phase 115 | Type-erased props, discriminator via `type` string |
| Walker dispatches on `Component::Card(..)` match variant | Walker dispatches on `el.type_name.as_str()` match arm | Phase 116 | Unknown types degrade to HTML comments, not compile errors |

**Deprecated / outdated:**

- jsonschema ≤ 0.28 API (`JSONSchema::compile` returning `Result<JSONSchema, ...>`) — replaced by `validator_for`. CONTEXT D-09's pin is stale.
- schemars 0.8.x `definitions` key — replaced by `$defs`. Not a concern if all deps are on 1.x (verified: ferro-json-ui uses `schemars = "1"`).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `framework/src/bin/ferro.rs` does NOT exist — unified binary dispatch lives in `framework/src/app.rs` Commands enum | §8, Risk M-2 | If a separate ferro binary DOES exist somewhere, CLI wiring goes to the wrong place. Mitigation: verified via `ls framework/src/bin/` → empty. HIGH confidence. |
| A2 | `jsonschema = "0.46"` API is stable enough to ship (no semver-major changes since 0.40.x) | §4, Risk H-1 | API could shift before Phase 117 lands. Mitigation: Plan 01 pins exact version and verifies via `cargo check` before committing. |
| A3 | schemars 1.x emits Draft 2020-12 by default, compatible with jsonschema Draft 2020-12 validator | §3 | If schemars emits Draft 2019-09 or older by default, validator refuses the schema. Mitigation: Plan 02 sanitizer normalizes + explicit draft declaration in `full_schema["$schema"]`. |
| A4 | `global_plugin_registry()` is safe to call during `Catalog::build()` (no re-entrancy issue) | §9, H-3 | Re-entrancy deadlock if `build()` itself registers plugins (it doesn't). Mitigation: `build()` only READS the registry via `registered_plugin_types()`. |
| A5 | The 8 KB prompt budget is generous enough for 39 components with inline enum variants | §7, M-3 | Prompt overflow. Mitigation: soft warn only (D-17), and enum-inline cutoff at 8 variants keeps worst-case bounded. |
| A6 | Existing `ferro-mcp/tools/json_ui_inspect.rs` does NOT need migration (v1 regex scanner, deferred to Phase 120) | §9 | If it does consume `COMPONENT_CATALOG`, Plan 06 grep will surface it. Mitigation: CONTEXT + file reading confirmed it uses its own stale `BUILTIN_TYPES`. LOW risk. |
| A7 | `derive_prop_infos_from_schema` bridging in `json_ui_catalog.rs` rewrite is feasible (~80 LOC) | §9 | Could balloon if schemars output shape varies significantly across types. Mitigation: Plan 06 has Task 1 to prototype the helper against 3 representative types before committing to the full rewrite. |

**Claims tagged [ASSUMED] above:** A2 (jsonschema API stability), A5 (prompt budget sufficiency), A7 (bridging feasibility). These are the decisions the planner may want to sanity-check at Plan 01 time or confirm with the user if the Phase 120 downstream tolerances turn out to be tighter.

## Open Questions

1. **Should `Catalog::build()` panic or return `Result` when called via `global_catalog()`?**
   - What we know: CONTEXT recommends `Result` for CLI clean-exit; `OnceLock::get_or_init` takes a panicking closure.
   - What's unclear: How should `global_catalog()` surface a `BuildFailed` error to production handlers?
   - Recommendation: `global_catalog()` panics with a clear message (matches `global_plugin_registry()` pattern); CLI code uses `Catalog::build()` directly for graceful error reporting. Document both paths in rustdoc.

2. **Should slot-ID graph validation (Phase 117.5) be folded into Plan 07 or stay deferred?**
   - What we know: CONTEXT D-31 explicitly defers it.
   - What's unclear: How much would it cost to add `Catalog::validate_slots(&spec)` in this phase?
   - Recommendation: Defer per CONTEXT. Adding it bloats Plan 04 + Plan 07 by ~150 LOC and moves Phase 117 from "schema validation" to "schema + semantic validation," expanding scope unnecessarily. Phase 117.5 or Phase 119 can add it cleanly.

3. **Should the `oneOf` variants use `allOf` wrapping (CONTEXT D-13 shape) or inline-merge?**
   - What we know: Both work; inline-merge produces smaller output but requires careful schema merging logic.
   - What's unclear: Which produces cleaner JSON for Phase 120's per-component structured output?
   - Recommendation: Start with `allOf` (simpler to implement, CONTEXT shape). If Phase 120 finds OpenAI/Anthropic structured output chokes on nested `allOf`, Plan 03 can re-emit using inline-merge in a follow-up.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | build + test | ✓ | stable 1.8x+ (workspace) | — |
| cargo | build + CLI | ✓ | bundled | — |
| schemars (workspace dep) | schema generation | ✓ | 1.x | — |
| serde / serde_json (workspace dep) | Value type | ✓ | 1.x | — |
| thiserror (workspace dep) | CatalogError derives | ✓ | 1.x | — |
| jsonschema | validator | ✗ (NEW DEP) | 0.46 target | — (no fallback; must install) |
| jq (dev smoke test) | Plan 07 CLI smoke | Likely present on macOS | — | Use `python3 -m json.tool` or plain `cat` |

**Missing dependencies with no fallback:** `jsonschema = "0.46"` must be added in Plan 01.

**Missing dependencies with fallback:** None critical. `jq` for CLI smoke testing is a nice-to-have.

## Sources

### Primary (HIGH confidence)

- [VERIFIED: `ferro-json-ui/src/lib.rs`] — existing `COMPONENT_CATALOG` const, public API, re-exports
- [VERIFIED: `ferro-json-ui/src/component.rs`] — 39 Props structs with `#[derive(JsonSchema)]`
- [VERIFIED: `ferro-json-ui/src/render/mod.rs`] — `BUILTIN_TYPES` canonical list, 39 entries
- [VERIFIED: `ferro-json-ui/src/spec.rs`] — Spec/Element shape, SCHEMA_VERSION
- [VERIFIED: `ferro-json-ui/src/plugin.rs`] — `JsonUiPlugin::props_schema()`, `global_plugin_registry()`, OnceLock pattern
- [VERIFIED: `ferro-json-ui/src/visibility.rs`] — `Visibility` with `JsonSchema`
- [VERIFIED: `ferro-json-ui/src/action.rs`] — `Action` with `JsonSchema`
- [VERIFIED: `ferro-json-ui/Cargo.toml`] — workspace deps (no jsonschema yet)
- [VERIFIED: `ferro-mcp/src/tools/json_ui_catalog.rs`] — current hand-maintained CatalogComponent shape
- [VERIFIED: `ferro-mcp/src/tools/json_ui_generate.rs`] — system prompt uses `COMPONENT_CATALOG`
- [VERIFIED: `ferro-mcp/src/tools/json_ui_inspect.rs`] — stale local `BUILTIN_TYPES`, v1 regex, `TODO(Phase 120)`
- [VERIFIED: `ferro-cli/src/ai.rs`] — also consumes `COMPONENT_CATALOG` (not mentioned in CONTEXT but must migrate)
- [VERIFIED: `ferro-cli/src/commands/db_status.rs`] — shell-out pattern for ferro-cli commands
- [VERIFIED: `framework/src/app.rs`] — unified Commands enum + run() match arm
- [VERIFIED: `.planning/ROADMAP.md` §Phase 117] — 8 success criteria, 4 caveats
- [VERIFIED: `.planning/phases/117-catalog-and-json-schema/117-CONTEXT.md`] — all 35 locked decisions
- [VERIFIED: `.planning/phases/116-flat-element-renderer/116-06-SUMMARY.md`] — Phase 116 hand-off
- [VERIFIED: crates.io via `cargo search jsonschema`] — current version 0.46.0
- [CITED: https://docs.rs/jsonschema/latest/jsonschema/] — jsonschema 0.46 API
- [CITED: https://graham.cool/schemars/] — schemars 1.x Draft 2020-12 default

### Secondary (MEDIUM confidence)

- [CITED: https://github.com/Stranger6667/jsonschema] — jsonschema crate is actively maintained (Stranger6667), stars count not critical but repo activity confirmed.
- [CITED: https://json-schema.org/draft/2020-12/schema] — JSON Schema Draft 2020-12 specification URL

### Tertiary (LOW confidence — needs verification at Plan 01 time)

- Exact `jsonschema = "0.46"` breaking changes vs. 0.28 — verify by reading the crate CHANGELOG during Plan 01 install.
- Precise overhead of 30 `schema_for!` calls + 1 `validator_for` compile on cold boot — measure in Plan 07.

## Metadata

**Confidence breakdown:**

- Standard stack (schemars, jsonschema, serde_json, thiserror): **HIGH** — all are workspace-default or verified via registry
- Architecture patterns (OnceLock, static table, two-stage validate): **HIGH** — all grounded in existing ferro-json-ui patterns
- Pitfalls: **HIGH** — derived from schemars/jsonschema docs and CONTEXT
- Props inventory: **HIGH** — directly read from `BUILTIN_TYPES` + `component.rs`
- jsonschema 0.46 API surface: **HIGH** — verified via docs.rs fetch
- Consumer migration scope: **HIGH** — workspace grep confirmed; only 2 ferro-mcp files + 1 ferro-cli file need migration
- Plan split: **MEDIUM** — 7 plans is conservative; planner may compress to 5

**Research date:** 2026-04-18
**Valid until:** ~2026-05-18 (30 days — jsonschema ecosystem moves, but 0.46 should stay stable for this window)
