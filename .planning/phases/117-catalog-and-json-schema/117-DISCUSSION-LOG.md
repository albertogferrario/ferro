# Phase 117: Catalog & JSON Schema - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md; this log preserves alternatives considered.

**Date:** 2026-04-18
**Phase:** 117-catalog-and-json-schema
**Mode:** `--auto` — all gray areas auto-selected with recommended defaults informed by Vercel json-render catalog, jsonforms schema-as-truth, rjsf per-component schema pattern, shadcn-ui documented-component model.
**Areas discussed:** Catalog crate boundary, catalog shape, discovery mechanism, validation pipeline, full schema assembly, prompt generation, CLI surface, consumer migration, testing.

---

## Crate boundary — where does Catalog live?

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro-json-ui/src/catalog.rs` — same crate as renderer + Props + BUILTIN_TYPES | Drift between dispatch list, Props types, and Catalog becomes a compile error. Simplest import graph. | ✓ |
| New `ferro-catalog` crate | Pure layering, but adds a workspace member for ~1000 LOC. No runtime benefit. | |
| `ferro-projections` | Mis-scoped — projections depend on catalog, not the other way around. | |

**Selected:** same crate. **Rationale:** locality of BUILTIN_TYPES, zero import churn, catalog and renderer evolve together.

---

## Catalog shape — eager build vs. lazy

| Option | Description | Selected |
|--------|-------------|----------|
| Eager build at first access via `OnceLock<Catalog>`; frozen after | Matches `global_plugin_registry()` pattern; Catalog is small so eager is fine. | ✓ |
| Lazy per-method (build `full_schema` only when requested, etc.) | Micro-optimization with no measurable payoff; harder to reason about thread-safety. | |
| Fully static `const`/`static` | schemars output is runtime-computed; can't be const. | |

**Selected:** eager + `OnceLock`. **Rationale:** tiny data, matches existing workspace patterns, no threading complications.

---

## Component discovery — how does Catalog know about Props types?

| Option | Description | Selected |
|--------|-------------|----------|
| Static `BUILTIN_SPECS: &[(name, description, schema_fn, slot_fields)]` table | Auditable, grep-friendly, one-line-per-component. Drift guard via unit test against BUILTIN_TYPES. | ✓ |
| Proc macro over `component.rs` to auto-collect all `*Props` structs | Magical, hidden, adds compile time. | |
| Build script reading component.rs and emitting static data | Same downsides plus build-script fragility. | |

**Selected:** static table. **Rationale:** explicit, reviewable, easy to author descriptions inline.

---

## Validation library

| Option | Description | Selected |
|--------|-------------|----------|
| `jsonschema` crate (Draft 2020-12, matches schemars output) | Per ROADMAP caveat. Compiled validators, pre-dispatch by type possible. | ✓ |
| `valico` | Older, Draft-4/6 only, less maintained. | |
| Hand-roll validation | Re-implements schema semantics; rejected. | |

**Selected:** `jsonschema = "0.28"`. **Rationale:** ROADMAP-specified, 2020-12 alignment with schemars, actively maintained.

---

## Validation pipeline — discriminator optimization

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-dispatch on `el.type_name` (O(1) HashMap check) then full jsonschema validate | Collapses the `oneOf` worst case per ROADMAP caveat. | ✓ |
| Single jsonschema `validate(&spec)` call | Linear in `oneOf` arity per element — slow for 40-variant schema. | |
| Custom validator that hardcodes per-type dispatch | Duplicates jsonschema logic; hard to maintain. | |

**Selected:** pre-dispatch + full validate. **Rationale:** directly implements the ROADMAP's stated optimization.

---

## Prompt vs. schema split

| Option | Description | Selected |
|--------|-------------|----------|
| `catalog.prompt()` = Markdown text (~4–8 KB); `catalog.json_schema()` = full JSON Schema (~40–80 KB) | Matches ROADMAP caveat ("prompt must emit concise text, NOT raw JSON Schema"). | ✓ |
| Emit JSON Schema and let callers trim | Callers lack domain knowledge to trim safely; high risk of broken LLM context. | |
| Emit both from the same method | Mixes concerns; callers always pay the full serialization cost. | |

**Selected:** separate methods. **Rationale:** ROADMAP-mandated; prompt and schema are targeted at different consumers.

---

## Full spec schema assembly

| Option | Description | Selected |
|--------|-------------|----------|
| Hand-assemble `oneOf` from per-component schemas at Catalog::build time | ~40 LOC, deterministic, controllable discriminator shape. | ✓ |
| Macro over component.rs to emit a single top-level JsonSchema impl for Element | Possible but requires custom JsonSchema impl with oneOf manipulation — more complex than hand-assembly. | |
| Skip full-spec schema; export per-component only | Breaks ROADMAP SC-4. | |

**Selected:** hand-assemble. **Rationale:** smallest correct implementation; oneOf+const-discriminator is the idiomatic JSON Schema pattern.

---

## CLI surface

| Option | Description | Selected |
|--------|-------------|----------|
| `ferro json-ui:schema` subcommand with `--output`, `--pretty`, `--component` flags | Matches ROADMAP SC-6; follows existing ferro-cli command patterns. | ✓ |
| Standalone binary `ferro-schema` | Fragments the CLI surface. | |
| MCP tool only (no CLI) | Breaks ROADMAP SC-6 which specifies CLI export. | |

**Selected:** subcommand. **Rationale:** ROADMAP-specified; consistent with ferro CLI.

---

## Consumer migration scope — what else changes?

| Option | Description | Selected |
|--------|-------------|----------|
| Delete `COMPONENT_CATALOG`; rewire ferro-mcp `json_ui_catalog.rs` + `json_ui_generate.rs` + ferro-cli `make_json_view.rs` to `global_catalog()` | Complete the replacement per ROADMAP SC-7 within Phase 117. | ✓ |
| Delete only `COMPONENT_CATALOG`; defer consumer migrations to Phase 120 | Leaves callers broken. | |
| Keep `COMPONENT_CATALOG` as a legacy fallback and add Catalog alongside | Violates "clean break" per CLAUDE.md project norm. | |

**Selected:** full migration in Phase 117. **Rationale:** ROADMAP criterion 7 requires `COMPONENT_CATALOG` replacement; half-migration defers pain.

---

## Plugin schema handling

| Option | Description | Selected |
|--------|-------------|----------|
| Plugin schemas stay opaque `serde_json::Value`; Catalog weaves into `oneOf` without deep validation | Preserves plugin author freedom; low-risk. | ✓ |
| Require plugins to adopt schemars | Breaking trait change; excludes existing plugins. | |
| Reject plugins lacking valid schemas at build time | Too strict; plugins may legitimately return JSON Schema that fails our meta-check. | |

**Selected:** opaque passthrough. **Rationale:** mirrors CONTEXT D-17 (Phase 116) plugin untrusted-but-functional stance.

---

## Slot-ID graph validation scope

| Option | Description | Selected |
|--------|-------------|----------|
| Document slot-ID graph validation as a KNOWN gap in Phase 117; defer to Phase 117.5 or a follow-up plan | Keeps Phase 117 focused on schema+type validation (ROADMAP scope). | ✓ |
| Bundle slot-graph validation into `catalog.validate(&spec)` | Extends scope; correct per domain but unclear if ROADMAP SC-3 covers it. | |
| Punt entirely; slot dangling stays a render-time HTML comment | Silently permits authoring bugs. | |

**Selected:** document + defer. **Rationale:** tightens Phase 117 scope to what ROADMAP specifies; explicit follow-up avoids scope creep.

---

## Catalog::build error surface

| Option | Description | Selected |
|--------|-------------|----------|
| `Catalog::build() -> Result<Catalog, CatalogError>` | Lets Phase 120 MCP tools surface plugin schema failures gracefully. | ✓ |
| Panic on build failure (consistent with `OnceLock::get_or_init`) | Hard to recover; bad UX for CLI tools. | |

**Selected:** `Result`. **Rationale:** observable failures > silent panics; `global_catalog()` can wrap with `unwrap_or_else(|e| eprintln!(...))`.

---

## Claude's Discretion

- Exact file split (single catalog.rs vs. catalog/ directory) — start single-file, split above ~1200 LOC.
- Prompt sort order (alphabetical vs. bucketed atoms/containers/form/data) — pick whatever reads cleaner.
- Per-component validator caching (on-demand vs. precompiled HashMap) — start on-demand, upgrade if profiling demands.
- `jsonschema` crate version within 0.28.x — latest compatible.

## Deferred Ideas (captured from this discussion)

- Phase 117.5 slot-ID graph validation.
- Per-component validator precompilation HashMap.
- Runtime plugin hot-swap / Catalog rebuild.
- Plugin schema meta-validation.
- Catalog diff tool.
- IDE / LSP plugin consuming exported JSON Schema.
- Schema `$id` URL hosting.
- Two-tier AI generation (Phase 120).
- Docs rewrite (Phase 121).
