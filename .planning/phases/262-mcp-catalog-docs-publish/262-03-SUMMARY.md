---
phase: 262-mcp-catalog-docs-publish
plan: 03
subsystem: publish
tags: [publish, ci-gate, version-bump, doc-fixes]
dependency_graph:
  requires: [262-01, 262-02]
  provides: [v17.0-publish-ready]
  affects: [crates.io/ferro-rs, crates.io/ferro-bundle, crates.io/ferro-macros]
tech_stack:
  added: []
  patterns: [operator-gated-publish, wave-publish-yml, ci-exact-gate]
key_files:
  created: []
  modified:
    - Cargo.toml
    - Cargo.lock
    - ferro-macros/src/lib.rs
    - ferro-mcp/src/tools/generation_context.rs
    - ferro-notifications/src/layout.rs
    - framework/src/bundle.rs
decisions:
  - "Bumped workspace version 0.2.91 → 0.2.102 (crates.io ferro-rs was at 0.2.101 >= 0.2.91, D-11)"
  - "ferry-payments at 0.1.6 on crates.io — no rider needed (D-12)"
  - "ferro-bundle confirmed in Wave 1a of publish.yml (line 217), ferro-a2ui absent (D-13)"
  - "cargo publish --dry-run structural failure is expected: ferro-rs depends on wave-1a crates not yet published; publish.yml wave ordering resolves this"
metrics:
  duration: ~35min
  completed: "2026-07-26T21:48:56Z"
  tasks_completed: 1
  tasks_pending: 2
  files_changed: 6
---

# Phase 262 Plan 03: Publish Gate Summary

One-liner: CI-exact gate green (fmt/clippy/test/doc all exit 0), version resolved to 0.2.102, staged for operator-gated publish.

## Status

| Task | Name | Status |
|------|------|--------|
| 1 | Full CI-exact gate + version resolution + commit staging | COMPLETE |
| 2 | Pre-publish checklist + operator approval | AWAITING OPERATOR |
| 3 | Commit, push, post-publish verification | NOT STARTED |

## Task 1: Gate Results

### CI-Exact Gate

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | EXIT 0 (green) |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | EXIT 0 (green) |
| `cargo test --all-features` | EXIT 0 (green, 0 failures) |
| `RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps --all-features` | EXIT 0 (green, after 4 doc fixes) |

### Version Resolution (D-11)

- crates.io ferro-rs newest_version: **0.2.101**
- Local workspace was at: 0.2.91
- Decision: crates.io (0.2.101) >= 0.2.91 → bump to crates.io_max + 1 = **0.2.102**
- Cargo.toml line 47 updated: `"0.2.91"` → `"0.2.102"`
- No v0.2.91 tag in local tags (local tags stop at v0.2.89)

### ferro-payments Rider (D-12)

- crates.io ferro-payments newest_version: **0.1.6**
- Local ferro-payments/Cargo.toml: 0.1.6
- Decision: already at 0.1.6 on crates.io → **NO rider** (cargo will skip, already published)

### publish.yml Wave Order (D-13)

- `ferro-bundle` confirmed in WAVE1A_CRATES at line 217
- `ferro-a2ui` confirmed absent (excluded by publish = false + line 36-58 continue ;;)
- No wave changes needed

### Dry-Run Note (Structural)

`cargo publish -p ferro-rs --dry-run --allow-dirty` exits 1 with "unresolved imports" for
`ferro_bundle::{mime_from_ext, BundleResponse}`, `ferro_bundle::serve_path`, `ferro_macros::asset`,
`ferro_macros::memoize`. This is expected and not a blocker: the dry-run downloads published
versions of dependencies (0.2.101), which predate the v17.0 additions in ferro-bundle and
ferro-macros. The actual publish.yml Wave 1a publishes ferro-bundle and ferro-macros BEFORE
Wave 2 publishes ferro-rs, resolving this. The workspace-level clippy + test gate (which
uses the local path deps) proves the code is correct.

### Staged File List (D-15)

```
M  Cargo.lock
M  Cargo.toml                                   (version bump 0.2.91 → 0.2.102)
M  ferro-macros/src/lib.rs                      (doc fix: [`ferro::bundle::Bundle`] → backtick span)
M  ferro-mcp/src/tools/generation_context.rs    (doc fix: #[memoize] field doc → backtick span)
M  ferro-notifications/src/layout.rs            (doc fix: [`DEFAULT_ACCENT`] → private-item link fixed)
M  framework/src/bundle.rs                      (doc fix: [`serve`] → crate-qualified links)
```

NO `.vite/deps_temp_*` in staged set. NO `.planning/config.json`. NO phantom planning/phases/158- deletion.

### What Ships

- **LiveFragment** — builtin JSON-UI element for per-key projection live binding (Phase 260)
- **`#[memoize]`** — request-scoped render-path fetch dedup (Phase 259)
- **`asset!()`** — one-line content-hashed embed + `ferro assets fetch` CLI (Phase 261)
- **`generation_context`** guidance for all three (Phase 262 Plan 01)
- **docs/src** coverage for all three (Phase 262 Plan 02)
- **4 doc-comment broken-link fixes** surfaced by the CI docs gate (`-Dwarnings`)
- **Workspace bumped** from 0.2.91 → 0.2.102

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Broken intra-doc link in ferro-macros/src/lib.rs**
- **Found during:** Task 1 STEP B4 (docs gate)
- **Issue:** `[`ferro::bundle::Bundle`]` in a proc-macro crate doc comment — rustdoc resolved it as an intra-doc link but `ferro` is not in scope in a proc-macro crate (would create cycle)
- **Fix:** Changed to plain backtick code span: `` `ferro::bundle::Bundle` ``
- **Files modified:** ferro-macros/src/lib.rs
- **Commit:** staged, not yet committed

**2. [Rule 1 - Bug] Broken intra-doc link in framework/src/bundle.rs**
- **Found during:** Task 1 STEP B4 (docs gate)
- **Issue:** `[`serve`]` in `//!` module-level doc comments — module-level docs resolve in the PARENT crate scope (lib.rs), not the module's own scope, so `serve` was not found
- **Fix:** Changed to `[`bundle::serve`](crate::bundle::serve)` crate-qualified links
- **Files modified:** framework/src/bundle.rs
- **Commit:** staged, not yet committed

**3. [Rule 1 - Bug] Broken intra-doc link in ferro-mcp/src/tools/generation_context.rs**
- **Found during:** Task 1 STEP B4 (docs gate)
- **Issue:** `#[memoize]` in a `///` doc comment field doc — rustdoc interprets `[memoize]` as an intra-doc link, which doesn't resolve
- **Fix:** Changed to `` `#[memoize]` `` backtick code span
- **Files modified:** ferro-mcp/src/tools/generation_context.rs
- **Commit:** staged, not yet committed

**4. [Rule 1 - Bug] Private-item intra-doc link in ferro-notifications/src/layout.rs**
- **Found during:** Task 1 STEP B4 (docs gate)
- **Issue:** `[`DEFAULT_ACCENT`]` links to a private constant; `rustdoc::private-intra-doc-links` is `-D warnings` at CI
- **Fix:** Changed to inline code span with the default value: `` `DEFAULT_ACCENT` (`#0052cc`) ``
- **Files modified:** ferro-notifications/src/layout.rs
- **Commit:** staged, not yet committed

### Scope Note

All four doc-comment fixes are Rule 1 (broken behavior in the docs gate). They were introduced by Phase 261 (ferro-bundle decoupling, asset/memoize macros) and Phase 262 Plan 01 (generation_context field docs) — none were pre-existing before those phases. The docs gate (`-Dwarnings`) is part of the CI-exact gate per D-09 and feedback_ci_matrix_wider_than_local_gate.

## Known Stubs

None — all generation_context guidance is wired to real prose values. All doc pages (262-02) have real content.

## Threat Surface Scan

No new network endpoints, no new auth paths, no new file access patterns. The doc-comment fixes are non-functional changes. Version bump is a manifest change only.

## Self-Check

- [x] Staged files exist: `git status --short` shows M Cargo.toml, M Cargo.lock, M ferro-macros/src/lib.rs, M ferro-mcp/src/tools/generation_context.rs, M ferro-notifications/src/layout.rs, M framework/src/bundle.rs
- [x] No polluting paths staged (.vite, config.json, phantom-158)
- [x] Version 0.2.102 in Cargo.toml confirmed
- [x] crates.io readings recorded (ferro-rs: 0.2.101, ferro-payments: 0.1.6)
- [ ] Task 2: awaiting operator approval
- [ ] Task 3: not started (requires operator go)

## Self-Check: PARTIAL — awaiting operator gate (Task 2) and publish (Task 3)

Task 1 gate is fully green. Task 3 commit hash will be recorded after operator approval.
