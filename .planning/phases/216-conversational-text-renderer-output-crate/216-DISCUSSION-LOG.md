# Phase 216: Conversational-text Renderer (output crate) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-13
**Phase:** 216-conversational-text-renderer-output-crate
**Mode:** `--auto` (Claude auto-selected the recommended option for every area)
**Areas discussed:** Output crate identity, Per-intent text strategy, Verbosity semantics, Guard filtering, render_hint design, Focus/Analyze fallback, Snapshot tooling + fixture

---

## A. Output crate identity

| Option | Description | Selected |
|--------|-------------|----------|
| New crate `ferro-text` / `TextRenderer`, reuse `BaseContext` | Mirrors JsonUiRenderer; no context wrapper since BaseContext already has all fields | ✓ |
| New crate + dedicated `TextContext` wrapper | Extra type with no extra fields to carry | |
| Add renderer to ferro-projections | Violates v11.5 crate-boundary rule + success criterion 1 | |

**Auto-selected:** New output crate, `type Context = BaseContext`, facade re-export mirroring `framework/src/lib.rs:265`, publish.yml wave after ferro-projections / before framework.
**Notes:** Crate/type name left to planner; `ferro-text`/`TextRenderer` recommended.

---

## B. Per-intent text rendering strategy

| Option | Description | Selected |
|--------|-------------|----------|
| One strategy per intent, conversational prose | First-class Browse/Collect/Process/Summarize/Track renderers; reads like a channel reply | ✓ |
| Single generic field/state dump for all intents | Mirrors the COMP-05 CLI summary; fails the "conversational" differentiator | |

**Auto-selected:** Per-intent strategies dispatched on `intents[ctx.intent_index].intent` via `Intent::label()`; deterministic plain-text; reuse `field_display_name`/`is_system_field`.
**Notes:** This is the killer-feature surface — text quality is the polish target.

---

## C. Verbosity semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Full = complete render; Brief = headline + actions/primary field | Both pinned by snapshot | ✓ |
| Ignore verbosity for v1 of the renderer | Wastes the Phase 215 `Verbosity` field; fails success criterion 2 | |

**Auto-selected:** Full (default) = fields + state + filtered actions; Brief = entity + intent + guard-passing verbs / primary field.

---

## D. Guard filtering

| Option | Description | Selected |
|--------|-------------|----------|
| Hide action if any precondition is explicitly `false`; absent/true renders | Honors Phase 215 D-04 | ✓ |
| Require all guards present-and-true to render | Breaks "absent = unconstrained" default; over-filters | |

**Auto-selected:** Action-level filtering keyed by `ActionDef::preconditions`; two snapshots (empty guards vs `is_approver:false`).

---

## E. `FieldDef::render_hint` (CHAN-03)

| Option | Description | Selected |
|--------|-------------|----------|
| `Option<RenderHint>` on FieldDef; `RenderHint{AltText(String),Skip}`; default None | Additive, backward-compatible; lives in field.rs | ✓ |
| Voice-specific `SkipInVoice` naming (COMP-05 wording) | Too channel-specific for a text-first generic hint | |
| New intent variant for media-Focus | Reshapes the frozen seven-intent vocabulary (CHAN-05, out of scope) | |

**Auto-selected:** `render_hint: Option<RenderHint>` in `ferro-projections/src/field.rs`, builder method, default None.

---

## F. Focus / Analyze fallback

| Option | Description | Selected |
|--------|-------------|----------|
| Best-effort degraded render + explicit limited-modality note, snapshot-tested | Defined + tested per success criterion 3 | ✓ |
| Return an error for Focus/Analyze | "Defined fallback" reads as graceful, not failure | |
| Fabricate a summary statistic for Analyze | No such data in ServiceDef; would invent output | |

**Auto-selected:** Focus → fields with render_hint rules + note; Analyze → entity + fields + "no full text form" note; both snapshot-tested.

---

## G. Snapshot tooling + anchor fixture

| Option | Description | Selected |
|--------|-------------|----------|
| `insta` (already a workspace dev-dep) + copy COMP-05 fixture | Zero new tooling; matches "snapshot tests" in goal | ✓ |
| Inline `assert_eq!` golden strings | Acceptable fallback; no snapshot files | |

**Auto-selected:** `insta`; copy `approval_workflow` fixture from the `cli.rs` sketch test module into the new crate; add minimal per-intent fixtures.

---

## Claude's Discretion

- Exact crate/type name, `RenderHint` derive set, insta-vs-inline, per-intent wording,
  and whether the COMP-05 sketch renderers stay (recommended) or `cli.rs` is removed.

## Deferred Ideas

- Voice / structured-API renderers, mobile `device_class` + chart-card, inbound `ferro-ai`
  classification, `ServiceDef::summary_hint`, intent-vocabulary reshaping (CHAN-05).
