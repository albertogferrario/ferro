---
phase: 160
plan: 06
subsystem: docs/protocol
tags: [docs, json-ui, v1-removal, reframe]
requires: []
provides:
  - "neutral, present-tense Renderer term definition in docs/protocol/src/terminology.md"
  - "neutral JsonUiRenderer pluggability bullet in docs/protocol/src/architecture.md"
  - "factually correct Spec wire-shape paragraph in docs/protocol/src/rendering.md"
affects:
  - docs/protocol (rendered mdbook)
tech_stack:
  added: []
  patterns:
    - "Pattern 5 (verbatim drop-in rewrites) applied to three protocol prose passages"
key_files:
  created: []
  modified:
    - docs/protocol/src/terminology.md
    - docs/protocol/src/architecture.md
    - docs/protocol/src/rendering.md
decisions:
  - "Apply RESEARCH Pattern 5 verbatim replacements per CONTEXT D-07 — no string substitution, full paragraph reframe"
  - "Correct two pre-existing factual errors in rendering.md Output Format (no `version` field, no `body` field) while reframing"
  - "Preserve A2UI/HTML/Native pluggability bullets in architecture.md untouched"
metrics:
  duration: "~5min"
  task_count: 3
  file_count: 3
  completed_date: "2026-05-17"
---

# Phase 160 Plan 06: Reframe Protocol Docs to v2 Spec Shape Summary

Three protocol documentation passages that still contrasted JSON-UI v2 against v1 were rewritten as neutral, present-tense descriptions of the current `Spec` wire shape; the `rendering.md` rewrite also fixed two pre-existing wire-shape inaccuracies (no `version` field, no `body` field).

## Objective

Per CONTEXT.md D-07 and RESEARCH Pattern 5, rewrite three protocol docs (`docs/protocol/src/terminology.md`, `architecture.md`, `rendering.md`) that still contained `ferro-json-ui/v1` framing. Each rewrite is a full paragraph reframe, not a `v1` → `v2` string substitution (per Pattern 5 anti-pattern rule). The wire-shape sentence in `rendering.md` was also factually wrong about v1 — the rewrite both removes the v1 framing AND corrects the wire shape to match the actual `Spec` struct at `ferro-json-ui/src/spec.rs:64-89`.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Rewrite terminology.md Renderer definition | `56360488` | docs/protocol/src/terminology.md |
| 2 | Rewrite architecture.md JsonUiRenderer bullet | `3031939b` | docs/protocol/src/architecture.md |
| 3 | Rewrite rendering.md Output Format section | `ef35eac0` | docs/protocol/src/rendering.md |

## What Changed

### docs/protocol/src/terminology.md

Renderer term definition (lines 94-100) reframed. The old prose described `JsonUiRenderer` as producing `ferro-json-ui/v1 component trees, but ...` (a contrast clause against an earlier shape). The new prose describes the renderer as producing a `Spec` conforming to the `ferro-json-ui/v2` schema, with no historical contrast.

### docs/protocol/src/architecture.md

`JsonUiRenderer` bullet inside the Pluggability section (lines 172-173) reframed. The old prose said "Produces ferro-json-ui/v1 component trees (Table, Card, Form, Badge, Progress, etc.)". The new prose says "Produces a `Spec` conforming to the `ferro-json-ui/v2` schema: a flat ID-keyed element map with components such as Table, Card, Form, Badge, and Progress." The surrounding A2UI/HTML/Native bullets are preserved.

### docs/protocol/src/rendering.md

Output Format section (lines 132-136) reframed and corrected. The old prose claimed the v1 envelope had `schema`, `version`, `title`, and `body` fields. The new prose describes the actual `Spec` shape from `ferro-json-ui/src/spec.rs:64-89`: a `$schema` tag, a `root` element ID, a flat `elements` map keyed by ID, and optional `title`, `layout`, and `data` fields, with children referenced by ID rather than by nesting. Two pre-existing factual errors (`version` and `body` fields that never existed on `Spec`) are removed by this rewrite.

## Verification

All per-task automated gates and the cross-file verification gates from the plan pass:

```
$ grep -n 'ferro-json-ui/v1' docs/protocol/src/terminology.md docs/protocol/src/architecture.md docs/protocol/src/rendering.md
(no matches)

$ grep -l 'ferro-json-ui/v2' docs/protocol/src/terminology.md docs/protocol/src/architecture.md docs/protocol/src/rendering.md
docs/protocol/src/rendering.md
docs/protocol/src/architecture.md
docs/protocol/src/terminology.md
```

Per-file acceptance gates:

- `terminology.md`: `ferro-json-ui/v1` count = 0; `ferro-json-ui/v2` count = 1; paragraph anchor `A component that transforms a Service Definition` count = 1; contrast clause `produces ferro-json-ui/v1 component trees, but` count = 0.
- `architecture.md`: `ferro-json-ui/v1` count = 0; `ferro-json-ui/v2` count = 1; `A2UI` count = 3; `Produces ferro-json-ui/v1 component trees` count = 0; `flat ID-keyed element map` count = 1.
- `rendering.md`: `ferro-json-ui/v1` count = 0; `ferro-json-ui/v2` count = 1; `elements.*map keyed by ID` count = 1; stale envelope phrase `envelope containing.*schema.*version.*title.*body` count = 0.

mdbook harness build:

```
$ cd docs/protocol && mdbook build
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `…/docs/protocol/book`
```

Exit code 0.

## Deviations from Plan

None — plan executed exactly as written. The three Pattern 5 verbatim rewrites applied cleanly with no surrounding text disturbed; no auto-fix rules triggered.

## Decisions Made

- **Verbatim Pattern 5 replacement.** Each of the three rewrites used the exact prose from RESEARCH.md Pattern 5 §(a)/(b)/(c) — no improvisation, no rewording. This honors the "full paragraph reframe" anti-pattern rule (CONTEXT D-07): avoid string substitution that leaves the surrounding "but / however" contrast structure intact.
- **Correct, do not append.** `rendering.md` had two pre-existing factual errors (`version` field, `body` field). The rewrite replaced both, rather than adding a corrective parenthetical. The Output Format section now matches `ferro-json-ui/src/spec.rs:64-89` field-for-field.
- **Preserve neighbouring prose.** Each rewrite is scoped to the smallest passage that contained the v1 framing. The A2UI/HTML/Native bullets in architecture.md and the preamble paragraph in rendering.md were left untouched.

## Key Links

- `docs/protocol/src/terminology.md:94-100` ↔ `ferro-json-ui/src/spec.rs:31` (`SCHEMA_VERSION` wire literal)
- `docs/protocol/src/architecture.md:172-174` ↔ `ferro-json-ui/src/spec.rs:64-89` (Spec struct)
- `docs/protocol/src/rendering.md:132-136` ↔ `ferro-json-ui/src/spec.rs:64-89` (Spec struct — `$schema`, `root`, `elements`, optional `title` / `layout` / `data`)

## Self-Check: PASSED

Files modified (all exist):

- `docs/protocol/src/terminology.md` — FOUND
- `docs/protocol/src/architecture.md` — FOUND
- `docs/protocol/src/rendering.md` — FOUND

Commits (all exist in git log):

- `56360488` — FOUND
- `3031939b` — FOUND
- `ef35eac0` — FOUND
