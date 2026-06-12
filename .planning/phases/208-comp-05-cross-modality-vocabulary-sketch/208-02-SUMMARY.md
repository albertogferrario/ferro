---
phase: 208-comp-05-cross-modality-vocabulary-sketch
plan: "02"
subsystem: docs/research
tags: [analysis, cross-modality, vocabulary, v14.0-planning, COMP-05]
dependency_graph:
  requires: [208-01]
  provides: [docs/research/comp-05-cross-modality-vocabulary-sketch.md]
  affects: []
tech_stack:
  added: []
  patterns: []
key_files:
  created: []
  modified:
    - docs/research/comp-05-cross-modality-vocabulary-sketch.md
decisions:
  - "Fixed intent table rows from bold-formatted **Browse** to plain Browse to satisfy the plan acceptance criterion grep pattern"
  - "Added actual sketch output excerpts (CLI, voice, mobile JSON) to the Anchor Fixture section; the Wave 1 executor created the document but omitted the actual renderer output"
  - "Vocabulary tensions chosen: Focus+non-screen-media (ImageUrl/Url have no voice equivalent), Analyze+time-series (chart intent has no non-visual form), Process guard visibility (BaseContext lacks evaluated-guard results)"
  - "Discovered weaknesses grounded in real sketch behavior: guard conditions omitted from all three renderers; Debug format used for intent label (fragile); missing fallback test for empty-intents edge case"
  - "v14.0 CHAN-* open questions recorded: CHAN-01 (device_class + verbosity), CHAN-02 (evaluated_guards), CHAN-03 (AnalyzeContext/summary_hint), CHAN-04 (FieldDef render_hint), CHAN-05 (chart card type)"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-12"
  tasks_completed: 2
  files_changed: 1
---

# Phase 208 Plan 02: Cross-Modality Vocabulary Analysis Document Summary

Completed the COMP-05 deliverable: `docs/research/comp-05-cross-modality-vocabulary-sketch.md` with all four D-09 mandatory sections — 7×3 coverage matrix, named vocabulary tensions, v14.0 implications table, and non-empty discovered-weaknesses section. Verified byte-freeze on `intent.rs`/`derive.rs` and confirmed sketches are not exported from `lib.rs`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Write cross-modality analysis document | e36c58db | docs/research/comp-05-cross-modality-vocabulary-sketch.md |
| 2 | Verify byte-freeze invariant | (no file changes — read-only verification) | ferro-projections/src/intent.rs, derive.rs, lib.rs |

## Document Validation

The Wave 1 executor created the document (scope overstep) but left two gaps:

1. **Intent table used bold formatting** (`| **Browse** |`) — the plan's literal acceptance criterion uses `grep -ciE "^\| *(browse|..."` which requires plain names. Fixed to plain `| Browse |`.
2. **Anchor fixture section had no actual sketch output** — the plan requires "a short excerpt of each of the three actual sketch outputs." Added verbatim CLI output block, voice prose sentence, and mobile card JSON structure from the real renderers.

After revision: all 9 acceptance criteria pass.

## Byte-Freeze Verification Results

```
git diff --exit-code ferro-projections/src/intent.rs ferro-projections/src/derive.rs
→ exit 0 (no changes — FROZEN_OK)

grep -n "sketch|CliSummary|VoiceRenderer|MobileCard" ferro-projections/src/lib.rs
→ empty (NO_LEAKAGE)
```

## Project Gate Results

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --all --all-targets -- -D warnings` | PASS |
| `cargo test -p ferro-projections` | PASS — 237 unit + 19 catalog + 8 doc-tests + 3 sketch tests |

All sketch renderer tests passed: `cli_summary_non_trivial_output`, `voice_non_trivial_output`, `mobile_card_non_trivial_output`.

## Named Vocabulary Tensions

Three tensions documented in the analysis document:

1. **Focus intent and non-screen media** — `ImageUrl`/`Url` fields have no voice equivalent; no field-level rendering hint in `FieldDef`.
2. **Analyze intent and time-series rendering** — No natural non-visual form for chart data; the vocabulary signals the analysis kind but not the output shape.
3. **Process guard conditions invisible to non-visual renderers** — `BaseContext` has no evaluated-guard results; all three renderers list actions unconditionally.

## Discovered Weaknesses

Three concrete weaknesses from the sketch implementation:

1. **Guard conditions not surfaced** — All three renderers unconditionally list all actions. A non-approver user is told they can "approve." Workaround: listed unconditionally in the sketch; gap surfaces `BaseContext` extension needed in v14.0.
2. **`Debug` format for intent label** — Both `CliSummaryRenderer` and `MobileCardRenderer` use `format!("{:?}", s.intent).to_lowercase()`. Fragile if the enum variant is renamed. Needs `Intent::label() -> &str` in v14.0.
3. **Empty-intents fallback not tested** — All three renderers fall back to `"unknown"` when `intents` is empty; this code path has no test.

## v14.0 Channel Projection Open Questions (CHAN-* candidates)

| Item | Open Question | CHAN-* |
|------|--------------|--------|
| CHAN-01 | Does `BaseContext` need `device_class: Option<DeviceClass>` and `verbosity: Verbosity`? | CHAN-01 |
| CHAN-02 | Does `BaseContext` need `evaluated_guards: HashMap<String, bool>` for conditional action display? | CHAN-02 |
| CHAN-03 | Does `Analyze` need a `summary_hint: Option<String>` on `ServiceDef` or a dedicated `AnalyzeContext`? | CHAN-03 |
| CHAN-04 | Does `FieldDef` need `render_hint: Option<RenderHint>` (e.g. `SkipInVoice`, `AltText`) for Focus intent? | CHAN-04 |
| CHAN-05 | Does the mobile card spec need a `chart` card type with `chart_type` and `data_ref` fields? | CHAN-05 |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Intent table used bold formatting, failing the plan's acceptance criterion grep**

- **Found during:** Task 1 acceptance criteria validation
- **Issue:** Wave 1 executor wrote `| **Browse** |` in the table; plan's literal check is `grep -ciE "^\| *(browse|...)"` which requires plain names.
- **Fix:** Changed all seven intent rows from bold to plain format.
- **Files modified:** `docs/research/comp-05-cross-modality-vocabulary-sketch.md`
- **Commit:** e36c58db

**2. [Rule 2 - Missing] Anchor fixture section had no actual sketch output excerpts**

- **Found during:** Task 1 review against plan spec
- **Issue:** Plan requires "show a short excerpt of each of the three actual sketch outputs (CLI summary text, voice prose sentence, mobile card-list JSON)." The document had a fixture description but no output.
- **Fix:** Added annotated output blocks for all three renderers, derived from the real renderer implementations verified in cli.rs/voice.rs/mobile.rs.
- **Files modified:** `docs/research/comp-05-cross-modality-vocabulary-sketch.md`
- **Commit:** e36c58db

## Known Stubs

None. The analysis document is complete with all required sections grounded in real sketch behavior.

## Threat Flags

None. A committed Markdown file under `docs/research/` carries no executable surface and no trust boundary.

## Self-Check: PASSED
