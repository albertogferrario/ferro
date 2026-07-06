---
status: passed
phase: 117-catalog-and-json-schema
verified: 2026-04-18T00:00:00Z
plans_executed: 7
success_criteria_passed: 8 of 8
---

# Phase 117 Verification — Catalog & JSON Schema

## Goal Achievement

Phase 117 delivered its goal. The hand-maintained `COMPONENT_CATALOG` constant string is gone; in its place lives a machine-readable `Catalog` in `ferro-json-ui/src/catalog.rs` that auto-discovers all 39 built-in component variants and live plugins via `schemars::schema_for!`, pre-computes per-component Props schemas and a full v2 spec schema, compiles a `jsonschema::Validator` exactly once at build time, validates specs through a 3-stage pipeline (type-name whitelist → per-element props → envelope), exposes a concise text system prompt under the 8 KB budget, and ships a `ferro json-ui:schema` CLI subcommand for schema export. All three downstream consumers (`ferro-cli/src/ai.rs`, `ferro-mcp/src/tools/json_ui_catalog.rs`, `ferro-mcp/src/tools/json_ui_generate.rs`) are migrated to `global_catalog()`. Workspace fmt/clippy/tests are green.

## Success Criteria Matrix

| SC # | Description | Evidence | Status |
|------|-------------|----------|--------|
| 1 | `Catalog::build()` auto-discovers all Component variants with descriptions and JSON Schema | `ferro-json-ui/src/catalog.rs:520` defines `pub fn build()`; `BUILTIN_SPECS` table (lines 123–362) contains exactly 39 entries matching `crate::render::BUILTIN_TYPES`; runtime drift guard at lines 524–531; build loop populates `components` + `per_component_schemas` via `schema_for!` on each Props struct; plugin loop meta-validates via `jsonschema::validator_for`. Tests `build_populates_all_builtins`, `builtin_specs_len_matches_dispatch` (== 39), `builtin_specs_names_match_dispatch`, `build_discovers_plugins_and_rejects_invalid_schema` all pass. | PASS |
| 2 | `catalog.prompt()` generates concise text system prompt (NOT JSON Schema) | `ferro-json-ui/src/catalog.rs:768` defines `pub fn prompt(&self) -> String` producing Markdown (`## Component Catalog`, `### <Name>` sections, `Props:` / `Slots:` lines). Test `prompt_under_size_budget` asserts ≤ 8 KB; `prompt_is_not_raw_json_schema` asserts no `"$schema"` substring and Markdown prefix; `prompt_mentions_every_builtin`, `prompt_is_deterministic`, `prompt_documents_slot_fields` all pass. | PASS |
| 3 | `catalog.validate(&spec)` uses `jsonschema` crate, compiled validator, pre-dispatches by `"type"` string | `ferro-json-ui/src/catalog.rs:630` defines `pub fn validate(&self, spec: &Spec)`. Stage 1 (lines 633–650) type-name whitelist short-circuits on unknowns; Stage 2 per-element Props validation; Stage 3 envelope via cached `self.validator`. Tests `validate_positive_per_type`, `validate_unknown_type`, `validate_missing_required_prop`, `validate_bad_schema_version`, `validate_pre_dispatch_short_circuits`, `validate_accumulates_multiple_errors_across_elements` all pass. | PASS |
| 4 | `catalog.json_schema()` exports complete JSON Schema for full v2 spec (root + elements + oneOf) | `ferro-json-ui/src/catalog.rs:607` defines `pub fn json_schema(&self) -> &Value`. `assemble_full_schema` (lines 428–505) emits `$schema`, `$id: "ferro-json-ui/v2"`, required `[$schema, root, elements]`, and `$defs/Element` with `oneOf` over all 39 components (each variant pins `type: {const: X}`). Tests `json_schema_has_spec_envelope_shape`, `json_schema_has_action_and_visibility_defs`, `json_schema_oneof_covers_all_builtins`, `json_schema_is_valid` (Draft 2020-12 meta-validation), `oneof_variants_are_deterministic_sorted` all pass. | PASS |
| 5 | `catalog.component_schema("Card")` returns JSON Schema for single component's Props | `ferro-json-ui/src/catalog.rs:727` defines `pub fn component_schema(&self, type_name: &str) -> Option<&Value>`. Returns Props-only schema (not Element envelope). Tests `component_schema_returns_props_only` (checks Card has `title` in properties, rejects Element wrapper shape), `component_schema_none_for_unknown`, `component_schema_resolves_every_builtin` all pass. | PASS |
| 6 | `ferro json-ui:schema` CLI command exports schema to stdout or file | (a) `framework/src/app.rs:83-98` defines the `JsonUiSchema` clap variant with `#[command(name = "json-ui:schema")]`, handler at `framework/src/app.rs:310-357` calls `Catalog::build()` then `json_schema()` or `component_schema(name)` and writes to stdout or `--output` file. (b) `ferro-cli/src/main.rs:317-331` declares the same clap variant; dispatch arm at lines 647-653. (c) `ferro-cli/src/commands/json_ui_schema.rs` exists as the shell-out wrapper. | PASS |
| 7 | `COMPONENT_CATALOG` const string is replaced by `catalog.prompt()` output | Workspace-wide grep for `COMPONENT_CATALOG` across all `.rs` files returns **0 matches** (Grep: 0 files, 0 occurrences). `ferro-cli/src/ai.rs:91` uses `global_catalog().prompt()`; `ferro-mcp/src/tools/json_ui_generate.rs:123` uses `global_catalog().prompt()`; `ferro-mcp/src/tools/json_ui_catalog.rs:46-47` builds from `global_catalog()`. Commit `b5911f9d` titled "refactor(117-06): delete COMPONENT_CATALOG const from ferro-json-ui" confirms the removal. | PASS |
| 8 | Schema validator compiled once in `Catalog::build()` and reused (no per-validation compilation) | `ferro-json-ui/src/catalog.rs:76` declares `pub(crate) validator: jsonschema::Validator` as a field on `Catalog`. Built exactly once at `catalog.rs:589` via `jsonschema::validator_for(&full_schema)` inside `Catalog::build()`. `validate()` reuses via `self.validator.iter_errors(&spec_value)` at line 698 (no recompilation). Tests `validator_is_compiled_once_and_usable`, `validator_rejects_wrong_schema_version`, `validator_is_cached_not_recompiled` (100-iteration reuse) all pass. | PASS |

## Requirement Traceability

| Req ID | Description | Plan | Status |
|--------|-------------|------|--------|
| CAT-01 | Machine-readable Catalog auto-discovering Component variants with descriptions and per-component JSON Schema | 117-01 (scaffold), 117-02 (discovery impl) | Covered |
| CAT-02 | Concise text system prompt via `catalog.prompt()`, not raw JSON Schema; consumers migrated | 117-06 | Covered |
| CAT-03 | `catalog.validate(&spec)` pipeline with typed errors, pre-dispatch by `"type"`, compiled validator reuse | 117-04 | Covered |
| CAT-04 | Per-component schema accessor `component_schema(name)` plus sorted iterators | 117-05 | Covered |
| SCHEMA-01 | Full v2 spec JSON Schema document: root + elements + `oneOf` over all component variants | 117-03 | Covered |
| SCHEMA-02 | `ferro json-ui:schema` CLI export to stdout or file, optional `--component` filter | 117-07 | Covered |
| SCHEMA-03 | Validator compiled once, reused across all `validate()` calls | 117-03 (compile in build), 117-04 (reuse in validate) | Covered |

## Test / Clippy / Fmt Results

| Check | Command | Result |
|-------|---------|--------|
| Format | `cargo fmt --all -- --check` | exit 0 |
| Lint | `cargo clippy --all --all-targets --all-features -- -D warnings` | exit 0 |
| Tests | `cargo test --all-features` | exit 0 — all test binaries report `ok.`, zero failures across the full workspace (representative counts: framework 473 + 444 + 385 + 229 + 203 passed; ferro-json-ui suite passes; catalog module tests including 12 new Plan-03/04/05/06 additions all green) |

Key catalog-specific assertions verified:
- `BUILTIN_SPECS.len() == 39` and matches `BUILTIN_TYPES` set (drift guard)
- `per_component_schemas.len() == BUILTIN_SPECS.len() + plugin_components.len()`
- `prompt().len() <= 8192` (8 KB budget from CONTEXT D-17)
- Meta-validation: assembled `full_schema` is valid Draft 2020-12
- Byte-stable output: two builds produce identical `json_schema()` serialization

## Gaps Found

None.
