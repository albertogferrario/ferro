# Phase 208: COMP-05 — Cross-Modality Vocabulary Sketch - Context

**Gathered:** 2026-06-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Determine whether the seven-intent vocabulary (Browse, Focus, Collect, Process, Summarize, Analyze, Track) is sufficient for non-visual rendering modalities — **before** v14.0 Channel Projection begins. The phase delivers a research probe, not a shipped feature:

- **Three `pub(crate)` sketch renderers** (`CliSummaryRenderer`, `VoiceRenderer`, `MobileCardRenderer`) in `ferro-projections/src/render/`, each implementing the existing `Renderer` trait with non-trivial output.
- **An analysis document** covering all seven intents across the three non-visual modalities, naming at least one vocabulary tension.

**Hard constraints (from ROADMAP SC + REQUIREMENTS Out of Scope):**
- `intent.rs` and `derive.rs` stay byte-frozen — no change to any of the seven intent-vocabulary symbols. Any revision the sketch surfaces is filed as a named v14.0 proposal, not implemented here.
- No new published crates; everything lives in `ferro-projections` as `pub(crate)` sketch modules.
- No non-visual renderer *implementations* (that is v14.0 Channel Projection) — these are sketches, marked `// Research sketch — not stable API`.

This is COMP-05, the fifth and final requirement of the v13.0 Compressive Validation milestone. v14.0 Channel Projection depends on this analysis.

</domain>

<decisions>
## Implementation Decisions

### Sketch Renderer Output Types
- **D-01:** `CliSummaryRenderer::Output = String` — a plain-text summary block (the kind a CLI command would print to stdout).
- **D-02:** `VoiceRenderer::Output = String` — spoken prose (what a voice assistant would say). **No SSML** in v13.0 — SSML/prosody markup is a v14.0 concern; the sketch produces plain narration so the vocabulary tension (not the markup format) is what surfaces.
- **D-03:** `MobileCardRenderer::Output = serde_json::Value` — a card-list spec (structured, not a string), so the "card shape" the intent maps to is directly inspectable. Mirrors the structured-JSON precedent set by `TemplateRenderer`.

### Context Strategy
- **D-04:** All three renderers reuse `BaseContext` (in `render/mod.rs`) **unchanged**. Do not add fields to `BaseContext` in this phase.
- **D-05:** Every modality-specific context need the sketch reveals (e.g. `device_class`, voice verbosity level, max-card-count) is recorded as a **v14.0 implication** in the analysis document, not added to the codebase. The gaps themselves are the research output — capturing them without acting on them is correct here.

### Anchor Fixture
- **D-06:** The three renderers all render **one shared `Process`-intent `ServiceDef`** — an order/approval workflow carrying a state machine, actions, and money/status fields. Process is the richest structural shape (state machine + actions + typed fields) and is the intent COMP-05 names as the example. Building all three renderers against the same fixture makes the cross-modality comparison concrete.
- **D-07:** The **document** then covers all seven intents across the three modalities analytically (SC#3). Only the anchor fixture needs working renderer output; the other six intents are analyzed in prose, not necessarily rendered.

### Analysis Document
- **D-08:** The analysis is a **standalone Markdown file** at `docs/research/comp-05-cross-modality-vocabulary-sketch.md` (create the `docs/research/` directory if absent), with a short pointer doc-comment on the sketch module linking to it. A 7×3 matrix plus a v14.0-implications section is too long for a module-level doc block, and a standalone file is the artifact v14.0 Channel Projection planning will read directly.
- **D-09:** The document MUST include: (a) the 7-intent × 3-modality coverage matrix; (b) at least one named **vocabulary tension** (an intent boundary that is unclear or insufficient for non-visual rendering); (c) a **"v14.0 implications"** section with specific open questions for Channel Projection scope (e.g. does `BaseContext` need `device_class`, does `Track` map cleanly to voice); (d) a **"discovered weaknesses"** note naming at least one place where the current vocabulary forced a workaround or awkward output in the sketch. Empty (b) or (d) fails the phase close.

### Test Depth
- **D-10:** One smoke test per renderer asserting **non-trivial output** (non-empty, and containing expected domain tokens from the anchor fixture — e.g. the CLI summary mentions the state name, the voice script mentions an action verb, the mobile card spec has a non-empty card array). Mirror the unit-test style already in `render/mod.rs` and `template.rs`. No `insta` snapshots — sketch code is intentionally throwaway-grade.

### Claude's Discretion
- Exact module file layout under `render/` (e.g. a `render/sketch/` submodule with `cli.rs` / `voice.rs` / `mobile.rs` + a `sketch/mod.rs`, vs three flat files) — planner/executor choice, as long as all three are `pub(crate)` and each carries the `// Research sketch — not stable API` marker.
- Exact field/action composition of the anchor `Process` fixture, provided it exercises a state machine, at least one action, and money/status fields.
- Wording and section ordering of the analysis document beyond the mandatory sections in D-09.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope & acceptance
- `.planning/ROADMAP.md` (Phase 208 details, lines ~2751–2766) — the five success criteria; the byte-freeze constraint on `intent.rs`/`derive.rs`; the `pub(crate)` + `// Research sketch` requirement.
- `.planning/REQUIREMENTS.md` §COMP-05 (line 17), Out of Scope table (lines 25–34), Future Requirements (line 23) — "document is a v14.0 planning input only"; "intent vocabulary revision is a v14.0 outcome (CHAN-05)".
- `.planning/PROJECT.md` §"Current Milestone: v13.0" — COMP-05 framing; multimodal as a v2.0+ direction.

### Renderer contract to implement against
- `ferro-projections/src/render/mod.rs` — the `Renderer` trait (associated `Output` + `Context: Default`), `BaseContext`, and the `field_display_name` / `is_system_field` helpers.
- `ferro-projections/src/render/template.rs` — `TemplateRenderer`, the reference implementation pattern (struct + `impl Renderer`, structured-JSON output, doctest, unit tests) the sketches should mirror.
- `ferro-projections/CLAUDE.md` — boundary rule (no rendering deps added to this crate; sketches are the explicit research exception); module conventions.

### Vocabulary being probed (READ-ONLY — must not change)
- `ferro-projections/src/intent.rs` — the seven `Intent` variants + `IntentScore`. Frozen.
- `ferro-projections/src/derive.rs` — `derive_intents()`. Frozen.
- `ferro-projections/src/service.rs` — `ServiceDef` builder (`new`, `field`, `action`, `state_machine`, `belongs_to`/`has_many`, etc.) for constructing the anchor fixture.
- `ferro-projections/src/state.rs` — `StateMachine`, `StateDef`, `Transition` for the Process fixture's state machine.
- `ferro-projections/src/action.rs` — `ActionDef`, `GuardDef`, `InputDef`.
- `ferro-projections/src/field.rs` — `FieldMeaning`, `DataType` (Money, Status, etc.).

### Downstream consumer
- `.planning/ROADMAP.md` v14.0 bullet (line 65) — "v14.0 Channel Projection depends on COMP-05 (intent vocabulary validation)"; the analysis document's "v14.0 implications" section is this milestone's input.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Renderer` trait** (`render/mod.rs:33`) — already modality-agnostic via associated `Output`/`Context` types (shipped v11.5). The sketches implement it directly; no trait change needed.
- **`TemplateRenderer`** (`render/template.rs`) — copy its shape: a unit struct, `type Output` / `type Context = BaseContext`, an `impl Renderer`, a doctest, and a `#[cfg(test)] mod tests`. The two String renderers and the Value renderer all fit this mold.
- **`field_display_name` / `is_system_field`** (`render/mod.rs`) — reuse for human-readable labels and to drop Identifier/CreatedAt/UpdatedAt from sketch output.
- **`ServiceDef` builder + `derive_intents`** — the anchor fixture is built with the same builder used in the Phase 207 catalog (`ferro-projections/tests/catalog.rs`); a Process fixture there may be a useful starting point.

### Established Patterns
- ferro-projections is **schema-only, no runtime engines, no closures** (CLAUDE.md). Sketch renderers must stay pure functions over `ServiceDef` + `IntentScore` → output; no I/O, no global state.
- Renderers normally live in their output crate; the `pub(crate)` sketch-in-ferro-projections placement is an explicit, ROADMAP-sanctioned exception for this research phase only — do NOT re-export them from `lib.rs` (they are not stable API).

### Integration Points
- New modules under `ferro-projections/src/render/` registered in `render/mod.rs` (the existing `pub mod template;` line shows the pattern — sketches use `pub(crate) mod`).
- Analysis document under `docs/research/` (sibling to `.planning/research/`, but committed under repo `docs/` per SC#3 "a file in `docs/`").

</code_context>

<specifics>
## Specific Ideas

- COMP-05's named example is **Process** ("takes one intent (e.g. `Process`) and expresses the same feature as a mobile flow, a voice interaction, and a CLI command") — D-06 follows it.
- The phase is deliberately low-stakes engineering / high-stakes analysis: the renderers exist to *force* the vocabulary question, and the document is the real deliverable. Invest effort in the analysis (D-09), not in renderer polish.

</specifics>

<deferred>
## Deferred Ideas

- **SSML / prosody markup for voice output** — v14.0 Channel Projection (D-02 keeps the sketch plain-prose).
- **`BaseContext` extensions** (`device_class`, voice verbosity, card-count limits) — recorded as v14.0 implications in the document, not implemented (D-05).
- **Any seven-intent vocabulary revision** the sketch motivates — filed as a named v14.0 / CHAN-05 proposal, never implemented in v13.0 (ROADMAP SC#2, REQUIREMENTS line 23).
- **Production non-visual renderers** (real CLI/voice/mobile output crates) — v14.0 Channel Projection direction.

None of these block phase close; all are explicitly out of v13.0 scope.

</deferred>

---

*Phase: 208-comp-05-cross-modality-vocabulary-sketch*
*Context gathered: 2026-06-12*
