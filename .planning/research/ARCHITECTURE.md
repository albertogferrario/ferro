# Architecture Research

**Domain:** v13.0 Compressive Validation — integration of COMP-01..05 with existing ferro architecture
**Researched:** 2026-06-12
**Confidence:** HIGH (based on direct source inspection of all relevant crates)

---

## System Overview

The existing projection/intent pipeline is:

```
  ServiceDef (ferro-projections)
      │ derive_intents()
      ▼
  Vec<IntentScore>
      │
      ▼ Renderer::render(&service, &intents, &ctx)
  ┌───────────────┬──────────────────────┬─────────────────────┐
  │ JsonUiRenderer│    McpRenderer        │  TemplateRenderer   │
  │ (ferro-json-ui│ (ferro-mcp-server)    │  (ferro-projections │
  │  Output=Spec) │  Output=rmcp::Tool)   │   Output=String)    │
  └───────────────┴──────────────────────┴─────────────────────┘
```

**Invariant:** ferro-projections owns the `Renderer` trait, `ServiceDef`, `derive_intents()`,
`BaseContext`. It has zero rendering dependencies (Cargo.toml: only schemars, serde,
serde_json, thiserror). All concrete `Renderer` impls live in output crates.

---

## COMP-01: Gestiscilo Migration

**What it is:** Migrate a real external application (gestiscilo) from ad-hoc rendering to
projection-driven rendering via `ServiceDef` builders.

**Integration point:** No new ferro crate needed. The integration happens inside the gestiscilo
repo, replacing its existing view code with `ServiceDef::new(...).field(...).intent_hint(...)` +
`JsonUiRenderer`. The ferro workspace is a passive consumer.

**Validation artifact produced:** A set of `ServiceDef` builders that cover real production
use cases. The relevant cross-repo verification gate is already established by prior friction
loops (phases 162-164, 176, 181).

**New vs modified components:** No new ferro crates. No modification to ferro-projections
or ferro-json-ui internals — only gestiscilo-side authoring work.

---

## COMP-02: Synthetic App-Class Catalog

### Where it lives

The catalog belongs in **`ferro-projections/tests/`** as a new integration test file —
`tests/catalog.rs` (or `tests/app_classes/mod.rs` if split by intent domain).

**Rationale for this location over alternatives:**

| Option | Verdict | Reason |
|--------|---------|--------|
| New crate `ferro-catalog` | Reject | Adds a published crate for what is test-only data; violates "prefer editing existing files"; increases publish surface for no API gain |
| `ferro-projections/tests/` corpus | Accept | `tests/generate_schemas.rs` already lives here and is the established pattern for integration-level validation against ferro-projections types; no new dependency, no new crate, runs under `cargo test --all-features` automatically |
| Fixtures in `app/src/projections/` | Reject | `app/` is a sample application, not a catalog; mixing synthetic catalog entries into the live app distorts its role as a reference implementation |
| Separate `examples/` directory | Reject | `cargo test` does not run examples by default; catalog must be in tests so CI gate covers it |

### Corpus shape

One `ServiceDef` builder function per app class, one test per intent assertion:

```
ferro-projections/
  tests/
    catalog.rs          # new: one function per canonical app class
                        # one #[test] per (app_class, expected_primary_intent) pair
```

Each catalog entry is a free function returning `ServiceDef` (same pattern as `app/src/projections/*.rs`).
The test asserts `derive_intents(&service)[0].intent == ExpectedIntent` and optionally
`confidence >= threshold`.

### Coverage requirement: all 7 intents

| Canonical class | Expected primary intent | Structural signals that drive it |
|-----------------|------------------------|----------------------------------|
| Product catalog (name, category, price, stock) | Browse | EntityName + Category + collection relationships |
| Article / blog post (title, body, hero image) | Focus | FreeText + ImageUrl + read-heavy |
| Registration form (fields, write-only password) | Collect | high writable ratio + write_only |
| Order fulfillment (guarded state machine) | Process | guarded transitions + branching states |
| Revenue dashboard (read-only Money/Pct/Qty) | Summarize | >70% non-writable + money fields |
| Sales analytics (DateTime + numeric measures) | Analyze | DateTime/numeric co-occurrence |
| Shipment tracking (linear states, no guards) | Track | linear progression + unguarded |

The `app/src/projections/` service defs (`order.rs`, `revenue_dashboard.rs`, etc.) are
real-world examples that can guide catalog design but must not be used as the catalog itself
— they live in a different crate and have app-specific concerns (tenant columns, MCP abilities).

### CI hook

`cargo test --all-features` already runs `ferro-projections/tests/`. No new CI step.
The catalog tests run on every commit that touches ferro-projections (derive.rs, intent.rs,
field.rs, service.rs) because the whole workspace test suite runs together.

**Regression contract:** if any `derive_intents()` change breaks a catalog test, the CI
gate fails. This is the desired behavior — the catalog is the regression baseline for the
analyzers.

---

## COMP-03: Agent-Success-Rate Harness

### Where it lives

**`ferro-mcp/tests/agent_harness.rs`** — an integration test inside the existing `ferro-mcp`
crate.

**Rationale:**

| Option | Verdict | Reason |
|--------|---------|--------|
| New crate `ferro-bench-agent` | Reject | Adds published crate overhead; this is a test, not a library API |
| `ferro-mcp/tests/` | Accept | COMP-03 drives the MCP introspection tools as a client — it calls the same in-process server that `ferro-mcp` tests already exercise; keeping it here avoids a new cross-crate dependency |
| External script | Reject | Not in the Rust test suite; not gated by `cargo test` |

### What it does

The harness instantiates an in-process MCP server (the same path `ferro-mcp::server` uses
under `ferro mcp`), issues tool calls against `list_projections`, `inspect_projection`,
`generate_projection`, and `checkpoint_projection`, then asserts that a given natural-language
description can be round-tripped through the MCP surface to produce a valid `ServiceDef`
without a runtime error.

```
test harness (in-process)
    ↓ tool call: list_projections
  ferro-mcp MCP server
    ↓ tool call: generate_projection("order fulfillment with guarded approvals")
  ferro-mcp → ferro-projections::ServiceDef
    ↓ tool call: checkpoint_projection("order")
  verdict: pass / warn / fail
```

**Agent-success-rate metric:** the fraction of `generate_projection` calls (over a fixed set
of natural-language descriptions) that produce a `checkpoint_projection` verdict of `pass`
or `warn` (not `fail`). Stored as a `#[test]` assertion with a floor threshold (e.g.,
`assert!(rate >= 0.7)`).

**Project-agnostic rule:** the harness must not embed gestiscilo-specific descriptions.
Use generic domain descriptions ("a product catalog with name, price, and category",
"an order with guarded approval workflow") that match the COMP-02 catalog classes.

### Relationship to ferro-mcp (not ferro-mcp-server)

`ferro-mcp` is the introspection library used by `ferro mcp` (the developer MCP subcommand).
`ferro-mcp-server` is the separate output crate that hosts `McpRenderer` — it drives the
application-served MCP endpoint (v12.6). COMP-03 is a client of `ferro-mcp` specifically,
not of `ferro-mcp-server`. The harness exercises the developer introspection surface
(generate_projection, checkpoint_projection, list_projections), not the per-tenant consumer
surface (McpRenderer).

### Dependency

COMP-03 depends on COMP-02 catalog: the catalog provides the ground-truth intent for each
domain description, making the success metric meaningful. Build COMP-02 first.

---

## COMP-04: Time-to-Working-App Benchmark

### Where it lives

**`ferro-cli/tests/benchmark_new_project.rs`** — integration test inside `ferro-cli`.

**Rationale:** COMP-04 measures `cargo new` → running service with auth, three entity types,
one background job. This is entirely about `ferro new` scaffolding and the CLI make commands.
`ferro-cli` is the right crate; its existing `tests/` directory (currently has `tempfile`
dev-dep, already used in other tests) supports file-system scaffolding tests.

**What it measures:**

```
1. ferro new <tmpdir>           → scaffolded project compiles
2. ferro make:auth              → auth routes + handler files created
3. ferro make:model Product ... → 3 entity files created
4. ferro make:job EmailJob      → job file created
5. cargo build (in tmpdir)      → succeeds within N seconds (wall clock)
```

The test records step timings and asserts:
- Each `make:*` command exits with code 0.
- The scaffolded project compiles (`cargo build` exit 0).
- Total wall clock <= threshold (e.g., 90 seconds) — or, if too slow for CI, the threshold
  is only asserted when a `FERRO_BENCH` env var is set.

**What COMP-04 does NOT do:** it does not run the server (avoids port conflicts in CI) and
does not measure runtime throughput — only scaffolding and build time.

**Note on CI disk:** `cargo test --all-features` already strains CI disk (see project memory
`project_ferro_disk_full_test_gate.md`). COMP-04 spawns `cargo build` in a temp dir, which
creates a second target directory. Gate it behind `#[cfg_attr(not(feature = "bench"), ignore)]`
or a `FERRO_BENCH=1` env var check so it is skipped in default CI but runnable locally.

---

## COMP-05: Cross-Modality Intent Vocabulary Sketch

### Where it lives

**`ferro-projections/src/render/` (extend existing, add non-pub sketch modules) + docs.**

No new crate. No new `Renderer` implementation that ships as production code at this phase.
COMP-05 is a design sketch — it asks: for one intent (e.g., `Browse`), what would the
output look like as mobile push notification, voice response, and CLI output?

**Integration with the Renderer trait:**

The v11.5 Renderer trait (`ferro-projections/src/render/mod.rs`) already uses associated
types for `Output` and `Context`, making it modality-agnostic by construction:

```rust
pub trait Renderer: Send + Sync {
    type Output;
    type Context: Default;
    fn render(&self, service: &ServiceDef, intents: &[IntentScore], ctx: &Self::Context)
        -> Result<Self::Output, Error>;
}
```

COMP-05 produces three sketch renderers (not shipped, not published) that demonstrate the
trait is sufficient for non-visual modalities:

| Sketch renderer | Output type | Location |
|-----------------|-------------|----------|
| `CliSummaryRenderer` | `String` (table rows) | `ferro-projections/src/render/cli.rs` |
| `VoiceRenderer` | `String` (SSML fragment) | `ferro-projections/src/render/voice.rs` |
| `MobileCardRenderer` | `serde_json::Value` (push payload) | `ferro-projections/src/render/mobile.rs` |

All three stay inside `ferro-projections` under `render/` as `pub(crate)` modules (or
`#[cfg(test)]` if they have no non-test callers) and are documented as research sketches,
not stable API. They inform vocabulary decisions before v14.0 Channel Projection, not after.

**Why not a new output crate:**

A new output crate (e.g., `ferro-voice`) is premature — COMP-05 is exploratory, not a
shipped feature. Adding a published crate now binds v14.0 prematurely. The sketch belongs
in `ferro-projections` as internal research code until v14.0 decides what to publish.

**What COMP-05 informs for v14.0:**

- Whether `BaseContext` needs new fields (e.g., `max_response_tokens`, `device_class`).
- Whether the 7-intent vocabulary is complete for non-visual modalities (does `Track`
  map cleanly to voice? does `Analyze` translate to mobile?).
- Whether `IntentHint` overrides are needed for channel-specific suppression.
- Whether output crates for v14.0 can share context via a `ChannelContext` that extends
  `BaseContext` without modifying ferro-projections.

The sketches produce a written analysis (in `docs/` or as module docs) documenting those
answers before v14.0 starts.

---

## Component Map: New vs Modified

| Component | Status | Crate | Notes |
|-----------|--------|-------|-------|
| `ferro-projections/tests/catalog.rs` | NEW file | ferro-projections | COMP-02 synthetic catalog corpus |
| `ferro-mcp/tests/agent_harness.rs` | NEW file | ferro-mcp | COMP-03 agent success rate harness |
| `ferro-cli/tests/benchmark_new_project.rs` | NEW file | ferro-cli | COMP-04 scaffolding benchmark |
| `ferro-projections/src/render/cli.rs` | NEW file | ferro-projections | COMP-05 CliSummaryRenderer sketch (pub(crate)) |
| `ferro-projections/src/render/voice.rs` | NEW file | ferro-projections | COMP-05 VoiceRenderer sketch (pub(crate)) |
| `ferro-projections/src/render/mobile.rs` | NEW file | ferro-projections | COMP-05 MobileCardRenderer sketch (pub(crate)) |
| `ferro-projections/src/render/mod.rs` | MODIFY | ferro-projections | Add mod declarations for sketch modules |
| gestiscilo projections (external repo) | NEW files | gestiscilo | COMP-01 migration, not in ferro workspace |
| No new published crates | — | — | COMP-01..05 are validation artifacts, not new public APIs |

---

## Build Order

The dependency graph for the COMP phases:

```
COMP-02 (catalog in ferro-projections/tests/)
    │
    ├── feeds ground-truth → COMP-03 (agent harness domain descriptions)
    │
    └── informs vocabulary → COMP-05 (which intents need sketch coverage)

COMP-04 (ferro-cli benchmark) — independent, no catalog dependency

COMP-01 (gestiscilo migration) — independent, cross-repo
    └── surfaces real-world vocab gaps → COMP-05 (informs BaseContext extensions)

COMP-05 (cross-modality sketch in ferro-projections/src/render/)
    └── required-before → v14.0 Channel Projection scope finalization
```

**Recommended phase order:**

1. **COMP-02** first. It is the regression baseline and the ground-truth for COMP-03.
   Pure test code, no production risk, fast to write, immediately improves CI coverage of
   `derive_intents()`.

2. **COMP-04** in parallel with COMP-02. Independent of catalog. Yields the time-to-working-app
   metric early.

3. **COMP-03** after COMP-02 is merged. Depends on catalog for domain descriptions and
   ground-truth intent per class.

4. **COMP-01** can start after COMP-02 establishes the intent vocabulary so gestiscilo
   migrations have a verified baseline to compare against. Large cross-repo effort; likely
   sliced across multiple phases.

5. **COMP-05** after COMP-01 surfaces real-world vocab gaps and COMP-02 confirms the
   7-intent system handles all canonical classes correctly. Non-blocking for other phases
   but required before v14.0 scope work.

---

## Architectural Invariants to Preserve

| Invariant | How enforced in COMP-01..05 |
|-----------|----------------------------|
| ferro-projections is renderer-free | `catalog.rs` only calls `derive_intents()` and asserts on `IntentScore` — no rendering calls; COMP-05 sketch modules are `pub(crate)`, not in Cargo.toml `[dependencies]` of any other crate |
| No hardcoded app identity in ferro-* crates | Catalog entries use generic domain names ("product", "order", "article"); harness uses generic descriptions; no gestiscilo strings anywhere in ferro workspace |
| COMP-05 sketches do not create premature public API | Sketch renderers are `pub(crate)` or `#[cfg(test)]` until v14.0 decides on scope |
| CI disk constraints respected | COMP-04 benchmark gated behind env var / feature flag to avoid second target dir in default CI |
| Renderer implementations live in output crates (production) | COMP-05 sketches in ferro-projections/src/render/ are acceptable as internal research code; any production Channel renderer goes into a new output crate at v14.0 |

---

## Data Flow: How COMP-02 Regression Tests Hook into Every Change

```
Developer edits ferro-projections/src/derive.rs
    │
    ▼
cargo test --all-features
    │
    ├── ferro-projections unit tests (derive.rs already has ~1050 lines of unit tests)
    │
    └── ferro-projections/tests/catalog.rs
            ├── test_product_catalog_derives_browse()
            ├── test_article_derives_focus()
            ├── test_registration_form_derives_collect()
            ├── test_order_workflow_derives_process()
            ├── test_revenue_dashboard_derives_summarize()
            ├── test_sales_analytics_derives_analyze()
            └── test_shipment_tracking_derives_track()
```

Each test calls `derive_intents(&service)` and asserts on position [0] intent and
confidence threshold. A regression in any analyzer immediately fails the specific test
with a clear subject name.

---

## Data Flow: COMP-03 In-Process MCP Client

```
ferro-mcp/tests/agent_harness.rs
    │
    ├── in-process: ferro-mcp server bootstrap (same as `ferro mcp` subcommand path)
    │
    ├── tool call: list_projections(project_root=test_fixtures_dir)
    │       → Vec<ProjectionInfo>
    │
    ├── for each domain_description in COMP-02-derived test cases:
    │   ├── tool call: generate_projection(description)
    │   │       → ServiceDef written to test fixtures dir
    │   │
    │   └── tool call: checkpoint_projection(name)
    │           → Verdict { status: pass|warn|fail, seams, next_steps }
    │
    └── assert: passing_count / total_count >= 0.7
```

---

## Integration Points Summary

| COMP | Integrates With | Communication | New Cargo Dependency |
|------|-----------------|---------------|---------------------|
| COMP-01 | ferro-json-ui (JsonUiRenderer), ferro-projections (ServiceDef) | `Renderer::render()` | None in ferro workspace |
| COMP-02 | ferro-projections (`derive_intents`, `ServiceDef`, `Intent`) | Direct function call in tests | None |
| COMP-03 | ferro-mcp (in-process server + tool execute fns) | In-process MCP tool dispatch | None new; ferro-mcp already uses ferro-projections |
| COMP-04 | ferro-cli (`ferro new`, `make:*` commands) | `std::process::Command` subprocess | `tempfile` already in dev-deps |
| COMP-05 | ferro-projections (Renderer trait, BaseContext) | Implements `Renderer` trait directly | None |

---

## Sources

- Direct source inspection: `ferro-projections/src/render/mod.rs`, `ferro-projections/Cargo.toml`, `ferro-projections/tests/generate_schemas.rs`
- Direct source inspection: `ferro-mcp-server/src/renderer.rs`, `ferro-mcp-server/Cargo.toml`
- Direct source inspection: `ferro-cli/Cargo.toml`, `ferro-cli/src/commands/` (50+ command files)
- Direct source inspection: `app/src/projections/` (8 existing projection examples showing the ServiceDef builder pattern)
- Direct source inspection: `ferro-mcp/src/tools/checkpoint_projection.rs` (established patterns for fixture-based tests, SeamStatus types)
- `.planning/PROJECT.md` v13.0 milestone definition, COMP-01..05 requirements
- `./CLAUDE.md` rendering architecture invariants, project-agnostic crate rule, workspace structure table
- Project memory: `project_ferro_disk_full_test_gate.md` — CI disk constraints informing COMP-04 gating

---
*Architecture research for: v13.0 Compressive Validation (COMP-01..05)*
*Researched: 2026-06-12*
