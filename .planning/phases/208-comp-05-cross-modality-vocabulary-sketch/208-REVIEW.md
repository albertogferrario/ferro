---
phase: 208-comp-05-cross-modality-vocabulary-sketch
reviewed: 2026-06-12T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - ferro-projections/src/render/mod.rs
  - ferro-projections/src/render/sketch/mod.rs
  - ferro-projections/src/render/sketch/cli.rs
  - ferro-projections/src/render/sketch/voice.rs
  - ferro-projections/src/render/sketch/mobile.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: issues_found
---

# Phase 208: Code Review Report

**Reviewed:** 2026-06-12
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Reviewed the COMP-05 cross-modality sketch renderers (`CliSummaryRenderer`,
`VoiceRenderer`, `MobileCardRenderer`) and the shared `render/mod.rs` helpers.
Severity calibrated per the research-sketch context: these are intentionally
throwaway `pub(crate)` renderers whose purpose is to force the seven-intent
vocabulary through non-visual modalities. Not-production-hardened, no-extensive-
error-handling, and not-public-API are by design and are NOT flagged.

Correctness assessment is clean. All three renderers are pure functions over
`ServiceDef` + `&[IntentScore]` + `BaseContext` with no I/O, no global state, and
no closures, satisfying the schema-only crate rule. No panics on valid input:
every slice index goes through `.get()` with a fallback, `split_last()` is only
called in the `_` arm where `verbs.len() >= 2` (guaranteeing `Some`), and the
`json!`/`writeln!` paths are infallible. No mutation of frozen vocabulary
(`Intent`, `FieldMeaning`) occurs — they are read by value/reference only. The
`field_display_name` and `is_system_field` helpers in `mod.rs` are correct and
test-covered, including the empty-string edge case.

No clippy/`-D warnings` violations were identified by inspection: the deliberate
`let _ =` discards on infallible `writeln!`/`get()` results are the idiomatic way
to silence `must_use`, and the `#[allow(unused_imports)]` on the re-exports is
appropriate for `pub(crate)` items consumed only by tests. The findings below are
all Info-level robustness/clarity notes; none block the phase.

## Info

### IN-01: `Intent` Debug-to-lowercase coupling is fragile for `Custom` variants

**File:** `ferro-projections/src/render/sketch/cli.rs:29`, `ferro-projections/src/render/sketch/mobile.rs:29`
**Issue:** Both renderers derive the intent label with
`format!("{:?}", s.intent).to_lowercase()`. This happens to match the serde
`snake_case` form for all known single-word variants (`Browse`, `Focus`,
`Collect`, `Process`, `Summarize`, `Analyze`, `Track`), but it is coupled to
`Debug` formatting rather than the canonical serialization. A `Custom(String)`
intent renders as `custom("some_value")` (the Debug tuple form lowercased), not
the bare value the serde `#[serde(untagged)]` representation would produce. For a
sketch this only affects label cosmetics, but it means CLI/mobile labels can
diverge from the vocabulary's own serialized names.
**Fix:** If label fidelity matters for the analysis output, serialize the intent
through serde instead of Debug, e.g.
`serde_json::to_value(&s.intent).ok().and_then(|v| v.as_str().map(str::to_owned))`,
falling back to `"unknown"`. Otherwise, a one-line comment noting the labels are
Debug-derived and intentionally approximate would prevent a future reader from
treating them as canonical.

### IN-02: `ctx.intent_index` out-of-range silently degrades to `"unknown"` / no narration

**File:** `ferro-projections/src/render/sketch/cli.rs:27-30`, `ferro-projections/src/render/sketch/mobile.rs:27-30`, `ferro-projections/src/render/sketch/voice.rs:51`
**Issue:** When `ctx.intent_index` exceeds the `intents` slice length, CLI and
mobile fall back to the literal label `"unknown"`, and voice discards the missing
lookup entirely (`let _ = intents.get(...)`). This is safe (no panic) and fine for
a sketch, but `"unknown"` is indistinguishable from a genuine empty-intents case,
which can muddy the cross-modality analysis if an out-of-range index is passed by
mistake.
**Fix:** No code change required for the sketch. If the analysis surfaces this
ambiguity, distinguish "no intents derived" from "index out of range" — e.g. label
`"none"` when `intents.is_empty()` versus `"oob"` when the slice is non-empty but
the index misses.

### IN-03: Voice action list omits the Oxford comma for 3+ verbs

**File:** `ferro-projections/src/render/sketch/voice.rs:45-48`
**Issue:** For three or more actions the output is
`"You can submit, approve or reject."` (no comma before `or`). This is
grammatically defensible but inconsistent with the spoken-prose framing the
sketch is exploring, and worth a deliberate decision since voice naturalness is
exactly what this anchor is meant to probe.
**Fix:** If the analysis wants natural TTS phrasing, change `head.join(", ")` joining
to include the Oxford comma: `format!("You can {}, or {}.", head.join(", "), last)`.
Leave as-is if the omission is intentional — flagging only so the choice is conscious.

---

_Reviewed: 2026-06-12_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
