---
phase: 208-comp-05-cross-modality-vocabulary-sketch
plan: "01"
subsystem: ferro-projections/render
tags: [sketch, cross-modality, cli, voice, mobile, process-intent, research]
dependency_graph:
  requires: []
  provides: [render/sketch module, CliSummaryRenderer, VoiceRenderer, MobileCardRenderer, docs/research/comp-05 analysis]
  affects: [ferro-projections]
tech_stack:
  added: []
  patterns: [Renderer trait impl, pub(crate) sketch module, inline test fixtures, serde_json card-list]
key_files:
  created:
    - ferro-projections/src/render/sketch/mod.rs
    - ferro-projections/src/render/sketch/cli.rs
    - ferro-projections/src/render/sketch/voice.rs
    - ferro-projections/src/render/sketch/mobile.rs
    - docs/research/comp-05-cross-modality-vocabulary-sketch.md
  modified:
    - ferro-projections/src/render/mod.rs
decisions:
  - "Used pub(crate) mod sketch with #[allow(unused_imports)] on re-exports to satisfy clippy -D warnings since renderers are only used in their own test modules"
  - "Placed // Research sketch comment as a preceding line comment (not trailing) to survive cargo fmt alphabetical reordering of mod declarations"
  - "Fixture inlined verbatim in each test module (no .display_name() on StateDef) to match Phase 207 catalog source exactly"
metrics:
  duration: "~6 minutes"
  completed: "2026-06-12"
  tasks_completed: 3
  files_changed: 6
---

# Phase 208 Plan 01: Cross-Modality Sketch Renderers Summary

Three `pub(crate)` sketch renderers implementing the `Renderer` trait for CLI summary (String), voice narration (String), and mobile card-list (serde_json::Value), all rendering the `Process`-intent `approval_workflow` anchor fixture.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Sketch module + CliSummaryRenderer | a4946f7e | render/mod.rs, sketch/mod.rs, sketch/cli.rs |
| 2 | VoiceRenderer | fa8362ea | sketch/voice.rs |
| 3 | MobileCardRenderer | fc1f054c | sketch/mobile.rs |

## Primary Intent Resolution

`derive_intents(approval_workflow_fixture())` resolves `intents[0]` to **`Intent::Process`** (primary intent, highest confidence). Evidence: guarded transitions, branching 5-state machine with 3 final states, 4 workflow actions with preconditions, `Status` + `Money` fields. This is the expected result per CONTEXT.md D-06 and RESEARCH.md.

## Renderer Observations

### CliSummaryRenderer

Output for the anchor fixture:
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

Awkwardness: guards are absent from the action listing. A user reading this output cannot tell that "approve" requires the `is_approver` guard. The intent vocabulary does not cause this gap — `BaseContext` lacks evaluated-guard results.

### VoiceRenderer

Output for the anchor fixture (`ctx.current_state = None`):
```
The approval_workflow starts in the draft state. You can submit, approve, reject or cancel.
```

Awkwardness: four action verbs narrated unconditionally. Guard conditions are invisible. The voice output would mislead a non-approver. The vocabulary is not the problem — the `BaseContext` lacks guard evaluation context.

### MobileCardRenderer

Output structure:
```json
{
  "intent": "process",
  "service": "approval_workflow",
  "cards": [
    { "type": "header", "title": "approval_workflow", "intent": "process" },
    { "type": "fields", "items": [...title, status, amount...] },
    { "type": "status", "initial_state": "draft", "states": [...5 states...] },
    { "type": "actions", "items": [...4 actions...] }
  ]
}
```

Awkwardness: no guard conditions surfaced on action items; no chart card type (relevant for `Analyze`); no device-class-aware card count.

## Process Mapping Quality

Process is the richest structural intent and maps reasonably to all three modalities:
- CLI: complete (state list + action list is meaningful)
- Voice: functional (state narration + action verbs work as prose)
- Mobile card: complete (4-card structure covers all service aspects)

The mapping is not perfect in any modality — guard conditions are uniformly invisible. But the vocabulary itself (the `Process` intent signal) does not cause the gap; the `BaseContext` / rendering contract does.

## Vocabulary Tensions Found

Three tensions documented in `docs/research/comp-05-cross-modality-vocabulary-sketch.md`:

1. **Focus + non-screen media**: `ImageUrl`/`Url` fields have no useful voice/CLI representation; no field-level rendering hint in `FieldDef`.
2. **Analyze + time-series**: No natural voice form for time-series trends; vocabulary signals the intent but not the output shape.
3. **Process guard visibility**: Guards are in `ServiceDef` but `BaseContext` has no evaluated-guard results for conditional action display.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] voice.rs and mobile.rs created before Task 1 commit**

- **Found during:** Task 1 — `cargo test --lib render::sketch::cli` failed because `sketch/mod.rs` declared `pub(crate) mod mobile;` and `pub(crate) mod voice;` which the compiler required to exist even when testing only the cli module.
- **Fix:** Created all three renderer files before the first commit. Tasks 1, 2, and 3 were committed separately in order after all three files were verified.
- **Impact:** No user-visible change; task commit order preserved.

**2. [Rule 1 - Bug] cargo fmt reordered mod declarations and relocated trailing comment**

- **Found during:** Task 1 post-commit fmt check.
- **Issue:** `cargo fmt` alphabetically reorders `mod` declarations (`sketch` before `template`), and a trailing `// Research sketch` comment on `pub mod template;` was incorrectly relocated.
- **Fix:** Moved the comment to a preceding line comment on its own line before `pub(crate) mod sketch;`. This survives fmt reordering.

**3. [Rule 2 - Missing] #[allow(unused_imports)] on sketch re-exports**

- **Found during:** Task 1 clippy run.
- **Issue:** `pub(crate) use cli::CliSummaryRenderer;` etc. in `sketch/mod.rs` triggered `unused-imports` warnings promoted to errors by `-D warnings`, because the re-exports are not consumed outside test code.
- **Fix:** Added `#[allow(unused_imports)]` per-item in `sketch/mod.rs`. The re-exports are intentionally available for future internal use.

## Known Stubs

None. All three renderers produce non-trivial output for the anchor fixture. No placeholder text or empty data sources.

## Threat Flags

None. All additions are `pub(crate)` pure functions with no I/O, network access, or auth surface.

## Self-Check: PASSED

Files verified present:
- ferro-projections/src/render/sketch/mod.rs
- ferro-projections/src/render/sketch/cli.rs
- ferro-projections/src/render/sketch/voice.rs
- ferro-projections/src/render/sketch/mobile.rs
- docs/research/comp-05-cross-modality-vocabulary-sketch.md

Commits verified:
- a4946f7e (Task 1)
- fa8362ea (Task 2)
- fc1f054c (Task 3)
