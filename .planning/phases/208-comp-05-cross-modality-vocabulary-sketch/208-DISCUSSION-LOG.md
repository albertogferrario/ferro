# Phase 208: COMP-05 — Cross-Modality Vocabulary Sketch - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-12
**Phase:** 208-comp-05-cross-modality-vocabulary-sketch
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** Output types, Context strategy, Anchor fixture, Document location, Test depth

---

## Sketch Renderer Output Types

| Option | Description | Selected |
|--------|-------------|----------|
| All `String` | All three renderers emit plain strings | |
| All `serde_json::Value` | All three emit structured JSON | |
| Mixed: String for linear, Value for cards | CLI + Voice → String; Mobile → Value | ✓ |

**Auto-selected:** Mixed — CLI/Voice `String`, Mobile `serde_json::Value`.
**Notes:** Linear modalities (CLI text, spoken prose) are naturally strings; the mobile card is structural, so a `Value` makes the "card shape" inspectable. Matches `TemplateRenderer` precedent. Voice stays plain-prose (no SSML) — SSML is a v14.0 concern.

---

## Context Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse `BaseContext` unchanged | All three use the existing context; gaps recorded as v14.0 implications | ✓ |
| Add fields to `BaseContext` | Extend with `device_class` etc. now | |
| Define per-modality context structs | New context type per renderer | |

**Auto-selected:** Reuse `BaseContext` unchanged.
**Notes:** v13.0 must stay churn-free; the *missing* context fields are themselves the research finding and belong in the v14.0-implications section.

---

## Anchor Fixture

| Option | Description | Selected |
|--------|-------------|----------|
| Process-intent workflow | One shared order/approval fixture (state machine + actions + money/status) | ✓ |
| Browse-intent collection | A list/collection fixture | |
| One fixture per intent | Seven fixtures rendered by all three | |

**Auto-selected:** Process-intent workflow, shared across all three renderers; document analyzes all 7 intents in prose.
**Notes:** Process is the richest structural shape and the example named in COMP-05. Only the anchor needs working renderer output (SC#1); the other six intents are covered analytically (SC#3).

---

## Analysis Document Location

| Option | Description | Selected |
|--------|-------------|----------|
| Standalone Markdown in `docs/research/` | Dedicated file, pointer doc-comment on module | ✓ |
| Module-level doc block | Analysis lives in the sketch module's `//!` docs | |

**Auto-selected:** `docs/research/comp-05-cross-modality-vocabulary-sketch.md` + short module pointer comment.
**Notes:** 7×3 matrix + v14.0 implications is too long for a doc block; a standalone file is the artifact v14.0 Channel Projection planning reads directly.

---

## Test Depth

| Option | Description | Selected |
|--------|-------------|----------|
| One smoke test per renderer | Non-empty + expected domain tokens; no snapshots | ✓ |
| `insta` snapshots per renderer | Full output snapshots | |
| No tests | Sketches uncovered | |

**Auto-selected:** One smoke test per renderer (mirrors `template.rs` unit-test style).
**Notes:** Proves SC#1 "non-trivial output" without over-investing in intentionally throwaway sketch code.

---

## Claude's Discretion

- Exact module layout under `render/` (submodule vs flat files), provided all three are `pub(crate)` and carry the `// Research sketch — not stable API` marker.
- Exact field/action composition of the Process fixture.
- Document wording/ordering beyond the mandatory sections.

## Deferred Ideas

- SSML/prosody for voice — v14.0.
- `BaseContext` extensions (`device_class`, verbosity, card limits) — recorded as v14.0 implications, not implemented.
- Any seven-intent vocabulary revision — named v14.0 / CHAN-05 proposal only.
- Production non-visual renderers — v14.0 Channel Projection.
