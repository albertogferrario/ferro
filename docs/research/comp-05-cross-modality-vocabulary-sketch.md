# COMP-05: Cross-Modality Vocabulary Sketch

**Phase:** 208 — v13.0 Compressive Validation  
**Status:** Research sketch — not stable API  
**Renderers:** `CliSummaryRenderer`, `VoiceRenderer`, `MobileCardRenderer`  
**Anchor fixture:** `approval_workflow` (Process intent)

This document is the primary deliverable of Phase 208 Plan 01. It provides:
(a) a 7-intent × 3-modality coverage matrix;
(b) named vocabulary tensions;
(c) v14.0 implications;
(d) discovered weaknesses from the sketch implementation.

---

## Anchor Fixture Summary

The `approval_workflow` `ServiceDef` (defined inline in each sketch test module) is the
forcing-function fixture for all three renderers:

- **Fields:** `id` (Identifier), `title` (EntityName), `status` (Status), `amount` (Money)
- **State machine:** `approval_lifecycle` — initial `draft`; states `draft`, `submitted`,
  `approved` (final), `rejected` (final), `cancelled` (final)
- **Transitions:** guarded (`has_required_fields`, `is_approver`, `is_cancellable`),
  branching (submit/approve/reject/cancel paths)
- **Actions:** `submit`, `approve`, `reject`, `cancel` with preconditions and transition triggers

`derive_intents()` resolves this fixture to **`Process`** as the primary intent (confidence
highest among all intents). Evidence: guarded transitions, branching state machine, workflow
actions with preconditions, `Status` + `Money` fields.

---

## Intent × Modality Coverage Matrix

| Intent | CLI Summary | Voice | Mobile Card |
|--------|-------------|-------|-------------|
| **Browse** | Lists entity names and related fields cleanly; state machine absent; no awkwardness | Narrates "Here are the available items" — natural for voice lists | Card per item with label/name; natural card-list shape |
| **Focus** | Detail view fields map to key-value lines; image/URL fields appear as labeled strings | Image URL is meaningless as spoken text ("image_url: https://…"); must skip or paraphrase | `ImageUrl`/`Url` fields need special card type (link card, image card) that the current card spec does not define |
| **Collect** | Fields map to "required: yes/no" prompt list; reasonable for CLI form scaffolding | Form fields map naturally to a dialog turn sequence; number of turns is unconstrained | Form cards (one field per card in a stepper) are a well-known mobile pattern; maps cleanly |
| **Process** | (Anchor) State names, action list, guarded transitions render cleanly as text | (Anchor) State narration + action verb list works as spoken prose; guard context lost | (Anchor) header/fields/status/actions card structure is complete; guard conditions not surfaced |
| **Summarize** | Read-only Money/Percentage/Quantity fields render as a stats block; clean | Verbal summary ("Total revenue: €12,400") is natural; works well | Stats cards with large numeric display are a native mobile pattern; maps cleanly |
| **Analyze** | DateTime + numeric co-occurrence renders as a table of values; no chart representation | Time-series data as voice narration is deeply awkward — no natural spoken form for trends | Chart cards require a chart component not in the current card spec; gap |
| **Track** | Linear state progression renders as a progress list; clean | Status narration ("The order is currently shipped") is the most natural voice pattern | Timeline/progress cards are a well-known mobile pattern; maps cleanly |

**Coverage summary:** Process, Browse, Collect, Summarize, and Track map cleanly to all three
modalities. Focus and Analyze have modality-specific gaps.

---

## Vocabulary Tensions

### Tension 1: Focus intent and non-screen media

The `Focus` intent signals `FreeText`, `ImageUrl`, and `Url` fields — content types whose
primary value is visual (images) or navigational (URLs). In voice output these fields have
no natural narration: reading a raw URL aloud is not useful, and describing an image requires
alt-text that `FieldDef` does not carry. In the CLI summary, the field appears as a labeled
string, but the label carries no information about what the URL points to.

The seven-intent vocabulary has no sub-intent or field-level annotation for "media-heavy Focus"
vs. "text-heavy Focus". A v14.0 renderer needs either a new intent variant or a rendering hint
on `FieldMeaning::ImageUrl` / `FieldMeaning::Url` to know when to skip or substitute.

### Tension 2: Analyze intent and time-series rendering

The `Analyze` intent derives from `datetime_numeric_cooccurrence` — a `DateTime` field and a
numeric field co-occurring. In a visual renderer this signals a chart. In voice or CLI there
is no natural output form for a trend: the vocabulary signals *what kind of analysis* the data
calls for but not *what the spoken/text output shape is*. A voice renderer for `Analyze` would
need to decide between narrating raw values, narrating a pre-computed summary statistic, or
declining to narrate. None of these choices is encoded in the intent.

### Tension 3: Process guard conditions are invisible to non-visual renderers

The `Process` anchor fixture has guarded transitions (`is_approver`, `has_required_fields`).
In `CliSummaryRenderer` and `VoiceRenderer` the guards are not surfaced — the CLI lists actions
without their preconditions, and the voice narration says "you can approve" without saying
"if you are an approver". The `BaseContext` has no field for the caller's role or guard
evaluation results, so renderers cannot conditionally show/hide actions.

This is not a vocabulary gap (the guards are in `ServiceDef`) but a `BaseContext` gap: non-visual
renderers need the guard evaluation context that visual renderers today delegate to the runtime.

---

## v14.0 Implications

| Open Question | Sketch Evidence | Proposed CHAN-* Scope |
|---------------|-----------------|----------------------|
| Does `BaseContext` need `device_class`? | `MobileCardRenderer` hardcodes a card-list shape regardless of device; a tablet and a phone need different card counts. No field in `BaseContext` to communicate this. | CHAN-01: add `device_class: Option<DeviceClass>` to `BaseContext` or introduce `MobileContext: BaseContext` |
| Does `BaseContext` need evaluated guard results? | `CliSummaryRenderer` and `VoiceRenderer` list all actions without filtering by guard; a voice assistant should say "you can approve" only if the caller is an approver. | CHAN-02: add `evaluated_guards: HashMap<String, bool>` or equivalent to `BaseContext` |
| Does `Track` map cleanly to voice? | `Track` (linear state progression, unguarded) maps to "The order is currently shipped" — the most natural voice pattern. No gap found. | No scope needed |
| Does `Analyze` need a dedicated voice strategy? | Time-series data has no natural voice form in the current vocabulary. A voice renderer for `Analyze` must either skip, narrate raw values, or require pre-computed summaries not in `ServiceDef`. | CHAN-03: define `AnalyzeContext` carrying a pre-computed summary string, or add a `summary_hint: Option<String>` to `ServiceDef` for voice use |
| Does `Focus` need `ImageUrl`/`Url` rendering hints? | Voice and CLI cannot usefully render `ImageUrl` fields without alt-text. `FieldDef` has no alt-text or skip-in-voice annotation. | CHAN-04: add `render_hint: Option<RenderHint>` to `FieldDef` (e.g. `RenderHint::SkipInVoice`, `RenderHint::AltText(String)`) |
| Does the card-list spec need a chart card type? | `MobileCardRenderer` has no chart card. `Analyze`-intent services cannot be fully rendered on mobile without one. | CHAN-05: define a `chart` card type in the mobile card spec with `chart_type` and `data_ref` fields |
| Is voice verbosity level needed? | `VoiceRenderer` produces a fixed-length narration regardless of context (quick glance vs. full summary). No verbosity knob in `BaseContext`. | CHAN-01 (same scope as device_class): add `verbosity: Verbosity` enum (`Brief` / `Full`) |

---

## Discovered Weaknesses

### 1. Guard conditions not surfaced in non-visual output

The `Process` anchor fixture's guard conditions (`is_approver`, `has_required_fields`,
`is_cancellable`) are defined on `ServiceDef` but not used by any of the three sketch renderers.
The `CliSummaryRenderer` lists all four actions unconditionally; `VoiceRenderer` narrates all four
action verbs. In a real deployment this would produce misleading output: a non-approver user
would be told they can "approve" when they cannot.

Workaround used in sketch: actions are listed unconditionally. This is intentional for the
research sketch — the gap surfaces the `BaseContext` extension needed in v14.0.

### 2. `IntentScore::Debug` formatting for the intent label

Both `CliSummaryRenderer` and `MobileCardRenderer` use `format!("{:?}", s.intent).to_lowercase()`
to derive the intent label string (e.g. `"process"`). The `{:?}` format depends on the
`#[derive(Debug)]` output of the `Intent` enum, which for `Intent::Process` happens to produce
`"Process"` — lowercased to `"process"`. This is fragile: if the enum variant is renamed or
a custom `Debug` impl is added, the label silently changes. A v14.0 renderer should use a
dedicated `Intent::label() -> &str` method.

### 3. No modality-agnostic fallback for missing intents

When `intents` is empty (e.g. a `ServiceDef` with no fields and no state machine),
`intents.get(ctx.intent_index)` returns `None` and all three renderers fall back to
`"unknown"` as the intent label. This is correct behavior for the sketch but the fallback
is not tested. A production renderer should return `Error::Render` or emit a warning when
the intent slice is empty.

---

*Research sketch — not stable API*  
*Phase 208, v13.0 Compressive Validation. v14.0 Channel Projection depends on this document.*
