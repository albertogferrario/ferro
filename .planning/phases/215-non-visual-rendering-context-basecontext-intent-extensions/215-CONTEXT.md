# Phase 215: Non-visual rendering context — BaseContext + Intent extensions - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Extend the modality-agnostic rendering *surface* (`ferro-projections`) so a future
non-visual renderer can (a) show an action only when its guard passes and (b) label
an intent reliably — **without touching the seven-intent vocabulary** and without
adding any renderer to `ferro-projections`.

In scope (CHAN-01, CHAN-02):
- `BaseContext` gains `evaluated_guards` (guard→bool) and `verbosity` (`Brief`/`Full`).
- `Intent` gains `label() -> &str`, replacing the fragile `format!("{:?}", intent)`.
- An empty intent slice produces a *typed* render error, not silent `"unknown"`.
- Existing `JsonUiRenderer` and the `ferro-mcp` projection tools compile and pass tests.

Explicitly **out of scope** (deferred to Phase 216 / later milestones):
- The conversational-text `Renderer` itself (Phase 216, CHAN-04).
- `FieldDef::render_hint` for `ImageUrl`/`Url` (Phase 216, CHAN-03).
- `device_class` / `MobileContext` (COMP-05 lists it under CHAN-01, but the
  text-renderer-first milestone scope **drops mobile** — see Deferred Ideas).
- Any reshaping of the seven intents (tracked as CHAN-05, a research outcome).
</domain>

<decisions>
## Implementation Decisions

### BaseContext / VisualContext composition
- **D-01:** Add `evaluated_guards` and `verbosity` to `BaseContext`
  (`ferro-projections/src/render/mod.rs`).
- **D-02:** Refactor `VisualContext` (`ferro-json-ui/src/projection/mod.rs`) to **embed**
  `base: BaseContext` rather than re-declaring `intent_index` / `current_state`. Today the
  two structs duplicate those fields and have drifted into parallel sources of truth;
  this phase collapses them into one. The text renderer (Phase 216) will likewise consume
  `BaseContext` (directly or via its own embedding wrapper). Rationale: single source of
  truth for modality-agnostic context; aligns with the no-duplicate-control-surface
  convention. The internal call sites in `builder.rs` that read `ctx.intent_index` /
  `ctx.current_state` migrate to `ctx.base.intent_index` etc.
  - *Note for planner:* if embedding proves to cascade into a large `builder.rs` churn,
    the fallback is to keep `VisualContext`'s flat fields and add the two new fields to
    both structs — but the embedding refactor is the intended outcome.

### evaluated_guards representation
- **D-03:** `evaluated_guards: HashMap<String, bool>`, keyed by precondition/guard **name**
  (the same strings used in `ActionDef::preconditions` and `GuardDef::name`,
  `ferro-projections/src/action.rs`).
- **D-04:** **Absent key = action renders** (guard treated as not-yet-evaluated /
  unconstrained). Only an explicit `false` filters an action out. This preserves the
  current visual behavior where every action renders regardless of caller role.
  `Default` for `BaseContext` = empty map = render everything.

### verbosity
- **D-05:** `enum Verbosity { Brief, Full }` with `Full` as the default
  (`#[default]` or a hand-written `Default`). `Full` reproduces today's full-render
  behavior, so the default is backward-compatible. The enum lives in `ferro-projections`
  alongside `BaseContext`. Derive `Debug, Clone, Copy, PartialEq, Eq` (and serde +
  `JsonSchema` only if it needs to round-trip — default to no serde unless the planner
  finds a consumer that serializes context).

### Intent labeling
- **D-06:** Add an **infallible** `impl Intent { pub fn label(&self) -> &str }` in
  `ferro-projections/src/intent.rs`. Known variants return a stable lowercase string
  (`"browse"`, `"focus"`, …); `Custom(s)` returns `s.as_str()`. This decouples the label
  from `#[derive(Debug)]`.
- **D-07:** Migrate the three `ferro-mcp` call sites that derive a label from
  `format!("{:?}", intent)` to `.label()`:
  `ferro-mcp/src/tools/render_projection.rs:94` and `:102`, and
  `ferro-mcp/src/tools/generate_projection.rs:89`. Also review
  `ferro-mcp/src/tools/projection_coverage.rs:173` (uses `format!("{:?}", primary.intent)`)
  and migrate if it is a user-facing label. After migration, a grep for
  `format!("{:?}", *intent)` / `{intent:?}` must show **no renderer/tool using it for a
  label** (the `intent_layout.rs` uses at lines 163/167 are error *messages*, not labels —
  leave them or convert at the planner's discretion).

### Empty-intent handling
- **D-08:** Add a typed, modality-agnostic variant to the `ferro-projections`
  `Error` enum (`ferro-projections/src/error.rs`) — e.g. `Error::NoIntents` — returned by
  render entry points when the `intents` slice is empty, instead of falling back to
  `"unknown"`. Living in `ferro-projections` lets the Phase 216 text renderer reuse it.
  Covered by a unit test (success criterion 3).
- **D-09:** The existing `ferro-json-ui` `ProjectionError::IntentIndexOutOfBounds`
  (`ferro-json-ui/src/projection/error.rs:15`) path is **unchanged** — an empty slice with
  `intent_index 0` already errors there. This phase does not need to reroute the visual
  renderer through `Error::NoIntents`; the new variant exists for the non-visual surface.
  (Planner may optionally unify them, but it is not required and must not change visual
  renderer test outcomes.)

### Claude's Discretion
- Exact variant name (`Error::NoIntents` vs `Error::EmptyIntents`), exact `Verbosity`
  derive set, and whether `Verbosity`/`evaluated_guards` get serde — left to the planner,
  constrained by "default preserves current visual behavior" and "no new serialization
  unless a consumer needs it."
- Whether `Intent::label()` returns `&'static str` for known variants via a `match` (it
  must return `&str` to accommodate `Custom`'s borrowed inner string).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source / driving research
- `docs/research/comp-05-cross-modality-vocabulary-sketch.md` — Phase 208 deliverable.
  §"v14.0 Implications" (lines ~162-172) maps each requirement to a concrete
  `BaseContext`/`FieldDef` extension; §"Discovered Weaknesses" (lines ~176-204) is the
  precise rationale for `evaluated_guards` (#1), `Intent::label()` (#2), and the
  empty-intent error (#3). Read these three weakness sub-sections — they are the spec.

### Requirements
- `.planning/REQUIREMENTS.md` — CHAN-01 (line 57), CHAN-02 (line 58). CHAN-03/04 (lines
  59-60) are Phase 216 and define the *next* consumer of this surface.

### Types being extended
- `ferro-projections/src/render/mod.rs` — `BaseContext` (line 22), `Renderer` trait
  (line 35). This is where `evaluated_guards` + `verbosity` land.
- `ferro-projections/src/intent.rs` — `Intent` enum (line 18); `Custom(String)` must stay
  last variant. `label()` impl goes here.
- `ferro-projections/src/error.rs` — `Error` enum; add the empty-intent variant.
- `ferro-projections/src/action.rs` — `ActionDef::preconditions` (Vec<String>, line 34)
  and `GuardDef` (line ~144): the guard-name strings that key `evaluated_guards`.

### Consumers that must keep compiling / migrate
- `ferro-json-ui/src/projection/mod.rs` — `VisualContext` (line 45), `JsonUiRenderer`
  (line 98). VisualContext refactor (D-02) happens here.
- `ferro-json-ui/src/projection/builder.rs` — reads `ctx.intent_index` / `ctx.current_state`
  at many sites; these migrate if D-02 embedding is applied.
- `ferro-json-ui/src/projection/error.rs` — `ProjectionError::IntentIndexOutOfBounds`
  (line 15); leave unchanged (D-09).
- `ferro-mcp/src/tools/render_projection.rs` (lines 94, 102),
  `ferro-mcp/src/tools/generate_projection.rs` (line 89),
  `ferro-mcp/src/tools/projection_coverage.rs` (line 173) — `{:?}`→`.label()` migration.

### Crate boundary rule
- `ferro-projections/CLAUDE.md` — "ferro-projections owns the `Renderer` trait,
  `derive_intents()`, `ServiceDef`… Do not add rendering dependencies to this crate."
  This phase only extends schema/context types and the `Error` enum — no renderer added.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `BaseContext` already exists and is `#[derive(Default)]` — adding fields that have a
  sensible `Default` (empty `HashMap`, `Verbosity::Full`) keeps construction
  backward-compatible automatically.
- `ferro-projections::error::Error` already exists with a `Render` variant — extend it.
- `GuardDef` / `ActionDef::preconditions` already model guard names as strings, so
  `evaluated_guards` keys are a natural fit (no new identifier scheme).

### Established Patterns
- `BaseContext` derives only `Debug, Clone, Default` — **not** `Serialize`/`Deserialize`.
  So `evaluated_guards: HashMap<String,bool>` and a non-serde `Verbosity` are consistent
  with the existing struct (no serde burden).
- `Intent` derives `Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema`
  with `#[serde(rename_all = "snake_case")]` — the serialized strings (`"browse"`, …)
  are the canonical labels `label()` should return for known variants.

### Integration Points
- The duplication between `BaseContext` and `VisualContext` (both carry `intent_index` +
  `current_state`) is the seam this phase tightens. After D-02, `VisualContext` =
  `BaseContext` + visual-only `mode` + `templates`.
- `Spec::from_service_def(service, intents, ctx)` (called by `JsonUiRenderer::render`)
  is the visual render entry point; its signature is unaffected if `VisualContext`
  keeps the same public field-access shape (planner decides flat-vs-embedded ergonomics).
</code_context>

<specifics>
## Specific Ideas

- The COMP-05 weakness write-ups name the exact desired API: `Intent::label() -> &str`
  (weakness #2) and `evaluated_guards: HashMap<String, bool>` on `BaseContext`
  (v14.0 implications table, CHAN-02 row). Follow those names unless there is a concrete
  reason to diverge.
- Verbosity is introduced now (CHAN-01 bundles it with the context extension) even though
  its first *consumer* is the Phase 216 text renderer. The visual renderer ignores it.
</specifics>

<deferred>
## Deferred Ideas

- **`device_class` / `MobileContext`** — COMP-05's v14.0 table lists it under CHAN-01, but
  the v14.0 milestone is explicitly *text-renderer-first* and drops mobile/`device_class`
  to a follow-up channel milestone (per ROADMAP v14.0 overview). Do **not** add a
  `device_class` field in this phase.
- **`FieldDef::render_hint` (AltText/Skip)** — CHAN-03, Phase 216. The `ImageUrl`/`Url`
  Focus-field rendering problem is real but belongs with the renderer that consumes it.
- **The conversational-text `Renderer`** — CHAN-04, Phase 216.
- **Intent vocabulary reshaping** — CHAN-05, a research outcome; this phase freezes the
  seven-intent vocabulary (success criterion: intent.rs vocabulary symbols unchanged).
- **Voice / structured-API / inbound `ferro-ai` classification channels** — later channel
  milestone.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.
</deferred>

---

*Phase: 215-non-visual-rendering-context-basecontext-intent-extensions*
*Context gathered: 2026-06-13*
