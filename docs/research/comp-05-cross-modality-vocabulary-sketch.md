# COMP-05: Cross-Modality Vocabulary Sketch

**Phase:** 208 — v13.0 Compressive Validation  
**Status:** Research sketch — not stable API  
**Renderers:** `CliSummaryRenderer`, `VoiceRenderer`, `MobileCardRenderer`  
**Anchor fixture:** `approval_workflow` (Process intent)

This document is the COMP-05 deliverable and a v14.0 Channel Projection planning input. It
analyzes whether the seven-intent vocabulary (Browse, Focus, Collect, Process, Summarize,
Analyze, Track) survives non-visual rendering. The three `pub(crate)` sketch renderers
(`cli.rs`, `voice.rs`, `mobile.rs`) are the forcing function — they are research artifacts,
not shipped API.

Sections: (a) anchor fixture with actual sketch output; (b) 7-intent × 3-modality coverage
matrix; (c) vocabulary tensions; (d) v14.0 implications; (e) discovered weaknesses.

---

## Anchor Fixture

The `approval_workflow` `ServiceDef` (defined inline in each sketch test module) is the
forcing-function fixture for all three renderers:

- **Fields:** `id` (Identifier), `title` (EntityName), `status` (Status), `amount` (Money)
- **State machine:** `approval_lifecycle` — initial `draft`; states `draft`, `submitted`,
  `approved` (final), `rejected` (final), `cancelled` (final)
- **Transitions:** guarded (`has_required_fields`, `is_approver`, `is_cancellable`),
  branching (submit/approve/reject/cancel paths)
- **Actions:** `submit`, `approve`, `reject`, `cancel` with preconditions and transition triggers

`derive_intents()` resolves this fixture to **`Process`** as the primary intent. Evidence:
guarded transitions, branching state machine, workflow actions with preconditions, `Status` +
`Money` fields.

### CLI Summary output (`CliSummaryRenderer`)

```
approval_workflow [process]
Fields:
  - Title (EntityName)
  - Status (Status)
  - Amount (Money)
States (initial: draft):
  - draft (draft)
  - submitted (submitted)
  - approved (approved) [final]
  - rejected (rejected) [final]
  - cancelled (cancelled) [final]
Actions:
  - submit (submit)
  - approve (approve)
  - reject (reject)
  - cancel (cancel)
```

Guard conditions are absent from the action listing. A reader cannot determine from this output
that `approve` requires the `is_approver` guard.

### Voice output (`VoiceRenderer`, `ctx.current_state = None`)

```
The approval_workflow starts in the draft state. You can submit, approve, reject or cancel.
```

Four action verbs are narrated unconditionally. Guard conditions are invisible. A non-approver
user would be told they can "approve" when they cannot.

### Mobile card output (`MobileCardRenderer`)

```json
{
  "intent": "process",
  "service": "approval_workflow",
  "cards": [
    { "type": "header", "title": "approval_workflow", "intent": "process" },
    { "type": "fields", "items": [
        { "label": "Title", "name": "title", "meaning": "EntityName" },
        { "label": "Status", "name": "status", "meaning": "Status" },
        { "label": "Amount", "name": "amount", "meaning": "Money" }
      ]
    },
    { "type": "status", "initial_state": "draft",
      "states": [
        { "name": "draft", "label": "draft", "is_final": false },
        { "name": "submitted", "label": "submitted", "is_final": false },
        { "name": "approved", "label": "approved", "is_final": true },
        { "name": "rejected", "label": "rejected", "is_final": true },
        { "name": "cancelled", "label": "cancelled", "is_final": true }
      ]
    },
    { "type": "actions", "items": [
        { "name": "submit", "label": "submit" },
        { "name": "approve", "label": "approve" },
        { "name": "reject", "label": "reject" },
        { "name": "cancel", "label": "cancel" }
      ]
    }
  ]
}
```

All four actions are emitted without guard context. No chart card type exists in the spec.

---

## Intent × Modality Coverage Matrix

All seven intents are analyzed below. Process is grounded in the actual sketch output above;
the remaining six are analyzed structurally against the same three modalities.

| Intent    | CLI Summary | Voice | Mobile Card |
|-----------|-------------|-------|-------------|
| Browse    | Clean — lists entity names and related fields; no state machine present | Natural — voice lists are well-established; "Here are the items" pattern works | Clean — card per item with label/name is the natural card-list shape |
| Focus     | Lossy — `ImageUrl`/`Url` fields appear as labeled strings with no navigational value | Broken — reading a raw URL aloud is not useful; no alt-text in `FieldDef` to substitute | Incomplete — `ImageUrl`/`Url` fields need a link-card or image-card type not in the current spec |
| Collect   | Functional — fields map to a "required: yes/no" prompt list; usable for CLI form scaffolding | Natural — form fields map to dialog turn sequence; step count is unconstrained but manageable | Clean — one-field-per-card stepper is a well-known mobile pattern |
| Process   | Clean — state names, action list render as text; guard conditions not surfaced (see weaknesses) | Functional — state narration + action verb list works as prose; guard context lost | Functional — header/fields/status/actions card structure is complete; guard conditions not surfaced |
| Summarize | Clean — read-only Money/Percentage/Quantity fields render as a stats block | Natural — verbal summary ("Total: €12,400") is idiomatic voice output | Clean — stats cards with large numeric display are a native mobile pattern |
| Analyze   | Awkward — DateTime + numeric fields render as a table of values; no chart equivalent | Broken — no natural spoken form for time-series trends; must narrate raw values or skip | Incomplete — chart cards require a chart component not in the current card spec |
| Track     | Clean — linear state progression renders as a progress list | Best fit — "The order is currently shipped" is the most natural voice pattern in the vocabulary | Clean — timeline/progress cards are a well-known mobile pattern |

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
