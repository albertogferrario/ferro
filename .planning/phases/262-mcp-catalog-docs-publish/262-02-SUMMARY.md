---
phase: 262-mcp-catalog-docs-publish
plan: "02"
subsystem: docs
tags: [docs, live-fragment, memoize, asset-macro, mdbook, json-ui]
dependency_graph:
  requires: [262-01-PLAN.md]
  provides: [SC-3 docs coverage for LiveFragment / #[memoize] / asset!()]
  affects: [docs/src/json-ui/components.md, docs/src/json-ui/runtime-primitives.md, docs/src/features/ferro-assets.md, docs/src/features/projections.md]
tech_stack:
  added: []
  patterns: [mdBook, extend-existing-pages-first (D-06), neutral-product-docs-voice (D-07)]
key_files:
  created: []
  modified:
    - docs/src/json-ui/components.md
    - docs/src/json-ui/runtime-primitives.md
    - docs/src/features/ferro-assets.md
    - docs/src/features/projections.md
decisions:
  - "D-06: Extended existing pages only — no new pages, SUMMARY.md unchanged"
  - "D-07: Neutral product-documentation voice throughout; no version-vs-version framing"
  - "D-08: No ferro-base.css regen — LiveFragment container emits only data-* attributes"
metrics:
  duration_seconds: 156
  completed_date: "2026-07-26"
  tasks_completed: 3
  files_modified: 4
requirements: [LIVE-04]
---

# Phase 262 Plan 02: v17.0 docs — LiveFragment, asset!(), #[memoize] Summary

**One-liner:** Four existing doc pages extended with LiveFragment WebSocket binding, asset!() content-hashed embed, ferro assets fetch CLI, and #[memoize] request-scoped render-dedup, with mdBook build exiting 0 (SC-3 green).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | LiveFragment docs — components.md + runtime-primitives.md | 8ea71f21 | docs/src/json-ui/components.md, docs/src/json-ui/runtime-primitives.md |
| 2 | asset!() + ferro assets fetch in ferro-assets.md; #[memoize] in projections.md | 53b93606 | docs/src/features/ferro-assets.md, docs/src/features/projections.md |
| 3 | mdBook build gate — docs build exits 0 | (no code change) | mdbook v0.5.2, exit 0 |

## What Was Built

### Task 1 — LiveFragment component and runtime behavior

**`docs/src/json-ui/components.md`:**
- Added `LiveFragment` row to the Component Overview table under a new `Live / Real-time` category.
- Added `## Live / Real-time Components` section with `### LiveFragment` subsection.
- Props table documents `projection` (ferro-projection NAME), `key` (per-key channel selector), and `template` (child JSON-UI spec rendered against snapshot as data scope).
- Usage example with sample `"inventory"` / `"warehouse-a"` identifiers, explicitly framed as illustration.
- Behavioral note: first-paint empty when no snapshot; innerHTML swap on delta; one binding pattern only; no list/collection reconciliation (explicit non-goal); link to runtime-primitives for subscription details.

**`docs/src/json-ui/runtime-primitives.md`:**
- Added `## data-live-fragment / data-channel` section parallel to the existing `## data-lazy-hero`.
- Contract table: `data-live-fragment` (opt-in marker) and `data-channel` (`"projection.{name}.{key}"`, HTML-escaped by server).
- Channel format section: states both segments are HTML-escaped server-side; channel values are server-controlled and not user-injectable (T-262-04 mitigation).
- Subscribe + swap: numbered list documenting the client runtime flow (channelMap, WebSocket open, subscribe message, innerHTML swap on fragment event).
- States: no WASM, no client-side reactive state, no `eval`.
- Limitations: one element per unique channel; no auto-reconnect; no list/collection reconciliation (explicit non-goal).

### Task 2 — asset!() + ferro assets fetch + #[memoize]

**`docs/src/features/ferro-assets.md`:**
- Added `## Compile-time Asset Embedding` section clearly separated from the pipeline transform content (explains it is a framework macro, not a pipeline transform).
- `rust,ignore` example showing `asset!("assets/app.js")` returning a content-hashed `&'static str` URL.
- States: path is call-site-source-relative; bytes registered once via `OnceLock`; MIME inferred from extension.
- Mount requirement: app must mount `ferro::bundle` serving or the hashed URL returns 404.
- Added `## ferro assets fetch` section documenting `ferro assets fetch iconify` and `ferro assets fetch fontsource`.
- States explicitly: fetched files are NOT auto-wired into `asset!()` calls or route generation.

**`docs/src/features/projections.md`:**
- Added `## Request-Scoped Render Deduplication` section at end of file (after `## MCP CRUD Opt-In`).
- `rust,ignore` example showing `#[memoize]` on `async fn fetch_stock(warehouse_id: String)`.
- Semantics documented: request-scoped `MEMO_STORE`; coalescing; error caching; graceful no-op outside request scope.
- Complement relationship: `eager_loading`/`BatchLoad` batch up front; `#[memoize]` deduplicates during render pass; can be used together.
- States explicitly: not a cross-request cache (use `ferro-cache` for that).

**`docs/src/SUMMARY.md`:** Unchanged — all four pages were already in the TOC; no new pages were created.

### Task 3 — mdBook build gate

- `mdbook --version` → `mdbook v0.5.2` (binary present, no install needed).
- `mdbook build docs/` → exit 0; output: `HTML book written to docs/book`.
- No missing-file errors; no broken-link errors.
- SC-3 gate: green.

## Deviations from Plan

None — plan executed exactly as written.

- Task 1 STEP C (quick local check) deferred to Task 3 as designed; both files are pure markdown and passed the build.
- No new pages created (D-06); SUMMARY.md unchanged as stated in Task 2 STEP C.
- D-08 confirmed: `render_live_fragment` in `containers.rs` emits only `data-*` attributes, no Tailwind classes. No `ferro-base.css` regen run.

## Known Stubs

None. All four sections document real shipped behavior with faithful contracts derived from the authoritative sources (RESEARCH.md §Code Examples, containers.rs, live_fragment.rs, ferro-macros/src/asset.rs, ferro-macros/src/memoize.rs).

## Threat Flags

T-262-04 mitigated: `runtime-primitives.md` `data-channel` section states that channel values are server-controlled and HTML-escaped (not user-injectable) and that the client runtime does no eval and holds no reactive state. See Contract table and Channel format subsections.

No new threat surface introduced (documentation-only changes; no new network endpoints, auth paths, or schema changes).

## Self-Check: PASSED

Files modified exist on disk:
- docs/src/json-ui/components.md — FOUND (contains `### LiveFragment`)
- docs/src/json-ui/runtime-primitives.md — FOUND (contains `data-live-fragment`)
- docs/src/features/ferro-assets.md — FOUND (contains `asset!(`)
- docs/src/features/projections.md — FOUND (contains `memoize`)

Commits exist:
- 8ea71f21 — Task 1 (LiveFragment component + runtime behavior)
- 53b93606 — Task 2 (asset!() + ferro assets fetch + #[memoize])
- Task 3 had no file changes (mdBook build evidence only)

mdBook build: exit 0 (mdbook v0.5.2, `HTML book written to docs/book`)
