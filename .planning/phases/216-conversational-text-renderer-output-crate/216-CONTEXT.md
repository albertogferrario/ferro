# Phase 216: Conversational-text Renderer (output crate) - Context

**Gathered:** 2026-06-13
**Status:** Ready for planning
**Mode:** `--auto` (all gray areas auto-resolved with recommended defaults)

<domain>
## Phase Boundary

Ship the **first production non-visual `Renderer`**: a conversational-text renderer that
projects the *same* `ServiceDef` the visual (`JsonUiRenderer`) and MCP (`McpRenderer`)
renderers consume, into text — guard-filtered (Phase 215 `evaluated_guards`) and
verbosity-aware (Phase 215 `Verbosity`), with a defined fallback for the Focus/Analyze
modality gaps. Plus the one remaining schema extension the renderer needs:
`FieldDef::render_hint` (CHAN-03).

In scope (CHAN-03, CHAN-04):
- `FieldDef` gains `render_hint: Option<RenderHint>` (`AltText(String)` / `Skip`) in
  `ferro-projections/src/field.rs`. Schema-only extension — **no renderer added to
  ferro-projections** (crate-boundary rule, v11.5).
- A new **output crate** hosts the text `Renderer` impl, re-exported via the `ferro`
  facade and registered in `publish.yml`.
- Per-intent text rendering for the five cleanly-mapping intents
  (Browse, Collect, Process, Summarize, Track), consuming `BaseContext`
  (`evaluated_guards`, `verbosity`, `intent_index`, `current_state`).
- A defined, tested fallback for Focus and Analyze (the two intents COMP-05 found have
  text/voice gaps).
- Deterministic snapshot tests over the COMP-05 `approval_workflow` anchor fixture.

Explicitly **out of scope** (deferred — see Deferred Ideas):
- Voice renderer, structured-API renderer, mobile `device_class` / chart-card type.
- Inbound intent classification via `ferro-ai` (the conversational *loop* — a
  channel-adapter concern, not the outbound renderer).
- `ServiceDef::summary_hint` for Analyze voice narration (a voice-channel concern).
- Reshaping the seven intents (frozen; CHAN-05 is a future research outcome).

**Killer feature framing:** the projection/intent payoff — one `ServiceDef`, authored
once, now renders to a *non-screen* modality through a per-intent strategy. The obsessive-
polish target of this phase is the **quality of the per-intent text** (decision area B),
not the crate plumbing. Clearing CHAN-03/04 without the text reading naturally is
completion without the killer.
</domain>

<decisions>
## Implementation Decisions

### A. Output crate identity
- **D-01:** Create a **new output crate** for the text renderer. It must NOT live in
  `ferro-projections` (success criterion 1 — `grep` confirms ferro-projections adds no
  renderer). Mirrors `JsonUiRenderer` in `ferro-json-ui` and `McpRenderer` in
  `ferro-mcp-server`.
  - *Recommended name:* crate `ferro-text`, renderer type `TextRenderer`. The crate name
    already conveys "text", so the type need not repeat "conversational". Planner may pick
    a different name (`ferro-conversational`, `ferro-channel-text`) but keep it the
    *text* channel only — voice/structured-API are separate future crates per v11.5.
- **D-02:** `impl Renderer for TextRenderer { type Output = String; type Context = BaseContext; }`.
  **Reuse `BaseContext` directly** — do NOT introduce a `TextContext` wrapper. All fields
  the renderer needs (`intent_index`, `current_state`, `evaluated_guards`, `verbosity`)
  already live on `BaseContext` after Phase 215. (The visual renderer wraps `BaseContext`
  in `VisualContext` only because it adds visual-only `mode`/`templates`; text has no such
  extra fields.)
- **D-03:** Re-export from the `ferro` facade mirroring the JsonUi line at
  `framework/src/lib.rs:265` (`pub use ferro_json_ui::{JsonUiRenderer, ...}`). Add a
  parallel `pub use ferro_text::TextRenderer;` (plus `RenderHint` if it is re-exported
  from the projections facade block). Success criterion 4 = reachable from facade.
- **D-04:** Register the crate in `.github/workflows/publish.yml`. It depends on
  `ferro-projections` (Wave 1b), so it cannot be Wave 1a; it must publish **after
  ferro-projections and before `framework`** (Wave 2, which re-exports it). Planner places
  it in the correct wave (a new sub-wave after 1b, or fold into the framework pre-wave).
  Also add to workspace `members` in root `Cargo.toml`.

### B. Per-intent text rendering strategy (the substance)
- **D-05:** Render via **one strategy function per intent**, dispatched on the primary
  intent (`intents[ctx.intent_index].intent`, labeled with `Intent::label()` from Phase
  215 — never `{:?}`). Five cleanly-mapping intents get first-class strategies:
  - **Browse** — a list/collection summary: entity name + the domain fields that identify
    each item.
  - **Collect** — a "fields to provide" framing: the input fields the form gathers.
  - **Process** — current state (`ctx.current_state`) + the guard-passing actions
    available from here ("Currently *submitted*. You can: approve, reject, cancel").
  - **Summarize** — the headline entity + key status/metric fields in a compact sentence.
  - **Track** — linear state-progression statement ("Currently *shipped*").
- **D-06:** Output is **deterministic plain text**, conversational-leaning (readable
  sentences/labels, not a rigid debug dump), no trailing-whitespace/locale nondeterminism.
  Lean on the existing `field_display_name()` / `is_system_field()` helpers in
  `ferro-projections::render` (drop system fields, title-case labels) so the text crate
  doesn't reinvent them. `Output = String`.
- **D-07:** Empty-intent input returns the Phase 215 typed `Error::NoIntents` (reuse the
  ferro-projections variant), not `"unknown"` — covered by a test.

### C. Verbosity semantics
- **D-08:** Concrete shapes for the two levels (success criterion 2 — render respects
  verbosity):
  - **`Full`** (default) — the complete per-intent render: fields, state machine context,
    and the guard-filtered action list.
  - **`Brief`** — headline only: entity name + intent + the guard-passing action verbs
    (Process/Track) or the primary identifying field (Browse/Collect/Summarize). Omits the
    full field listing and state enumeration.
  Both levels are snapshot-tested over the anchor fixture so the difference is pinned.

### D. Guard filtering semantics
- **D-09:** An action is rendered unless **any** of its `ActionDef::preconditions` maps to
  an explicit `false` in `ctx.evaluated_guards`. Absent key or `true` → render (Phase 215
  D-04: absent = unconstrained). This is action-level filtering keyed by the same
  guard-name strings used in `preconditions` / `GuardDef::name`.
- **D-10:** Snapshot the anchor fixture twice: once with `evaluated_guards` empty (all four
  actions render) and once with `{"is_approver": false}` (approve/reject filtered out,
  submit/cancel remain) — proving success criterion 2's "lists only guard-passing actions".

### E. `FieldDef::render_hint` (CHAN-03)
- **D-11:** Add `pub render_hint: Option<RenderHint>` to `FieldDef`
  (`ferro-projections/src/field.rs:60`). `enum RenderHint { AltText(String), Skip }`.
  Default `None` preserves current behavior (success criterion 3 / CHAN-03: "absent hint
  preserves current behavior"). Add a builder method (`FieldDef::render_hint(...)`)
  consistent with the existing `with_*`/fluent field builders.
  - Derive set mirrors `FieldDef`'s neighbors (Debug, Clone, PartialEq, + serde/JsonSchema
    if `FieldDef` itself round-trips — planner matches the struct's existing derives).
- **D-12:** Renderer behavior for `ImageUrl`/`Url` fields (the Focus content types):
  - `Some(AltText(s))` → render the alt text instead of the raw URL.
  - `Some(Skip)` → omit the field entirely.
  - `None` on an `ImageUrl`/`Url` field → render a useful label form (the field's display
    name + a "(link)"/"(image)" marker), **not** the raw URL string. Success criterion 3:
    "render usefully instead of as raw labels / raw URL".

### F. Focus / Analyze fallback
- **D-13:** Focus and Analyze get a **defined, tested degraded fallback** rather than a
  panic or `"unknown"`:
  - **Focus** — render the fields applying the `render_hint` rules (D-12), plus a one-line
    note that this is a media/navigational view with limited text representation.
  - **Analyze** — render the entity + the field set with a one-line note that time-series /
    trend output has no full text form in this channel (COMP-05 Tension 2). No fabricated
    statistics (none are in `ServiceDef`).
  Both fallbacks are snapshot-tested (success criterion 3: "Focus/Analyze gaps have a
  defined, tested fallback").

### G. Snapshot tooling + anchor fixture
- **D-14:** Use **`insta`** for snapshot tests — it is already a workspace dev-dependency
  (`ferro-projections/Cargo.toml:20`, `features = ["yaml"]`), so no new tooling and it
  satisfies "deterministic snapshot tests" in the goal. Plain inline `assert_eq!` golden
  strings are an acceptable fallback if the planner prefers zero snapshot files.
- **D-15:** Copy the COMP-05 `approval_workflow` anchor fixture (defined in
  `ferro-projections/src/render/sketch/cli.rs` test module) into the new crate's test
  module. Test-fixture duplication is acceptable; do not add a public dependency on the
  sketch (it is `pub(crate)` research-only). The fixture is the canonical Process anchor —
  also construct minimal Browse/Collect/Summarize/Track/Focus/Analyze fixtures for the
  per-intent and fallback snapshots.

### Claude's Discretion
- Exact crate name (`ferro-text` recommended) and renderer type name (`TextRenderer`).
- Exact `RenderHint` derive set and whether it is serde/JsonSchema (match `FieldDef`).
- Whether to use `insta` snapshots vs inline golden strings (D-14).
- The precise conversational wording per intent — constrained only by determinism and
  "reads naturally, not a debug dump."
- Whether the COMP-05 `pub(crate)` sketch renderers (`cli.rs`/`voice.rs`/`mobile.rs`)
  stay as-is (recommended — they document COMP-05) or `cli.rs` is removed as superseded.
  Default: leave them; they are tied to the research doc.

### Folded Todos
None — no pending todos matched this phase.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source / driving research
- `docs/research/comp-05-cross-modality-vocabulary-sketch.md` — Phase 208 deliverable and
  the spec for this phase. §"Vocabulary Tensions" (Tension 1 = Focus `ImageUrl`/`Url`,
  Tension 2 = Analyze time-series, Tension 3 = Process guards), §"v14.0 Implications"
  table (CHAN-* rows), §"Discovered Weaknesses". **Note a numbering remap:** the doc's
  CHAN-04 (`render_hint`) is `.planning/REQUIREMENTS.md` **CHAN-03**; the doc's CHAN-02
  (guards) shipped as **CHAN-01** in Phase 215. REQUIREMENTS.md numbering is authoritative.

### Requirements
- `.planning/REQUIREMENTS.md` — **CHAN-03** (line 59, `FieldDef.render_hint`) and
  **CHAN-04** (line 60, the text renderer) are this phase. CHAN-01/02 (lines 57-58)
  shipped in Phase 215 and are the surface this renderer consumes.

### Prior phase (the surface this renderer consumes)
- `.planning/phases/215-*/215-CONTEXT.md` — D-03/D-04 (`evaluated_guards` semantics:
  absent = render, only explicit `false` filters), D-05 (`Verbosity::Full` default),
  D-06 (`Intent::label()`), D-08 (`Error::NoIntents`). This phase MUST honor those
  semantics, not redefine them.

### Types being consumed / extended
- `ferro-projections/src/render/mod.rs` — `Verbosity` (line ~22), `BaseContext`
  (line ~37, now carries `evaluated_guards` + `verbosity`), `Renderer` trait (line ~55),
  and the reusable `field_display_name()` / `is_system_field()` helpers.
- `ferro-projections/src/field.rs:60` — `FieldDef` (where `render_hint` lands) and
  `FieldMeaning` (`ImageUrl`/`Url`/`FreeText` = the Focus content types).
- `ferro-projections/src/intent.rs` — `Intent` + `Intent::label()` (Phase 215); the seven
  intents the renderer dispatches on. Frozen vocabulary.
- `ferro-projections/src/error.rs` — `Error::NoIntents` (Phase 215) for the empty-intent path.
- `ferro-projections/src/action.rs` — `ActionDef::preconditions` (Vec<String>) and
  `GuardDef` — the guard-name strings that key `evaluated_guards` (D-09).
- `ferro-projections/src/render/sketch/cli.rs` — the `approval_workflow` anchor fixture to
  copy (test module) and the prior `CliSummaryRenderer` shape to learn from (not depend on).

### Crate-boundary rule (MUST hold)
- `ferro-projections/CLAUDE.md` / root `CLAUDE.md` "Rendering architecture" — renderers
  live in their output crate; ferro-projections owns only the trait + `derive_intents()` +
  `ServiceDef`. This phase: schema extension (`render_hint`) goes in ferro-projections; the
  renderer goes in the new crate.

### Plumbing references
- `framework/src/lib.rs:265` — the `JsonUiRenderer` facade re-export line to mirror (D-03).
- `.github/workflows/publish.yml` (Waves 1a/1b/2, lines ~206-301) — where the new crate
  registers (D-04). Root `Cargo.toml` `members` (line 3) — add the crate.

### Project convention
- `MEMORY.md` → "When adding a new crate to the workspace, always add it to publish.yml in
  the correct wave" + "no co-author lines" + run `fmt && clippy --all-targets -Dwarnings &&
  test --all-features` before commit + `cargo doc -Dwarnings` clean (success criterion 4).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`BaseContext`** already carries everything the text renderer reads (`evaluated_guards`,
  `verbosity`, `intent_index`, `current_state`) — so `type Context = BaseContext`, no wrapper.
- **`field_display_name()` / `is_system_field()`** (`ferro-projections::render`) — reuse
  for label casing and dropping infra fields; don't reimplement.
- **`Intent::label()` / `Error::NoIntents` / `Verbosity`** — Phase 215 deliverables built
  specifically for this consumer.
- **`insta`** (yaml) is already a workspace dev-dep — snapshot tooling is in-tree.
- **The COMP-05 `cli.rs` sketch** is a working reference for the Process render shape and
  owns the canonical `approval_workflow` fixture.

### Established Patterns
- Renderer-per-output-crate: `JsonUiRenderer`@ferro-json-ui, `McpRenderer`@ferro-mcp-server.
  The text crate is the third instance of this pattern.
- `Renderer::render(&self, service, intents, ctx)` returns `Result<Output, Error>` — the
  text impl returns `Result<String, Error>`.
- Facade re-export pattern at `framework/src/lib.rs` (lines 81, 257, 265).

### Integration Points
- New crate → workspace `members` (root `Cargo.toml`), `framework` dependency + facade
  re-export, `publish.yml` wave (after ferro-projections, before framework).
- `render_hint` is the only edit inside ferro-projections (`field.rs`) — additive, `Option`,
  default `None`, so all existing `FieldDef` construction and the visual/MCP renderers
  compile unchanged.
</code_context>

<specifics>
## Specific Ideas

- COMP-05 names the desired API directly: `RenderHint::AltText(String)` / a skip variant on
  `FieldDef` (Tension 1 / implications table). Follow those names (`Skip`, not the doc's
  voice-specific `SkipInVoice`, since the milestone is text-first and generic).
- The Process render is the forcing-function case — the anchor fixture resolves to `Process`,
  and the guard-filtering + verbosity behavior is most visible there. Pin both the
  unfiltered and `is_approver: false` outputs as snapshots.
- "Conversational-text" ≠ the COMP-05 CLI *summary* dump. The differentiator this phase must
  deliver is text that reads like a channel reply, not a struct printout.
</specifics>

<deferred>
## Deferred Ideas

- **Voice renderer** — separate output crate; Analyze has no natural spoken form and needs
  `ServiceDef::summary_hint` (COMP-05). Later channel milestone.
- **Structured-API renderer** — separate output crate. Later.
- **Mobile `device_class` / `MobileContext` / chart-card type** — explicitly dropped from
  the text-first v14.0 scope (Phase 215 deferred it too).
- **Inbound intent classification via `ferro-ai`** — the conversational *loop* (channel
  adapter), distinct from this outbound renderer.
- **`ServiceDef::summary_hint`** for Analyze narration — a voice-channel concern; this phase
  uses the "no full text form" fallback (D-13) instead.
- **Intent vocabulary reshaping (CHAN-05)** — frozen here; a future research outcome.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.
</deferred>

---

*Phase: 216-conversational-text-renderer-output-crate*
*Context gathered: 2026-06-13*
