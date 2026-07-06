# Phase 173: make:json-view v2 + projection-roundtrip test - Context

**Gathered:** 2026-06-09
**Status:** Ready for planning
**Mode:** `--auto` (recommended defaults selected; review decisions below)

<domain>
## Phase Boundary

Make `ferro make:json-view` consume a `ServiceDef` and render it through the
**existing** deterministic `ServiceDef → Spec` renderer, and ship the
**projection-roundtrip test** that proves AI is a first-class projection
consumer end to end: NL description → `ServiceDef` (via `ai:make`, Phase 171) →
rendered JSON-UI v2 spec (via `Spec::from_service_def`) → spec validated against
`catalog.json_schema()`.

This is the **capstone of the v12.1 AI milestone** — its killer feature is the
roundtrip itself: the structural proof that the AI pipeline feeds the
projection/intent core rather than running as a parallel scaffolding system.

**Scope-narrowing discovery (verified, drives this phase):** much of what the
raw ROADMAP criteria describe already exists:
- `Spec::from_service_def(...)` — `ferro-json-ui/src/projection/builder.rs:54` —
  the concrete `Renderer` over a `ServiceDef` + `derive_intents()` IntentScores
  already exists. **Reuse it; do not rebuild a renderer.**
- `make:json-view` already emits **v2 flat specs**, already uses
  `catalog.prompt()`, and already validates against `catalog.json_schema()`
  before writing (`ferro-cli/src/commands/make_json_view.rs`).
- `ai:make` already produces a typed `ServiceDef` and already has
  `emit_service_def_source()` with full `FieldMeaning`/`Intent` handling
  (`ferro-cli/src/commands/ai_make.rs`).

So the phase delta is **integration + the proof test**, not greenfield.

In scope (AICLI-04, AICLI-06):
- Route `make:json-view`'s generation through `ServiceDef` → `Spec::from_service_def`
- The projection-roundtrip test (`ferro-ai/tests/projection_roundtrip.rs`)

Out of scope: new component types, new renderer machinery, non-visual modalities.
</domain>

<decisions>
## Implementation Decisions

### ServiceDef-driven rendering path (the core)
- **D-01:** `make:json-view`'s spec generation routes through the existing
  `Spec::from_service_def(service, &intent_scores)` (`builder.rs:54`) —
  deterministic, `FieldMeaning`/`Intent`-driven component selection via
  `ferro_projections::derive_intents()`. The LLM does **not** re-prompt about
  field types or pick components (ROADMAP SC3). This satisfies "first concrete
  `Renderer` over a `ServiceDef`" by **reusing** the shipped renderer.
- **D-02:** The generated spec is validated against `catalog.json_schema()`
  before write (already implemented — preserve), and contains no v1 `JsonUiView`
  types (SC4 — verification, expected already true).

### NL path unification
- **D-03:** `make:json-view`'s AI path becomes a two-stage **projection** flow:
  NL description → `ServiceDef` (reuse the Phase 171 `ai:make` production logic) →
  `Spec::from_service_def` → catalog validation. This **replaces** the current
  direct NL→spec two-pass (`generate_with_ai` in `make_json_view.rs`). Per the
  feature-branch convention, the superseded direct-to-spec LLM path is deleted,
  not kept in parallel — the ServiceDef is now the single intermediary, which is
  exactly the "AI as projection consumer" thesis the roundtrip proves.
- **D-04:** `make:json-view` also accepts a `ServiceDef` **already present** in
  the project (not only freshly AI-produced) — SC3 says "freshly produced by
  `ai:make` OR loaded from an existing project file." Planner picks the exact
  flag spelling (e.g. `--from-service <name>` vs positional); the behavioral
  contract is: given a `ServiceDef`, render deterministically with no LLM call.

### component_schema vs deterministic selection (flagged tension)
- **D-05 (Claude's discretion — planner resolves):** ROADMAP SC1 says use
  `catalog.component_schema()` for per-component structured output; SC3 says
  component selection is deterministic from `FieldMeaning`/`Intent` via the
  builder. These partially overlap now that `Spec::from_service_def` exists and
  selects components without an LLM. Recommended resolution: the deterministic
  builder owns component **selection**; `catalog.json_schema()` remains the
  write-gate validator; `component_schema()` is used only if a residual LLM
  refinement pass survives (e.g. filling free-text copy for a slot). If the
  deterministic path needs no per-component LLM call, SC1's `component_schema()`
  clause is satisfied vacuously and the planner documents that in VERIFICATION.md
  rather than inventing an LLM pass to use it. Do not add an LLM step solely to
  exercise `component_schema()`.

### Projection-roundtrip test (the killer-feature proof)
- **D-06:** New test `ferro-ai/tests/projection_roundtrip.rs`, mirroring the
  **offline** style of the sibling `ferro-ai/tests/projection_schema.rs`
  (no network, deterministic). Structure:
  1. A fixed `ServiceDef` (constructed in-test as a fixture, OR produced by a
     mock/stub `LlmClient` returning a canned completion) — NOT a live API call,
     so CI is deterministic and key-free.
  2. Assert the `ServiceDef`'s `derive_intents()` outputs, `FieldMeaning` set,
     and `ActionDef` set match the expected shape (the "ServiceDef-aware" half).
  3. Run that `ServiceDef` through `Spec::from_service_def` → produce a v2 spec.
  4. Validate the spec against `catalog.json_schema()` — must pass.
  5. Assert the path is the `ServiceDef`-aware one, **not** a generic
     schema-normalization fallback (ROADMAP SC5).
- **D-07:** The **live** NL→`ServiceDef` quality (a real `ai:make` call against a
  real provider) is a **separate human/manual verification gate** signed off in
  173-VERIFICATION.md — mirroring Phase 171's SC4/SC6 "human-verify live ai:make
  quality" precedent. The automated roundtrip test does not depend on a live key.

### Claude's Discretion
- Exact `make:json-view` flag/arg spelling for "render an existing ServiceDef"
  (D-04) and how a project `ServiceDef` is located/loaded.
- Whether any residual LLM refinement pass is retained (D-05) — default: no.
- The fixture-vs-mock-LlmClient choice for the roundtrip test (D-06) — default:
  whichever keeps the test offline and deterministic with least machinery.
- Exact `intent_scores` construction at the `make:json-view` call site
  (`derive_intents(&service)` is the established pattern in `builder.rs` tests).

### Folded Todos
None — `todo match-phase 173` surfaced no matches.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase contract
- `.planning/ROADMAP.md` § "v12.1 AI" → "Phase 173: make:json-view v2 + projection-roundtrip test" — goal, 5 success criteria, dependencies (Phase 170 SDK migration, Phase 171 `ai:make`), and the AICLI-06 roundtrip requirement.
- `.planning/REQUIREMENTS.md` — AICLI-04 (make:json-view v2), AICLI-06 (projection-roundtrip test).

### The renderer to REUSE (do not rebuild)
- `ferro-json-ui/src/projection/builder.rs:54` — `Spec::from_service_def(service, &intent_scores)` — the existing concrete `ServiceDef → Spec` renderer; slot-based, Intent-driven. Test examples in the same file (`derive_intents(&service)` usage) show the call pattern.
- `ferro-projections` — `derive_intents()` (ServiceDef → ranked IntentScore list), `ServiceDef`, `FieldMeaning`, `Intent`.

### The command to upgrade
- `ferro-cli/src/commands/make_json_view.rs` — current v1-era command: already v2 + `catalog.prompt()` + `json_schema()` validation; the `generate_with_ai` direct-to-spec two-pass is what D-03 replaces.
- `ferro-cli/src/commands/ai_make.rs` — Phase 171 NL→`ServiceDef` production + `emit_service_def_source()`; the logic `make:json-view` reuses for its NL stage.

### Catalog API
- `ferro-json-ui/src/catalog.rs` — `json_schema()` (:647, write-gate validator), `component_schema()` (:775, per-component schema — see D-05 tension), `prompt()` (:817), `validate(&spec)`.

### Test pattern to mirror
- `ferro-ai/tests/projection_schema.rs` — offline, deterministic, no-network ServiceDef-schema test; the structural template for `projection_roundtrip.rs`.

### Sibling-phase context
- `.planning/phases/171-ferro-ai-make-ferro-ai-explain-cli-commands/171-CONTEXT.md` — `ai:make` decisions (no `ScaffoldPlan`, single `ServiceDef` output, live-quality human gate).
- Phase 166 — `ServiceDef`-aware schema normalizer (`ferro_ai::schema::for_structured_output`) the NL→ServiceDef stage relies on.

No external ADRs/specs — the contract is captured by ROADMAP + REQUIREMENTS plus the existing renderer/command/catalog surface.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Spec::from_service_def` (builder.rs:54): the deterministic renderer — the
  single most important reuse. SC3's "first concrete Renderer over a ServiceDef"
  is satisfied by wiring `make:json-view` to this, not by new rendering code.
- `emit_service_def_source` + the whole `ai_make.rs` NL→ServiceDef pipeline:
  reused for `make:json-view`'s NL stage (D-03).
- `catalog.json_schema()` + `Catalog::validate` + `Spec::from_json`: the
  write-gate validation already wired in `make_json_view.rs` (preserve).
- `ferro-ai/tests/projection_schema.rs`: offline-test template for the roundtrip.

### Established Patterns
- `derive_intents(&service)` → `Spec::from_service_def(&service, &intents)` is the
  canonical render call (shown throughout `builder.rs` tests).
- ferro-cli `main()` is sync; commands build a `tokio::runtime::Runtime` locally
  for async LLM calls (see `generate_with_ai`) — reuse for the NL→ServiceDef stage.
- Offline tests construct fixtures / normalized schemas with no network and assert
  with `jsonschema::draft202012` (projection_schema.rs).

### Integration Points
- `make:json-view` command in `ferro-cli/src/commands/make_json_view.rs` (+ its
  registration in `commands/mod.rs` and `main.rs`).
- New test file under `ferro-ai/tests/` (workspace test target; sibling to the
  existing two test files).
</code_context>

<specifics>
## Specific Ideas

- The roundtrip test is the deliverable that *matters* — it is the v12.1 thesis
  made executable. Treat it with disproportionate care: it must demonstrably go
  through the `ServiceDef`-aware path and **fail** if someone later reroutes
  generation through a generic LLM fallback. An assertion that only checks "a
  valid spec was produced" is insufficient — it must pin the path.
- Phase 173 closes v12.1. After it lands, v12.1 is complete and can be marked
  shipped (removing the milestone-pointer ambiguity with v12.4).
</specifics>

<deferred>
## Deferred Ideas

- Additional non-visual `Renderer`s over `ServiceDef` (conversational, voice,
  API) — that is the v14.0 Channel Projection direction, not this phase.
- Live-LLM roundtrip in CI (real provider key in the test matrix) — kept as a
  manual gate (D-07); automating it is a CI-secrets decision, not a v12.1 item.

### Reviewed Todos (not folded)
None — `todo match-phase` surfaced no matches for Phase 173.
</deferred>

---

*Phase: 173-make-json-view-v2-projection-roundtrip-test*
*Context gathered: 2026-06-09 via /gsd-discuss-phase --auto*
