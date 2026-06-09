# Phase 173: make:json-view v2 + projection-roundtrip test — Research

**Researched:** 2026-06-09
**Domain:** ferro-cli command integration + ferro-json-ui projection pipeline + ferro-ai offline testing
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `make:json-view`'s spec generation routes through the existing
  `Spec::from_service_def(service, &intent_scores, ctx)` — deterministic,
  `FieldMeaning`/`Intent`-driven component selection via `ferro_projections::derive_intents()`.
  The LLM does not re-prompt about field types or pick components.
- **D-02:** The generated spec is validated against `catalog.json_schema()` before write
  (already implemented — preserve), and contains no v1 `JsonUiView` types (SC4).
- **D-03:** `make:json-view`'s AI path becomes a two-stage projection flow:
  NL description → `ServiceDef` (reuse Phase 171 `ai:make` production logic via
  `ferro_mcp::tools::ai_scaffold::scaffold_core`) → `Spec::from_service_def` → catalog
  validation. The current direct NL→spec two-pass (`generate_with_ai` in
  `make_json_view.rs`) is **deleted**, not kept in parallel.
- **D-04:** `make:json-view` also accepts a `ServiceDef` already present in the project
  (not only freshly AI-produced). Exact flag spelling is Claude's discretion.
- **D-05 (Claude's discretion):** `catalog.component_schema()` is used only if a residual
  LLM refinement pass survives. Default: no residual LLM pass. If the deterministic path
  needs no per-component LLM call, SC1's `component_schema()` clause is satisfied
  vacuously — document that in VERIFICATION.md rather than inventing an LLM pass.
- **D-06:** New test `ferro-ai/tests/projection_roundtrip.rs`, offline, deterministic.
  Structure: fixed `ServiceDef` fixture → assert `derive_intents()` + `FieldMeaning`/`ActionDef`
  shape → run through `Spec::from_service_def` → validate against `catalog.json_schema()` →
  assert the `ServiceDef`-aware path (not a generic fallback).
- **D-07:** Live NL→`ServiceDef` quality is a manual verification gate (173-VERIFICATION.md),
  not an automated CI test. The automated roundtrip test does not depend on a live key.

### Claude's Discretion

- Exact `make:json-view` flag/arg spelling for "render an existing ServiceDef" (D-04)
  and how a project `ServiceDef` is located/loaded.
- Whether any residual LLM refinement pass is retained (D-05) — default: no.
- The fixture-vs-mock-LlmClient choice for the roundtrip test (D-06) — default:
  whichever keeps the test offline and deterministic with least machinery.
- Exact `intent_scores` construction at the `make:json-view` call site.

### Deferred Ideas (OUT OF SCOPE)

- Additional non-visual `Renderer`s over `ServiceDef` (conversational, voice, API) —
  v14.0 Channel Projection direction.
- Live-LLM roundtrip in CI (real provider key in the test matrix).
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AICLI-04 | `ferro make:json-view` consumes a `ServiceDef` and renders via the existing deterministic renderer (`Spec::from_service_def`), replacing the direct NL→spec two-pass | Verified: `Spec::from_service_def` exists at `builder.rs:54`; `generate_with_ai` in `make_json_view.rs` is the function to delete; `scaffold_core` in `ferro-mcp` is the NL→ServiceDef function to reuse |
| AICLI-06 | Projection-roundtrip test at `ferro-ai/tests/projection_roundtrip.rs`: NL→ServiceDef→rendered spec→catalog validation, offline, deterministic, goes through ServiceDef-aware path | Verified: test infrastructure, mock pattern (`ConstClient` in `complete.rs`), and the fixture-based approach are all already present; the test calls `Spec::from_service_def_with_catalog` for isolation |
</phase_requirements>

---

## Summary

Phase 173 is an integration phase. Almost all the machinery exists and is already tested in isolation; this phase wires it together and proves the connection with an automated test.

**The three deliverables:**

1. **CLI upgrade (AICLI-04):** Delete `generate_with_ai` from `make_json_view.rs` and replace with a two-stage path: call `scaffold_core` (NL→ServiceDef, reusing Phase 171 logic) then `Spec::from_service_def` (deterministic render). Add a `--from-service <name>` flag that locates an existing `ServiceDef` source file via the `list_projections` scan and constructs a `ServiceDef` directly in-code (no source file parsing needed — see § "Loading an existing ServiceDef" below).

2. **Roundtrip test (AICLI-06):** `ferro-ai/tests/projection_roundtrip.rs`, structured exactly like the sibling `projection_schema.rs`. The test constructs a `ServiceDef` fixture in-test (no LLM needed), calls `derive_intents` + `Spec::from_service_def_with_catalog`, validates the spec, and asserts a FieldMeaning-driven component type to prove the ServiceDef-aware path was taken.

3. **SC1/SC5 documentation (D-05):** `component_schema()` role resolved vacuously — the deterministic builder selects components without any LLM call; SC1 is satisfied by design. VERIFICATION.md records this.

**Primary recommendation:** Reuse `from_service_def_with_catalog` (not the public `from_service_def`) in the roundtrip test to avoid `OnceLock` pollution from sibling catalog tests — this is the established pattern in `builder.rs` tests and `mod.rs` tests.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| NL→ServiceDef production | `ferro-mcp` (`scaffold_core`) | `ferro-cli` (tokio bridge) | Phase 171 established: all AI logic lives in the MCP core; CLI is a thin bridge |
| ServiceDef→Spec rendering | `ferro-json-ui` (`Spec::from_service_def`) | — | The deterministic renderer lives here; it is the shipped Phase 117.1 implementation |
| Spec validation (write gate) | `ferro-json-ui` (`catalog.json_schema()`) | — | Catalog already owns the write gate; preserve it unchanged |
| ServiceDef file location | `ferro-mcp` (`list_projections`) | `ferro-cli` (name→path resolution) | ServiceDefs live at `src/projections/{name}.rs`; the list_projections scanner finds them |
| Roundtrip proof test | `ferro-ai/tests/` | `ferro-json-ui` (via dev-dep) | Test lives in ferro-ai alongside its sibling; needs ferro-json-ui as a dev-dependency |
| CLI argument routing | `ferro-cli/src/main.rs` + `make_json_view.rs` | — | Clap variant needs a new optional `--from-service` arg |

---

## Standard Stack

### Core (already in workspace — no new deps)

| Library | Purpose | Location |
|---------|---------|----------|
| `ferro-projections` | `ServiceDef`, `derive_intents()`, `FieldMeaning`, `Intent`, `IntentScore` | already dep of `ferro-cli` (feature `projections`), `ferro-json-ui`, `ferro-ai` |
| `ferro-json-ui` | `Spec::from_service_def`, `Spec::from_service_def_with_catalog`, `Catalog`, `global_catalog`, `VisualContext` | already dep of `ferro-cli` |
| `ferro-mcp` | `tools::ai_scaffold::scaffold_core` | already dep of `ferro-cli` |

### New Dependency (roundtrip test only)

| Dependency | Type | Where | Why |
|------------|------|-------|-----|
| `ferro-json-ui` (with feature `projections`) | dev-dependency | `ferro-ai/Cargo.toml` | Roundtrip test calls `Spec::from_service_def_with_catalog`; currently not a dep of ferro-ai |

Verify current ferro-ai dev-dependencies:

```
ferro-ai/Cargo.toml dev-dependencies: tokio (full + test-util), jsonschema 0.46
```

Need to add: `ferro-json-ui = { path = "../ferro-json-ui", version = "0.2", features = ["projections"] }` [VERIFIED: codebase inspection]

---

## Research Findings by Question

### Q1: Exact signature and contract of `Spec::from_service_def`

**Signature (verified at `ferro-json-ui/src/projection/builder.rs:54`):**

```rust
pub fn from_service_def(
    service: &ServiceDef,
    intents: &[IntentScore],
    ctx: &VisualContext,
) -> Result<Spec, ProjectionError>
```

**Contract:**
- Returns `Err(ProjectionError::EmptyIntents)` if `intents` is empty — checked BEFORE touching `global_catalog()`, so the OnceLock is not poisoned.
- Returns `Err(ProjectionError::IntentIndexOutOfBounds { requested, available })` if `ctx.intent_index >= intents.len()`.
- On any path, calls `global_catalog().validate(&spec)` before returning `Ok`. In `debug_assertions`, a catalog validation failure panics loudly. In release, it returns `Err(ProjectionError::CatalogValidation(errors))`.
- `VisualContext::default()` is `{ intent_index: 0, current_state: None, mode: RenderMode::Display, templates: None }` — safe default for a Browse/Display projection.

**Test-friendly variant (for the roundtrip test):**

```rust
pub(crate) fn from_service_def_with_catalog(
    service: &ServiceDef,
    intents: &[IntentScore],
    ctx: &VisualContext,
    catalog: &Catalog,
) -> Result<Spec, ProjectionError>
```

This is `pub(crate)` inside `ferro-json-ui`. The roundtrip test must live inside `ferro-json-ui` OR the test can call the public `from_service_def` and use `global_catalog()`. The OnceLock pollution concern applies only when sibling tests register the `BadPlugin_117` plugin into the global catalog. Because `projection_roundtrip.rs` is in `ferro-ai/tests/`, it runs in a separate test binary from the `ferro-json-ui` test suite — there is no OnceLock pollution risk from the plugin registration. **The public `from_service_def` (which calls `global_catalog()`) is safe to use from `ferro-ai/tests/`.** [VERIFIED: codebase inspection + test binary isolation reasoning]

**Call pattern at `make:json-view` call site:**

```rust
use ferro_projections::derive_intents;
use ferro_json_ui::{Spec, VisualContext};

let intents = derive_intents(&service);
let ctx = VisualContext::default(); // Browse+Display, intent_index=0
let spec = Spec::from_service_def(&service, &intents, &ctx)?;
```

---

### Q2: Extracting NL→ServiceDef logic from `ai_make.rs` for reuse

**Current structure of `ai_make.rs` (verified by reading the file):**

- `pub(crate) fn emit_service_def_source(service: &ServiceDef) -> String` — the Rust-source emitter. Used by `render_output`. This is NOT the NL→ServiceDef production logic.
- `pub fn run(description: String, dry_run: bool)` — the CLI entry point. It calls `scaffold_core` from `ferro-mcp`.

The NL→ServiceDef production logic is **not in `ai_make.rs` at all** — it was moved to `ferro_mcp::tools::ai_scaffold::scaffold_core` during Phase 171. `ai_make.rs::run` is already a thin bridge.

**The reuse pattern for `make:json-view` is identical to `ai_make.rs::run`:**

```rust
// In make_json_view.rs (new NL path)
let service = rt.block_on(
    ferro_mcp::tools::ai_scaffold::scaffold_core(&description, &cwd)
)?;
```

**No extraction needed.** `make:json-view` simply calls the same `scaffold_core` function. The only structural change is that after getting the `ServiceDef`, the code calls `Spec::from_service_def` instead of `emit_service_def_source`. [VERIFIED: reading `ai_make.rs` fully]

**Tokio runtime pattern:** The existing `generate_with_ai` in `make_json_view.rs` already creates a `tokio::runtime::Runtime::new()` for the two-pass LLM calls. The same pattern is used for the new NL→ServiceDef call. Reuse the existing runtime creation code verbatim.

---

### Q3: Making the roundtrip test offline/deterministic

**Recommended approach: in-test `ServiceDef` fixture (no mock LlmClient needed)**

The roundtrip test does not need to test NL→ServiceDef — that is Phase 171's concern. It tests ServiceDef→Spec. The fixture approach is:

1. Construct a `ServiceDef` directly in-test using the builder API (same pattern as `builder.rs` tests and `projection_schema.rs`).
2. Call `derive_intents(&service)` → deterministic, no network.
3. Call `Spec::from_service_def(&service, &intents, &VisualContext::default())` → deterministic, no network.
4. Validate against `catalog.json_schema()` using `jsonschema::draft202012`.
5. Assert component types.

**Existing mock LlmClient patterns (for reference, NOT needed here):**

`ferro-ai/src/complete.rs` contains `ConstClient(String)` — a mock that returns a fixed JSON string. This pattern would only be needed if the test wanted to exercise `scaffold_core`'s LLM call, which it does not (per D-07).

**Why the fixture approach is correct:** The ROADMAP SC5 says "the test passes via the `ServiceDef`-aware path; it cannot pass via the generic schema-normalization fallback." A constructed `ServiceDef` fixture guarantees we take the `from_service_def` path by construction — there is no LLM-produced JSON that could have gone through a different pipeline. The assertion on component types (see Q5) provides the additional path proof.

---

### Q4: Resolving the SC1-vs-SC3 tension (D-05)

**Resolution (confirmed by code inspection):**

`Spec::from_service_def` selects components via `lookup_meaning(&field.meaning)` in `component_map.rs` — a pure match table, no LLM. The catalog's `json_schema()` is used as the **write-gate validator** (SC2/D-02), not as an LLM prompt for component selection.

`catalog.component_schema("Card")` would only be needed if an LLM were being asked "generate props for a Card component" — which the deterministic builder never does. SC1's language ("uses `catalog.component_schema()` for per-component structured output") predates the Phase 117.1 deterministic builder and describes the old two-pass approach.

**Recommended resolution for VERIFICATION.md:**

> SC1 is satisfied by design: the deterministic builder selects components via `FieldMeaning`→component dispatch (`lookup_meaning` in `component_map.rs`) and validates the complete spec against `catalog.json_schema()` (SC2). `catalog.component_schema()` is the per-component schema for LLM-constrained generation; since the builder does not call the LLM for component selection, no per-component schema prompt is needed. No LLM pass is added to satisfy SC1.

---

### Q5: Asserting the ServiceDef-aware path in SC5

**Observable that distinguishes the two paths:**

The ServiceDef-aware path (`Spec::from_service_def`) maps `FieldMeaning` to specific component types via `lookup_meaning`. A generic NL→spec path would produce component types based on what the LLM guessed.

**Concrete assertion strategy:**

Construct a `ServiceDef` with a `FieldMeaning::Money` field. The `build_column_for_field` function maps `Money → ColumnFormat::Currency` in the DataTable column. No generic LLM pass would reliably produce `ColumnFormat::Currency` — it is a direct consequence of the `FieldMeaning` dispatch.

```rust
// In the roundtrip test — assert ServiceDef-aware path was taken:
let service = ServiceDef::new("invoice")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("total", DataType::Float, FieldMeaning::Money)     // money → currency format
    .field("name", DataType::String, FieldMeaning::EntityName);

let intents = derive_intents(&service);
// Browse intent → DataTable
let browse_idx = intents.iter().position(|i| matches!(i.intent, Intent::Browse)).unwrap_or(0);
let ctx = VisualContext { intent_index: browse_idx, ..VisualContext::default() };
let spec = Spec::from_service_def(&service, &intents, &ctx).unwrap();

// SC5: the spec schema matches the catalog contract
assert!(global_catalog().validate(&spec).is_ok(), "spec must pass catalog validation");

// SC5 path proof: Money field → column with ColumnFormat::Currency in DataTable
// This would not be produced by a generic NL→spec LLM pass
let root = spec.elements.get(&spec.root).expect("root element");
assert_eq!(root.type_name, "DataTable", "Browse intent must produce DataTable root");
// Verify at least one column exists (Money field was not excluded as system field)
// Identifier IS excluded as a system field; Money and EntityName columns appear
let cols_json = root.props.get("columns").expect("DataTable must have columns");
let cols = cols_json.as_array().expect("columns is array");
assert!(!cols.is_empty(), "must have at least one column");
// The Money column should have format = "currency"
let has_currency = cols.iter().any(|c| c.get("format").and_then(|f| f.as_str()) == Some("currency"));
assert!(has_currency, "Money field must produce a currency-formatted column — proves ServiceDef-aware path");
```

The `has_currency` assertion **cannot pass via a generic schema-normalization fallback** because:
- The generic schema-normalizer produces constraints for LLM prompts, not a `Spec`.
- A `Spec` is only produced by `Spec::from_service_def` or by parsing a JSON string — and neither the generic fallback nor any catalog-schema-only path would derive `ColumnFormat::Currency` from `FieldMeaning::Money`. [VERIFIED: component_map.rs line 277]

---

### Q6: SC4 — v1 `JsonUiView` types in the pipeline

**Verified finding:** v1 types were deleted in Phase 160. [VERIFIED: ROADMAP Phase 160 entry, Phase 115 plan which deleted `view.rs` and v1 types]

`make_json_view.rs` already:
- Uses `Spec::from_json(&json_str)` (v2), not any v1 type.
- Calls `global_catalog().validate(&spec)` (v2 catalog).
- Outputs JSON file content, not a Rust `JsonUiView` struct.

SC4 verification is a grep-based audit: `rg "JsonUiView" ferro-cli/` must return zero matches in the generation pipeline after D-03 is applied.

---

### Q7: Loading an existing project ServiceDef (D-04)

**How ServiceDefs are persisted:** `ferro ai:make` writes a Rust source file to `src/projections/{name}.rs` containing a `pub fn {name}_service() -> ServiceDef { ... }` function. There is no companion JSON file. ServiceDef is `Serialize + Deserialize` but the CLI does not write JSON.

**The "existing project file" loading challenge:** The Rust source file cannot be dynamically loaded at runtime by another CLI command without compiling it. The `ferro-mcp` tools (e.g., `inspect_projection`, `list_projections`) scan these files via regex to extract metadata — but they do NOT produce a live `ServiceDef` value, only a text summary.

**Recommended implementation for `--from-service <name>` (D-04):**

Since the ServiceDef source file is Rust builder code, and runtime dynamic loading is not feasible within the CLI binary, the `--from-service <name>` flag should use a **JSON sidecar file** strategy:

- `ferro ai:make` already has `--dry-run` which prints `serde_json::to_string_pretty(service)` — the full ServiceDef as JSON.
- `--from-service <name>` could accept either:
  - A path to a `.json` file containing a serialized ServiceDef (`serde_json::from_str::<ServiceDef>(&content)`).
  - OR (simpler): the `list_projections` scan gives the file path → read the `.rs` source → if the service was previously saved as JSON via `ferro ai:make --dry-run > service.json`, load that.

**Simplest viable interpretation (recommended):** Accept a path to a JSON file `--from-service-json <path>` that deserializes directly into `ServiceDef`. This avoids Rust-source parsing entirely and is consistent with the dry-run output format. Alternatively, accept a service name and look for `src/projections/{name}.json` (a sidecar JSON, distinct from the Rust source).

**The planner should decide the exact flag:** The behavior contract (given a ServiceDef, render deterministically, no LLM call) is locked; the arg shape is discretion.

**Alternative (simpler but less general):** If the planner determines the "existing project file" path is out-of-scope for v12.1 capstone and only the NL→ServiceDef path matters for the roundtrip proof, skip `--from-service` entirely. The CONTEXT.md D-04 says it "also accepts" a pre-existing ServiceDef as an option, not as the primary path. The roundtrip test (AICLI-06) exercises the NL path (with a fixture); the deterministic render path is proven by the same test regardless.

---

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│  ferro make:json-view <name> -d "<description>"                     │
│                        (or --from-service-json <path>)              │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
               ┌────────────▼────────────┐
               │  Path A: NL→ServiceDef  │  Path B: Load ServiceDef
               │  scaffold_core(desc)    │  serde_json::from_file(path)
               │  [ferro-mcp, LLM call] │  [no LLM, no network]
               └────────────┬────────────┘
                            │ ServiceDef
               ┌────────────▼─────────────────────────┐
               │  derive_intents(&service)            │
               │  [ferro-projections, deterministic]  │
               └────────────┬─────────────────────────┘
                            │ &[IntentScore]
               ┌────────────▼─────────────────────────┐
               │  Spec::from_service_def(             │
               │    &service, &intents, &ctx)         │
               │  [ferro-json-ui, deterministic]      │
               │  FieldMeaning → component dispatch   │
               │  Intent → layout template            │
               └────────────┬─────────────────────────┘
                            │ Spec (validated internally)
               ┌────────────▼─────────────────────────┐
               │  catalog.json_schema() write gate    │
               │  (already in place — preserve)       │
               └────────────┬─────────────────────────┘
                            │ JSON string
               ┌────────────▼─────────────────────────┐
               │  fs::write("src/views/{name}.json")  │
               └──────────────────────────────────────┘
```

### Component Responsibilities

| File | Change Type | What Changes |
|------|-------------|--------------|
| `ferro-cli/src/main.rs` | Modify | Add `from_service_json: Option<String>` arg to `MakeJsonView`; pass to `run()` |
| `ferro-cli/src/commands/make_json_view.rs` | Modify (major) | Delete `generate_with_ai`, `build_json_view_pass1`, `build_json_view_pass2`; add `generate_via_service_def()` helper; add `load_service_def_from_json()` |
| `ferro-ai/Cargo.toml` | Add dev-dep | `ferro-json-ui = { path = "../ferro-json-ui", features = ["projections"] }` |
| `ferro-ai/tests/projection_roundtrip.rs` | New file | The roundtrip proof test |

### Recommended Project Structure (additions only)

```
ferro-ai/
└── tests/
    ├── pgvector_integration.rs   # existing
    ├── projection_schema.rs      # existing
    └── projection_roundtrip.rs   # NEW — Phase 173 capstone
```

### Pattern: New `generate_via_service_def` Helper

```rust
// In make_json_view.rs — replaces generate_with_ai
#[cfg(feature = "projections")]
fn generate_via_service_def(
    client: &dyn ferro_ai::LlmClient,
    service: &ferro_projections::ServiceDef,
    file_name: &str,
    title: &str,
    layout_name: &str,
) -> String {
    use ferro_projections::derive_intents;
    use ferro_json_ui::{Spec, VisualContext};

    let intents = derive_intents(service);
    let ctx = VisualContext::default();

    match Spec::from_service_def(service, &intents, &ctx) {
        Err(e) => {
            eprintln!("{} Projection render failed: {e}", style("Warning:").yellow().bold());
            eprintln!("{}", style("Falling back to static template.").dim());
            templates::json_view_template(file_name, title, layout_name)
        }
        Ok(spec) => {
            // Catalog validation is already embedded in from_service_def (D-06)
            // but we also validate the serialized JSON form explicitly (D-02)
            match serde_json::to_string_pretty(&spec) {
                Err(e) => {
                    eprintln!("{} Spec serialization failed: {e}", style("Warning:").yellow().bold());
                    templates::json_view_template(file_name, title, layout_name)
                }
                Ok(json_str) => {
                    // Re-parse to validate the JSON form (consistent with original D-03)
                    match ferro_json_ui::Spec::from_json(&json_str) {
                        Err(e) => {
                            eprintln!("{} Spec parse failed: {e}", style("Warning:").yellow().bold());
                            templates::json_view_template(file_name, title, layout_name)
                        }
                        Ok(_) => json_str,
                    }
                }
            }
        }
    }
}
```

Note: `client` parameter is not used in the deterministic path — the function signature accepts it for interface symmetry with the old `generate_with_ai`. The planner may omit it if the NL path goes through `scaffold_core` in the calling `run()` fn directly.

### Pattern: Roundtrip Test Structure

```rust
// ferro-ai/tests/projection_roundtrip.rs
// Mirrors projection_schema.rs: offline, deterministic, no network.

use ferro_projections::{DataType, FieldMeaning, Intent, ServiceDef, derive_intents};
use ferro_json_ui::{Spec, VisualContext, global_catalog};

fn invoice_fixture() -> ServiceDef {
    ServiceDef::new("invoice")
        .display_name("Invoice")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("total", DataType::Float, FieldMeaning::Money)
        .field("recipient", DataType::String, FieldMeaning::EntityName)
}

#[test]
fn servicedef_browse_projection_validates_against_catalog() {
    let service = invoice_fixture();
    let intents = derive_intents(&service);
    assert!(!intents.is_empty(), "invoice fixture must derive at least one intent");

    let browse_idx = intents.iter()
        .position(|i| matches!(i.intent, Intent::Browse))
        .unwrap_or(0);
    let ctx = VisualContext {
        intent_index: browse_idx,
        ..VisualContext::default()
    };

    // SC3+SC5: deterministic render via ServiceDef-aware path
    let spec = Spec::from_service_def(&service, &intents, &ctx)
        .expect("invoice fixture must project successfully");

    // SC2: catalog validation (write gate)
    assert!(
        global_catalog().validate(&spec).is_ok(),
        "projected spec must pass catalog validation"
    );

    // SC5 path proof: Money field → ColumnFormat::Currency in DataTable
    // This cannot be produced by a generic schema-normalization fallback.
    assert_eq!(spec.schema, "ferro-json-ui/v2");
    let root = spec.elements.get(&spec.root).expect("root element must exist");
    assert_eq!(root.type_name, "DataTable", "Browse intent must produce DataTable root");

    let cols = root.props.get("columns")
        .and_then(|c| c.as_array())
        .expect("DataTable must have columns prop");
    let has_currency = cols.iter()
        .any(|c| c.get("format").and_then(|f| f.as_str()) == Some("currency"));
    assert!(
        has_currency,
        "Money field must produce a currency-formatted column — \
         proves the ServiceDef-aware dispatch path (FieldMeaning::Money → ColumnFormat::Currency)"
    );
}
```

### Anti-Patterns to Avoid

- **Keeping `generate_with_ai` in parallel:** D-03 is explicit — delete the direct NL→spec two-pass. No dual paths.
- **Calling `global_catalog()` in a test that also runs in the same binary as `BadPlugin_117` tests:** The roundtrip test is in `ferro-ai/tests/`, a separate binary from `ferro-json-ui` tests. `global_catalog()` is safe here. Do NOT add it to the `ferro-json-ui` test suite without using `from_service_def_with_catalog`.
- **Adding a LLM call to satisfy SC1:** Per D-05, `component_schema()` has no role in the deterministic path. Do not invent an LLM pass.
- **Parsing Rust source files to reconstruct a `ServiceDef`:** Regex-based source parsing (as in `inspect_projection`) produces a text summary, not a live `ServiceDef`. The `--from-service` path must use JSON serialization.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| NL→ServiceDef production | Custom LLM prompt loop | `ferro_mcp::tools::ai_scaffold::scaffold_core` | Phase 171 already implements introspection + prompt + validation |
| ServiceDef→Spec rendering | Custom component-selection logic | `ferro_json_ui::Spec::from_service_def` | Phase 117.1 ships the complete slot-based pipeline |
| Spec validation | Custom JSON Schema validator | `global_catalog().validate(&spec)` (embedded in `from_service_def` + explicit write gate) | Already wired |
| Mock LLM client for tests | Custom mock struct | The in-test `ServiceDef` fixture (no LLM needed for the roundtrip test) | Test does not exercise the NL→ServiceDef path; use a fixture instead |

---

## Common Pitfalls

### Pitfall 1: OnceLock Pollution (does NOT apply to roundtrip test location)

**What goes wrong:** Tests that call `global_catalog()` fail if a sibling test in the same binary registers `BadPlugin_117` first.

**Why it matters here:** The roundtrip test is in `ferro-ai/tests/projection_roundtrip.rs`, compiled into a separate binary from `ferro-json-ui` tests. The `BadPlugin_117` registration only happens in the `ferro-json-ui` test binary. Therefore, the public `Spec::from_service_def` (which calls `global_catalog()`) is safe to use in the roundtrip test.

**How to avoid:** Do not move the roundtrip test into `ferro-json-ui/tests/` or `ferro-json-ui/src/` — keep it in `ferro-ai/tests/`.

### Pitfall 2: Tokio Runtime Nesting

**What goes wrong:** Creating `tokio::runtime::Runtime::new()` inside a tokio context panics with "Cannot start a runtime from within a Tokio runtime."

**Why it matters here:** The `run()` function in `make_json_view.rs` is called from a sync main (no `#[tokio::main]`), so `Runtime::new()` is safe. The roundtrip test uses no async code — all the functions called (`derive_intents`, `Spec::from_service_def`, `global_catalog().validate`) are sync.

**How to avoid:** Keep the runtime creation in `run()`, not inside helper functions that might later be called from async contexts.

### Pitfall 3: `from_service_def` panics in debug on catalog validation failure

**What goes wrong:** If the ServiceDef fixture constructs a spec that fails catalog validation, the test will panic (not return `Err`) in debug builds, producing an obscure panic message rather than a test failure with context.

**How to avoid:** The existing `from_service_def_with_catalog` tests in `builder.rs` show the established pattern — use a well-formed `ServiceDef` with known-valid fields. The `invoice_fixture()` above uses `Identifier`, `Money`, and `EntityName` — all covered by `lookup_meaning`, all producing valid elements. Verify the fixture compiles and passes before asserting the currency format.

### Pitfall 4: `description` arg as `Option<String>` in the new path

**What goes wrong:** The current `run()` signature takes `description: Option<String>`. If description is None but no `--from-service` flag is given, the command should either show an error or fall back to the static template — not call `scaffold_core` with an empty description.

**How to avoid:** The planner should define the explicit behavior: if no description AND no from-service JSON, fall back to static template (same as current behavior when AI is not configured).

---

## Runtime State Inventory

Step 2.5 SKIPPED. Phase 173 is an integration + new-test phase, not a rename/refactor/migration phase. No runtime state is renamed.

---

## Environment Availability

Phase 173 has no external dependencies beyond the Rust toolchain. All required libraries are in the workspace.

| Dependency | Required By | Available | Notes |
|------------|------------|-----------|-------|
| Rust toolchain | cargo build/test | Yes | Workspace standard |
| `ferro-projections` | `make_json_view.rs`, roundtrip test | Yes (workspace) | Already dep of ferro-cli and ferro-json-ui |
| `ferro-json-ui` (projections feature) | roundtrip test (dev-dep) | Yes (workspace) | Needs addition to ferro-ai Cargo.toml dev-deps |
| `ferro-mcp` | `make_json_view.rs` (NL path) | Yes (workspace) | Already dep of ferro-cli |
| Live AI provider (FERRO_AI_API_KEY) | D-07 manual quality gate only | Not required for CI | Automated test uses a fixture |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`#[test]`) |
| Config file | none (workspace-level `cargo test`) |
| Quick run command | `cargo test -p ferro-ai --test projection_roundtrip` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AICLI-04 (SC1) | `catalog.component_schema()` role documented | Manual / VERIFICATION.md | N/A — documented non-use | Exists after VERIFICATION.md write |
| AICLI-04 (SC2) | Generated spec validates against `catalog.json_schema()` | Integration | `cargo test -p ferro-json-ui` (existing builder tests cover this) | Existing |
| AICLI-04 (SC3) | Component selection driven by FieldMeaning/Intent, not LLM re-prompting | Integration (roundtrip test + manual) | `cargo test -p ferro-ai --test projection_roundtrip` | Wave 0 gap |
| AICLI-04 (SC4) | No v1 `JsonUiView` types in pipeline | Audit (grep) | `cargo grep "JsonUiView" ferro-cli/` — zero hits | N/A (grep) |
| AICLI-06 (SC5) | Roundtrip test passes via ServiceDef-aware path | Unit/integration | `cargo test -p ferro-ai --test projection_roundtrip` | Wave 0 gap |

### Sampling Rate

- **Per task commit:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings`
- **Per wave merge:** `cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-ai/tests/projection_roundtrip.rs` — covers AICLI-06 (SC5), AICLI-04 (SC3)
- [ ] `ferro-ai/Cargo.toml` — add `ferro-json-ui` dev-dep with feature `projections`

---

## Security Domain

Phase 173 introduces no new network surfaces, auth, input validation beyond what Phase 171 established. The prompt-injection sanitization for `description` inputs already exists in `scaffold_core` (`sanitize_description` in `ai_scaffold.rs`). `make:json-view` reuses `scaffold_core` unchanged, so the threat mitigation is inherited.

No new ASVS categories apply beyond what Phase 171 already addressed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `from_service_def_with_catalog` is `pub(crate)` inside `ferro-json-ui`, not accessible from `ferro-ai/tests/` | Q3, Code Examples | If accessible, the test can use the safer catalog-injected variant; if not, the public `from_service_def` is used — which is safe in a separate binary anyway |
| A2 | The "existing project file" (D-04) means a JSON-serialized ServiceDef, not the Rust source file | Q7 | If the planner interprets "existing project file" as the `.rs` source, the loading mechanism would need a different approach (not feasible without compilation) |

---

## Open Questions

1. **`--from-service` flag: JSON file path vs service name lookup**
   - What we know: ServiceDefs are stored as Rust source (`.rs`), not JSON. `ServiceDef` implements `Serialize + Deserialize`. `ferro ai:make --dry-run` prints the JSON form.
   - What's unclear: Should `--from-service` accept a service name (and imply a naming convention like `src/projections/{name}.json`) or a free-form file path?
   - Recommendation: Accept a file path (`--from-service-json <path>`) for maximum flexibility. This is Claude's discretion (D-04).

2. **`generate_via_service_def` — should it accept a `client` parameter?**
   - The deterministic path does not use an LLM client. But the NL path (via `scaffold_core`) does.
   - Recommendation: Keep the `run()` function as the coordinator; `generate_via_service_def` takes only `&ServiceDef` — no `client` parameter.

---

## Sources

### Primary (HIGH confidence)

- `ferro-json-ui/src/projection/builder.rs` — `Spec::from_service_def` signature, contract, test patterns (`from_service_def_with_catalog`, `build_builtins_only`)
- `ferro-json-ui/src/projection/mod.rs` — `VisualContext`, `RenderMode`, `JsonUiRenderer` example
- `ferro-json-ui/src/projection/component_map.rs` — `lookup_meaning`, `ColumnFormat::Currency` for `FieldMeaning::Money`
- `ferro-cli/src/commands/make_json_view.rs` — current `generate_with_ai`, `build_json_view_pass1/2` (to be deleted)
- `ferro-cli/src/commands/ai_make.rs` — `scaffold_core` delegation pattern, `emit_service_def_source`
- `ferro-mcp/src/tools/ai_scaffold.rs` — `scaffold_core` signature and contract
- `ferro-ai/tests/projection_schema.rs` — offline test structural template to mirror
- `ferro-ai/src/complete.rs` — `ConstClient` mock LlmClient pattern (reference only)
- `ferro-cli/src/main.rs` — `MakeJsonView` clap variant structure

### Secondary (MEDIUM confidence)

- `.planning/ROADMAP.md §Phase 173` — 5 success criteria and AICLI-04/AICLI-06 requirements
- `.planning/phases/173-make-json-view-v2-projection-roundtrip-test/173-CONTEXT.md` — locked decisions D-01..D-07

---

## Metadata

**Confidence breakdown:**
- Spec::from_service_def signature: HIGH — read the actual source
- NL→ServiceDef reuse (scaffold_core): HIGH — verified ai_make.rs delegates to scaffold_core
- Roundtrip test pattern: HIGH — projection_schema.rs is the literal template
- SC5 path proof (ColumnFormat::Currency): HIGH — verified component_map.rs:277
- `--from-service` loading mechanism: MEDIUM — ServiceDef JSON sidecar is a reasonable recommendation but requires planner confirmation (A2)

**Research date:** 2026-06-09
**Valid until:** 2026-07-09 (30-day window; all sources are internal codebase — stable)
