# Phase 117: Catalog & JSON Schema - Context

**Gathered:** 2026-04-18
**Status:** Ready for planning
**Mode:** `--auto` — decisions auto-selected for a well-designed implementation inspired by Vercel json-render's catalog model, `jsonforms` schema-as-truth discipline, rjsf's per-component schema pattern, and shadcn-ui's documented-component model. Downstream agents may override any auto-choice by editing this file before `/gsd-plan-phase`.

<domain>
## Phase Boundary

Replace `COMPONENT_CATALOG` (a 4.5 KB hand-maintained const string in `ferro-json-ui/src/lib.rs`) with a machine-readable `Catalog` backed by `schemars::JsonSchema` derives already present on every `*Props` struct (Phase 115). Catalog responsibilities:

- Auto-discover every built-in Component with its name, description, and per-Props JSON Schema.
- Pull plugin components from the global plugin registry (`JsonUiPlugin::props_schema()`) at build time.
- Emit a concise text prompt (`catalog.prompt()`) for AI system contexts — same size envelope (~4–8 KB) as today's string, but generated from the live Props shapes.
- Export the full JSON Schema document (`catalog.json_schema()`) for external tools / IDEs / validators.
- Validate v2 Specs against the catalog (`catalog.validate(&spec)`) using a single-compile `jsonschema` validator + a pre-dispatch type-name check that collapses the `oneOf` worst case.
- Expose `ferro json-ui:schema` CLI so external tooling can export the schema to stdout or a file.

**What this phase does NOT do** (locked by ROADMAP, do not re-open):
- Schema-driven projections from `ServiceDef` — Phase 117.1.
- Real render-time expression resolution (`$data`, `$template`) — Phase 118.
- Page loader / hot-reload — Phase 119.
- Full MCP `json_ui_generate` rewrite into a two-tier AI strategy — Phase 120.
- Docs rewrite + gestiscilo field test — Phase 121.

</domain>

<decisions>
## Implementation Decisions

### Crate boundary

- **D-01: Catalog lives in `ferro-json-ui/src/catalog.rs`.** Same crate as `render/*`, `component.rs`, `spec.rs`, `plugin.rs`. Keeps dispatch-list (`BUILTIN_TYPES`), Props types, and Catalog co-located so drift between them is a compile error. No new crate, no `ferro-catalog` split.
- **D-02: Catalog is public API.** `pub use catalog::{Catalog, CatalogError, ComponentSpec};` in `lib.rs`. Used by `framework`, `ferro-cli`, `ferro-mcp`.

### Catalog shape

- **D-03: `Catalog` struct holds everything pre-computed.** Lazy build is not worth it for a catalog this small (~40 components). Build runs once; subsequent accesses are O(1) HashMap lookups.
  ```rust
  pub struct Catalog {
      components: HashMap<String, ComponentSpec>,       // type_name → spec (built-ins)
      plugin_components: HashMap<String, ComponentSpec>, // plugin type_name → spec
      full_schema: serde_json::Value,                    // spec schema (root + elements + oneOf)
      per_component_schemas: HashMap<String, serde_json::Value>, // props schema per type_name
      validator: jsonschema::Validator,                  // compiled once, reused
  }

  pub struct ComponentSpec {
      pub name: String,
      pub description: String,
      pub props_schema: serde_json::Value,  // schemars output for TProps
      pub is_plugin: bool,
      pub slot_fields: Vec<String>,         // ["footer"], ["children"], etc. — empty for non-slot components
  }
  ```
- **D-04: Global singleton via `OnceLock<Catalog>` — same pattern as `global_plugin_registry()`.** Exposed as `pub fn global_catalog() -> &'static Catalog`. Built the first time it is called, using the plugin registry state at that moment. After first build the Catalog is frozen — subsequent plugin registrations do NOT propagate into the catalog (Phase 117 scope; hot-swap is deferred).

### Component discovery

- **D-05: Static list of built-in components in `catalog::builtin_specs()`.** One entry per Props struct. Each entry is `(type_name: &str, description: &str, schema_fn: fn() -> serde_json::Value, slot_fields: &[&str])`. The `schema_fn` invokes `serde_json::to_value(schemars::schema_for!(TProps))?` at Catalog build time.

  Example entries:
  ```rust
  static BUILTIN_SPECS: &[(&str, &str, fn() -> Value, &[&str])] = &[
      ("Text",   "Semantic text element (p/h1/h2/h3/span/div/section).",
                 || to_value(schema_for!(TextProps)).unwrap(), &[]),
      ("Card",   "Content container with title/description and optional footer slot.",
                 || to_value(schema_for!(CardProps)).unwrap(), &["footer"]),
      ("Tabs",   "Tab switcher; per-tab children live in TabsProps.tabs[].children.",
                 || to_value(schema_for!(TabsProps)).unwrap(), &[]),
      // …one entry per built-in type in BUILTIN_TYPES
  ];
  ```

- **D-06: Descriptions are authored inline in `builtin_specs()`.** They match the voice of today's `COMPONENT_CATALOG` (short, imperative, one sentence). This avoids reading them off `#[doc = "..."]` attributes via macro acrobatics — straightforward, auditable, searchable.

- **D-07: Drift guard at Catalog::build time.** Assert `builtin_specs().len() == BUILTIN_TYPES.len()` and every name in `BUILTIN_SPECS` matches an entry in `BUILTIN_TYPES`. Mismatch → panic at startup with a clear message ("component added to dispatch without a catalog entry"). Unit test enforces the same invariant so CI catches it.

- **D-08: Plugin discovery via `global_plugin_registry().read()`.** For every registered plugin:
  - `type_name = plugin.component_type().to_string()`
  - `description = "Plugin component."` (generic fallback; plugins can override in a future phase)
  - `props_schema = plugin.props_schema()` (already `serde_json::Value` per existing `JsonUiPlugin` trait)
  - `is_plugin = true`
  - `slot_fields = &[]` (plugins don't participate in the built-in slot model)

### Validation pipeline

- **D-09: Use the `jsonschema` crate.** Add `jsonschema = { version = "0.28", default-features = false }` to `ferro-json-ui/Cargo.toml`. Version pinned to a Draft 2020-12-compatible release so schemars output (also 2020-12) is directly consumable.

- **D-10: Two-stage validation — pre-dispatch on `"type"` then full jsonschema check.** Per ROADMAP caveat (jsonschema's `oneOf` is linear). Concrete flow:

  ```rust
  impl Catalog {
      pub fn validate(&self, spec: &Spec) -> Result<(), Vec<CatalogError>> {
          let mut errors = Vec::new();

          // Stage 1: type_name whitelist (O(n))
          for (id, el) in &spec.elements {
              if !self.components.contains_key(&el.type_name)
                 && !self.plugin_components.contains_key(&el.type_name) {
                  errors.push(CatalogError::UnknownType {
                      element_id: id.clone(),
                      type_name: el.type_name.clone(),
                  });
              }
          }
          if !errors.is_empty() { return Err(errors); }

          // Stage 2: per-element Props validation against per_component_schemas
          for (id, el) in &spec.elements {
              if let Some(schema) = self.per_component_schemas.get(&el.type_name) {
                  let validator = jsonschema::validator_for(schema)?;  // cached per-type
                  if let Err(schema_errors) = validator.validate(&el.props) {
                      errors.push(CatalogError::PropsInvalid {
                          element_id: id.clone(),
                          type_name: el.type_name.clone(),
                          errors: schema_errors.map(|e| e.to_string()).collect(),
                      });
                  }
              }
          }

          // Stage 3 (optional): full spec schema validation (catches $schema, root, etc.)
          if let Err(e) = self.validator.validate(&serde_json::to_value(spec)?) {
              errors.push(CatalogError::SpecInvalid { errors: vec![e.to_string()] });
          }

          if errors.is_empty() { Ok(()) } else { Err(errors) }
      }
  }
  ```

  The per-element `validator_for(schema)` call is cheap because schemas are small and the `jsonschema` crate caches internally; if profiling later shows it dominating, cache per-type validators at Catalog build time (CONTEXT D-12 notes this escape hatch).

- **D-11: `CatalogError` variants.** Structured, `thiserror`-derived:
  ```rust
  pub enum CatalogError {
      UnknownType { element_id: String, type_name: String },
      PropsInvalid { element_id: String, type_name: String, errors: Vec<String> },
      SpecInvalid { errors: Vec<String> },
      BuildFailed(String),  // jsonschema compile failure during Catalog::build
      SchemaSerialization(#[from] serde_json::Error),
  }
  ```

- **D-12: Compiled validator reuse.** `Catalog::build()` compiles the full spec `jsonschema::Validator` once and stores it on the Catalog. For per-component validation, Phase 117 starts with on-demand `validator_for(per_component_schema)` inside `validate()` (simple, correct). If real-world profiling shows per-call compile overhead > 1 ms × N elements, upgrade to `HashMap<String, Validator>` precompiled at `build()` time. This is noted as a Phase 117 follow-up in the SUMMARY, not a v1.0 blocker.

### Full spec schema assembly

- **D-13: `Catalog::build()` emits the full spec schema by hand-assembling `oneOf` from per-component schemas.** Shape:
  ```json
  {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "ferro-json-ui/v2",
    "type": "object",
    "required": ["$schema", "root", "elements"],
    "properties": {
      "$schema": { "const": "ferro-json-ui/v2" },
      "root": { "type": "string" },
      "elements": {
        "type": "object",
        "additionalProperties": { "$ref": "#/$defs/Element" }
      },
      "title": { "type": ["string", "null"] },
      "layout": { "type": ["string", "null"] },
      "data": true
    },
    "$defs": {
      "Element": {
        "type": "object",
        "required": ["type"],
        "properties": {
          "type": { "type": "string" },
          "props": { "oneOf": [ /* per-component props schemas, each with its `type` const pinned */ ] },
          "children": { "type": "array", "items": { "type": "string" } },
          "action": { "$ref": "#/$defs/Action" },
          "visible": { "$ref": "#/$defs/Visibility" }
        }
      },
      "Action": { /* schema_for!(Action) */ },
      "Visibility": { /* schema_for!(Visibility) */ }
    }
  }
  ```
  Each entry in the `oneOf` is the component's props schema with a sibling `"type": { "const": "Card" }` constraint, so AI tools see discriminated-union semantics.

- **D-14: Hand-assemble the `oneOf`.** `schemars` does not automatically produce a discriminated union across disparate Props types. Phase 117 iterates `BUILTIN_SPECS` (and the plugin registry) and builds the `oneOf` array manually. This is ~40 LOC and keeps the output shape deterministic.

- **D-15: Cache the assembled schema as `catalog.full_schema: serde_json::Value`.** `catalog.json_schema()` returns `&Value` (zero-copy). Writing to a file or stdout clones once at the callsite.

### Prompt generation

- **D-16: `catalog.prompt() -> String` generates a Markdown-like text in the style of today's `COMPONENT_CATALOG`.** Per-component entry:
  ```
  ### Card
  Content container with title/description and optional footer slot.
  Props: title (String), description (Option<String>), max_width (Option<narrow|default|wide>), footer (Vec<String> of element IDs)
  ```
  Enum variants listed inline when the enum has ≤ 8 variants (ButtonVariant, Size, etc.). Longer enums use `one of N variants` with a reference to the schema. Slot fields documented explicitly.

- **D-17: Prompt output is bounded ≤ 8 KB.** Measured at build time; if it exceeds the budget, the overflow is logged (not an error — the ROADMAP caveat says this string is for AI context, and agents can deal with a longer one if needed). Current `COMPONENT_CATALOG` is 4.5 KB; growth margin is comfortable.

- **D-18: Prompt generation traverses `BUILTIN_SPECS + plugin_components` sorted by name.** Deterministic output so diffs are meaningful.

### Per-component schema export

- **D-19: `catalog.component_schema(name: &str) -> Option<&serde_json::Value>`** returns the per-component Props schema only (NOT wrapped in an Element or Spec). Used by AI structured-output generation in Phase 120 for targeted generation ("generate a CardProps").

- **D-20: Plugin schemas are opaque passthroughs.** Phase 117 does NOT validate plugin schemas themselves — it assumes plugin authors provide valid JSON Schema objects via `JsonUiPlugin::props_schema()`. If a plugin returns an invalid schema, it surfaces as a `Validator::for` failure at `Catalog::build()` → `CatalogError::BuildFailed`. Plugin authors are responsible for their own schema quality (mirrors CONTEXT D-17 from Phase 116).

### CLI surface

- **D-21: `ferro json-ui:schema` subcommand.** Lives in `ferro-cli/src/commands/json_ui_schema.rs`, follows the existing pattern (`cargo run -- json-ui:schema`). Flags:
  - `--output <path>` → write to file; default stdout
  - `--pretty` → pretty-print (serde_json::to_string_pretty)
  - `--component <name>` → export only that component's Props schema (via `catalog.component_schema(name)`)
  - No flags → pretty-printed full spec schema to stdout.
- **D-22: Binary entry point.** Add a `json-ui:schema` subcommand in `framework/src/bin/ferro.rs` (the unified binary). CLI shell-out and direct execution both converge on the same `Catalog::build().json_schema()` call.

### Consumer migrations (inside Phase 117 scope)

- **D-23: Delete `ferro-json-ui/src/lib.rs::COMPONENT_CATALOG`** const string. Replace call sites with `ferro_json_ui::global_catalog().prompt()`. Success criterion 7.
- **D-24: `ferro-mcp/src/tools/json_ui_catalog.rs` backing data.** The public MCP API shape (`JsonUiCatalog { components, plugin_components, builder_api, action_api }`) is preserved. Its body is rewritten to pull from `ferro_json_ui::global_catalog()`. The hand-maintained inlined prop/description/variant data in `json_ui_catalog.rs` is deleted. The hand-maintained `builder_api` and `action_api` strings stay — they document DSL idioms, not component shapes.
- **D-25: `ferro-mcp/src/tools/json_ui_generate.rs`** — update the system prompt to use `ferro_json_ui::global_catalog().prompt()` instead of `COMPONENT_CATALOG`. Two-tier AI strategy (concise prompt + per-component structured output) remains Phase 120 scope — Phase 117 only changes the context source.
- **D-26: `ferro-cli/src/commands/make_json_view.rs`** — update to use `global_catalog().prompt()` if it currently references COMPONENT_CATALOG. Grep confirms at planning time.

### Testing

- **D-27: Inline unit tests in `catalog.rs`.** Build the Catalog and assert:
  - Every name in `BUILTIN_TYPES` appears as a key in `components`.
  - Every `ComponentSpec.props_schema` deserializes into an object with a `"type": "object"` or `"type": "null"` entry.
  - `catalog.prompt()` length ≤ 8 KB.
  - `catalog.prompt()` contains every built-in type name at least once.
  - `catalog.json_schema()` is a valid JSON Schema (`jsonschema::Validator::compile` succeeds).
  - `catalog.component_schema("Card")` returns the CardProps schema and not the Element wrapper.

- **D-28: Validation tests — positive.** For every built-in type, construct a minimal valid Element and confirm `catalog.validate(&spec)` passes.

- **D-29: Validation tests — negative.** For each error variant:
  - Unknown `type_name` → `UnknownType`.
  - Invalid Props shape (e.g., CardProps missing `title`) → `PropsInvalid`.
  - Spec missing `$schema` or with malformed `root` → `SpecInvalid`.
  - Plugin schema that doesn't compile as JSON Schema → `BuildFailed` during `Catalog::build()`.

- **D-30: Integration test — schema round-trip.** A fixture valid under Phase 115's `Spec::from_json` structural validator must also pass `catalog.validate`. A fixture with a dangling slot-ID (which Phase 115 does NOT catch — CONTEXT 116 D-07 note) must be caught by Phase 117's schema validation when the slot field is declared with `"items": { "type": "string" }` + per-element lookup. Actual slot-ID graph validation (walking `CardProps.footer` IDs to confirm they exist in `spec.elements`) is NOT a jsonschema concern — it is a semantic check layered on top. Phase 117 covers schema shape; deep slot-graph validation stays a documented follow-up in CONTEXT D-31 below.

- **D-31: Slot-ID graph validation is NOT in Phase 117's scope.** ROADMAP criterion 3 is specifically about schema + type-name validation. The "are these IDs in the elements map?" check belongs with `Spec::from_json` (Phase 115 validator could be extended) or a separate catalog method like `Catalog::validate_slots(&spec)`. Phase 117 documents this as a known gap and defers to a Phase 117.5 or rolls it into the `catalog.validate` call as an optional check in a follow-up plan. Planner decides split.

### Out-of-scope reminders

- **D-32: `Spec` / `Element` struct shape is frozen.** Phase 115/116 shipped these; Phase 117 does not touch them.
- **D-33: Walker (`render/*`) is catalog-unaware.** Phase 117 does NOT add catalog validation inside the walker — validation is an explicit, caller-invoked step (typically at spec load time, Phase 119). The walker continues to emit HTML comments for anything it can't render.
- **D-34: `$data` / `$template` — Phase 118.**
- **D-35: Hot-swap plugin schemas after Catalog build — deferred.** Catalog is a one-shot build. If plugins register late, users rebuild by dropping and re-creating the global catalog (deliberate downtime event, not a v1.0 concern).

### Claude's Discretion

- Whether `global_catalog()` rebuilds on demand or is strictly `OnceLock`-frozen — prefer the latter for simplicity.
- Exact `CatalogError` variant naming — keep it flat and grep-friendly.
- Whether to split `catalog.rs` into `catalog/mod.rs` + `catalog/build.rs` + `catalog/validate.rs` + `catalog/prompt.rs` once the file exceeds ~1200 LOC — start single-file, split if it grows past that.
- Whether the CLI subcommand uses `clap`-style or hand-rolled arg parsing — match whatever the rest of `ferro-cli` uses.
- How large a tolerance to allow on prompt size before logging a warning — 8 KB is a heuristic, not a hard limit.
- Whether to ship `Catalog::build()` as `Result<Catalog, CatalogError>` (build can fail on invalid plugin schemas) or panic (consistent with today's lazy registries). Prefer `Result` to give callers (especially Phase 120 MCP tools) a chance to surface the failure cleanly.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase goal and success criteria
- `.planning/ROADMAP.md` §"Phase 117: Catalog & JSON Schema" — goal, 8 success criteria, 4 domain-research caveats (catalog size, jsonschema discriminator, compile-once, two-tier AI).
- `.planning/ROADMAP.md` §"v12.0 JSON-UI v2" milestone preamble — overall context.

### Upstream Phase 115–116 locked decisions
- `.planning/phases/115-spec-v2-data-structures/115-CONTEXT.md` — Spec/Element shape, `schemars::JsonSchema` already derived on every Props struct, schema version `"ferro-json-ui/v2"`.
- `.planning/phases/116-flat-element-renderer/116-CONTEXT.md` — `BUILTIN_TYPES` constant in `render/mod.rs` is the canonical built-in list, 5 slot fields (CardProps.footer, ModalProps.footer, Tab.children, KanbanColumnProps.children, PageHeaderProps.actions).
- `.planning/phases/116-flat-element-renderer/116-06-SUMMARY.md` — hand-off to Phase 117 explicitly documented: BUILTIN_TYPES is the source of truth, plugin props stay untyped (`serde_json::Value`), slot-ID graph validation is Phase 117's responsibility to consider.

### Downstream constraints (read to avoid painting into a corner)
- `.planning/ROADMAP.md` §"Phase 117.1: Schema-Driven Projections" — consumes Catalog to generate v2 specs from ServiceDef. Catalog's `component_schema()` + type-discovery is what 117.1 needs.
- `.planning/ROADMAP.md` §"Phase 118: Server-Side Expressions" — must not assume catalog has evaluated `$data`/`$template`.
- `.planning/ROADMAP.md` §"Phase 119: Page Loader" — calls `catalog.validate(&spec)` at load time.
- `.planning/ROADMAP.md` §"Phase 120: CLI & MCP Updates" — two-tier AI generation depends on `catalog.prompt()` + `catalog.component_schema()`.

### ferro-json-ui source (what Phase 117 rewires)
- `ferro-json-ui/src/lib.rs` — `COMPONENT_CATALOG` const string at line 88–168 (**delete in Phase 117**).
- `ferro-json-ui/src/render/mod.rs` — `BUILTIN_TYPES` at line 41 (**source of truth for Catalog drift check**).
- `ferro-json-ui/src/component.rs` — ~30 `*Props` structs with `#[derive(JsonSchema)]` (**read by Catalog::build**).
- `ferro-json-ui/src/spec.rs` — `Spec`, `Element`, `SCHEMA_VERSION = "ferro-json-ui/v2"` (**informs the full spec schema shape**).
- `ferro-json-ui/src/plugin.rs` — `JsonUiPlugin::props_schema() -> serde_json::Value`, `registered_plugin_types()`, `with_plugin()` (**used by plugin discovery in Catalog::build**).
- `ferro-json-ui/src/visibility.rs` — `Visibility` enum with `JsonSchema` derive (**referenced in full spec schema under `$defs/Visibility`**).
- `ferro-json-ui/src/action.rs` — `Action` struct with `JsonSchema` derive (**referenced in full spec schema under `$defs/Action`**).
- `ferro-json-ui/Cargo.toml` — add `jsonschema = "0.28"` dep.

### Consumer call sites
- `ferro-mcp/src/tools/json_ui_catalog.rs` — hand-maintained `JsonUiCatalog` struct. Rewire body to pull from `ferro_json_ui::global_catalog()`. Public API shape preserved.
- `ferro-mcp/src/tools/json_ui_generate.rs` — uses `COMPONENT_CATALOG` as LLM context. Switch to `global_catalog().prompt()`.
- `ferro-mcp/src/tools/json_ui_inspect.rs` — may benefit from `catalog.validate(spec)`; confirm at plan time.
- `ferro-cli/src/commands/make_json_view.rs` — check for `COMPONENT_CATALOG` references; migrate if present.
- `framework/src/bin/ferro.rs` (or equivalent unified binary entry) — add `json-ui:schema` subcommand dispatch.
- `ferro-cli/src/commands/json_ui_schema.rs` — **new file**.

### JSON Schema / jsonschema crate
- `jsonschema` crate docs — https://docs.rs/jsonschema (Draft 2020-12 support, compiled validators).
- `schemars` crate (v1.x) — already a workspace dep; `schema_for!(T)` is the canonical derivation call.

### Workspace conventions
- `CLAUDE.md` — builder pattern (consuming `mut self`), `thiserror` per crate, testing gate (`cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features`), no co-author lines in commits, update ferro-mcp when framework behavior changes.
- `.planning/codebase/CONVENTIONS.md` — crate conventions, error patterns.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `schemars::JsonSchema` derive on every built-in `*Props` struct — Catalog reads schemas directly via `schema_for!`.
- `schemars::JsonSchema` on `Action`, `Visibility`, `Size`, `ButtonVariant`, etc. — enum variants become JSON Schema `enum` arrays automatically.
- `BUILTIN_TYPES: &[&str]` in `render/mod.rs` — canonical list for Catalog drift check.
- `JsonUiPlugin::props_schema() -> serde_json::Value` — already on the trait; Catalog consumes as-is without requiring plugins to adopt schemars.
- `PluginRegistry::registered_types()` — returns sorted plugin type names for Catalog plugin discovery.
- `OnceLock<RwLock<…>>` pattern from `global_plugin_registry` — replicate for `global_catalog`.

### Patterns to Replicate
- **Builder-less globals via `OnceLock`.** Catalog is global, once, frozen — no `CatalogBuilder` needed; the static `BUILTIN_SPECS` table IS the "builder config." `Catalog::build() -> Result<Catalog, CatalogError>` constructs once.
- **`thiserror` per crate.** `CatalogError` joins `SpecError` as the second error type in ferro-json-ui.
- **Public surface via `lib.rs` re-exports.** `pub use catalog::{Catalog, CatalogError, ComponentSpec, global_catalog};`.

### Integration Points
- `ferro_json_ui::global_catalog()` → new public function; entry point for every consumer.
- `framework/src/bin/ferro.rs` — unified binary gets a `json-ui:schema` subcommand that calls `global_catalog().json_schema()` or `.component_schema(name)`.
- `ferro-cli/src/commands/json_ui_schema.rs` — new CLI shell-out file (follows `db_status.rs` pattern).
- `ferro-mcp/src/tools/json_ui_catalog.rs` — same public Protobuf-like struct output, body rewired to Catalog.
- `ferro-mcp/src/tools/json_ui_generate.rs` — system prompt source swap.

### Non-obvious behaviors to preserve
- **`COMPONENT_CATALOG` current size is ~4.5 KB.** `catalog.prompt()` targets the same envelope; growth to ~8 KB is acceptable but warrants a log warning at build.
- **Plugin schemas are untrusted.** A plugin returning a malformed `serde_json::Value` breaks Catalog build. Surface as `CatalogError::BuildFailed` with plugin name; do not panic.
- **`BUILTIN_TYPES.len() == 39` asserted in `render/mod.rs::builtin_types_count_matches_dispatch`.** Catalog drift check parallels this — `BUILTIN_SPECS.len() == BUILTIN_TYPES.len()` asserted at build and in unit test.

### Non-obvious behaviors to drop
- **Hand-maintained component/prop/variant metadata in `ferro-mcp/src/tools/json_ui_catalog.rs`.** All prop lists and variant lists get dropped — Catalog supplies them. The hand-written `builder_api` / `action_api` DSL docs strings stay (they document HOW to author, not component shapes).

</code_context>

<specifics>
## Specific Ideas

- **Catalog is the contract for every downstream AI/validator.** Phase 117.1 drives ServiceDef → Spec generation through `component_schema()`. Phase 119 validates loaded Specs via `validate()`. Phase 120 prompts LLMs with `prompt()`. Phase 120 also uses `component_schema()` as an OpenAI / Anthropic structured-output schema. Getting Catalog right matters disproportionately.
- **Prompt vs. schema is the key architectural distinction.** `catalog.prompt()` is HUMAN-targeted prose (and AI context-window-optimized). `catalog.json_schema()` is MACHINE-targeted JSON (IDEs, validators, external tooling). They are not interchangeable and they are generated by different methods on the Catalog. AI agents reading the prompt will NEVER see the raw schema because it's 10× too large — that's the entire point of the split.
- **Discriminated union via `oneOf` + `"type": { "const": "X" }` pattern.** Every entry in the `elements.*` oneOf pins its `type` field to a const string. This is the idiomatic JSON Schema pattern for "sum types"; every modern validator (AJV, jsonschema crate, OpenAPI) understands it. The `oneOf` is linear in the number of variants, but the pre-dispatch by `type_name` collapses validation to `O(1 + props_schema_size)` per element.
- **`OnceLock` is the right shape for a v1.0 runtime catalog.** No lazy-plugin-hot-swap. No thread-local variants. Plugins register at app startup, Catalog builds on first access, and the whole thing stays valid for the life of the process. If a future phase needs dynamism, swap to `RwLock<Arc<Catalog>>` — but not now.
- **Deleting `COMPONENT_CATALOG` is the visible kill of the old regime.** Consumers grep for it, find zero hits, follow the trail to `global_catalog().prompt()`. This is a clean-break success metric. No re-export shim, no deprecation warning, no migration window.
- **Prompt output ordering matters.** Sort components alphabetically (after an optional "atoms first / containers second / form third / data fourth" bucketing if the generated output reads better that way). Deterministic output means committing a snapshot test is meaningful.

</specifics>

<deferred>
## Deferred Ideas

- **Phase 117.5: slot-ID graph validation.** Walking `CardProps.footer`, `Tab.children`, etc. to confirm each ID resolves to an element in `spec.elements`. Phase 117 is scoped to schema + type validation; slot-graph validation is semantic. Either extend `Spec::from_json` or add `Catalog::validate_slots(&spec)` in a follow-up plan.
- **Per-component validator precompilation.** Upgrade `HashMap<String, Validator>` if profiling shows per-call `validator_for` overhead. Escape hatch per CONTEXT D-12.
- **Runtime plugin hot-swap / Catalog rebuild.** Deferred per CONTEXT D-35. Revisit if a concrete use case appears.
- **Plugin schema validation (is the plugin's schema itself a valid JSON Schema).** Phase 117 trusts `props_schema()` output; a future audit could run `jsonschema::meta::validator()` against plugin schemas.
- **Catalog diff tool.** Produce a human-readable "what changed between catalog v1 and v2" output for release notes. Nice-to-have; defer.
- **IDE plugin that consumes the exported JSON Schema** (VS Code LSP, JetBrains) — v1.0+ direction, not Phase 117.
- **Schema versioning / `$id` URL resolution.** Phase 117 sets `$id: "ferro-json-ui/v2"` as an identifier but does not host a resolvable URL. Future phase sets up schema hosting if needed.
- **Two-tier AI generation strategy** — Phase 120.
- **Full JSON-UI docs rewrite referencing the new Catalog surface** — Phase 121.
- **Catalog description authoring macro** (to promote `#[doc = "..."]` attributes into descriptions). Not worth the complexity for Phase 117.

</deferred>

---

*Phase: 117-catalog-and-json-schema*
*Context gathered: 2026-04-18*
*Mode: --auto*
