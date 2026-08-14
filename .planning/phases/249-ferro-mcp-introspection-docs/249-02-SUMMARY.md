---
phase: 249-ferro-mcp-introspection-docs
plan: "02"
subsystem: docs
tags: [offload, work-distribution, documentation, queues, deployments]
dependency_graph:
  requires: [249-01]
  provides: [offload-docs-canonical-page]
  affects: [docs/src/features/offload.md, docs/src/features/queues.md, docs/src/features/deployments.md, docs/src/SUMMARY.md]
tech_stack:
  added: []
  patterns: [mdBook feature-page structure, neutral-public-voice docs discipline]
key_files:
  created:
    - docs/src/features/offload.md
  modified:
    - docs/src/features/queues.md
    - docs/src/features/deployments.md
    - docs/src/SUMMARY.md
decisions:
  - "offload.md is the single canonical home for authoring surface, result path, scaling model, honest limitations, and 2.0 non-goals"
  - "queues.md §Offloading reduced to a pointer paragraph — no duplicated prose"
  - "deployments.md carries a blockquote callout cross-linking offload.md#scaling-model, not a full section"
metrics:
  duration: "~10 minutes"
  completed: "2026-08-15"
  tasks: 3
  files: 4
---

# Phase 249 Plan 02: Canonical offload.md Documentation Summary

Authored the canonical `docs/src/features/offload.md` page, relocated the existing offload prose out of `queues.md` (replacing it with a pointer), cross-linked the scaling model from `deployments.md`, and registered the page in the mdBook nav — completing OFFLOAD-06 Deliverable B (SC#2, SC#3).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Grep-verify four ASSUMED facts; create canonical offload.md | 5e6516ad | docs/src/features/offload.md (created) |
| 2 | Reduce queues.md §Offloading to pointer; register in SUMMARY.md nav | c94068dd | docs/src/features/queues.md, docs/src/SUMMARY.md |
| 3 | Cross-link from deployments.md; validate mdBook build | 3539f1ce | docs/src/features/deployments.md |

## Fact Verification (Task 1 Step A)

All four ASSUMED facts re-grepped against code before prose was written — no divergence:

- **A1 CONFIRMED:** `enqueue_and_mark_pending`, `read_result`, `read_result_redacted`, `resolve` all present in `framework/src/offload.rs` (L197, L225, L382, L432). Paths used in relocated prose are correct.
- **A2 CONFIRMED:** `pub use migration::Migration as CreateProjectionSnapshotsTable` in `ferro-projection/src/lib.rs:87`. Example import in offload.md is correct.
- **A3 CONFIRMED:** `no_worker: bool` at `app/src/main.rs:82`. The `serve --no-worker` CLI surface used in the deploy recipe is confirmed shipped.
- **A4 CONFIRMED:** Zero `queues.md#` anchor links in `docs/src/` — relocation is safe, no cross-doc links broken.

## Deviations from Plan

None — plan executed exactly as written. The mdBook build (`mdbook build docs`) completed without any ERROR lines, confirming nav registration and cross-link anchors resolve.

## Neutral-Voice Check

`grep -Eic "killer feature|the bet|load-bearing|we accept that|forcing function" docs/src/features/offload.md` → 0 matches. Page is in neutral public-repository voice throughout.

## Key Decisions

1. `offload.md` is the single canonical home for the work-distribution authoring surface, result-path streaming pattern, scaling model, honest limitations, and 2.0 non-goals.
2. `queues.md` §Offloading reduced to a single pointer paragraph; no duplicated prose remains (enqueue_and_mark_pending count in queues.md == 0).
3. `deployments.md` carries a brief blockquote callout (not a full section), keeping that page focused on the artifact/promote subsystem while pointing to the scaling recipe.

## Stubs

None. The offload.md page is complete: authoring surface, result-path prose, and scaling model all wired from confirmed shipped code. No placeholder content.

## Threat Surface Scan

This plan authors Markdown documentation only. No new network endpoints, auth paths, file access patterns, or schema changes. The neutral-voice acceptance grep (0 matches) confirms no internal-strategy leakage into the public artifact.

## Self-Check: PASSED

- `docs/src/features/offload.md` exists: FOUND
- `docs/src/features/queues.md` pointer: FOUND (`grep "(offload.md)"` → match)
- `docs/src/features/queues.md` enqueue_and_mark_pending count: 0 (CONFIRMED removed)
- `docs/src/SUMMARY.md` nav entry between queues and notifications: FOUND (lines 24/25/26)
- `docs/src/features/deployments.md` deep cross-link: FOUND
- `docs/src/features/offload.md` `## Scaling model` anchor: FOUND
- Commits 5e6516ad, c94068dd, 3539f1ce: FOUND in git log
- mdBook build: completed without ERROR lines
