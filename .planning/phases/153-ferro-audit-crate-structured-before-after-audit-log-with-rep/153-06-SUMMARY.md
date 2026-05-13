---
phase: 153
plan: "06"
subsystem: ferro-audit
tags: [sea-orm, audit-log, integration-test, documentation, changelog, release]
dependency_graph:
  requires: [153-01, 153-02, 153-03, 153-04, 153-05]
  provides: [D-31-integration-test, audit-log-doc-page, changelog-entry, pre-release-gate]
  affects: [docs/src/database/, CHANGELOG.md, ferro-audit public API surface on crates.io]
tech_stack:
  added: []
  patterns:
    - "sea_orm_migration::prelude::* import in integration tests (brings async_trait into scope)"
    - "TestMigrator inline struct + MigratorTrait impl in integration test (same pattern as unit tests)"
    - "11-section mdBook doc page mirroring atomic-updates.md structure"
    - "CHANGELOG newest-on-top section ordering (## ferro-audit above ## ferro-orm)"
key_files:
  created:
    - ferro-audit/tests/replay_round_trip.rs
    - docs/src/database/audit-log.md
  modified:
    - docs/src/SUMMARY.md
    - CHANGELOG.md
decisions:
  - "Use sea_orm_migration::prelude::* in integration test (not bare MigratorTrait import) — brings async_trait into scope, matching the pattern established in unit test modules"
  - "CHANGELOG version 0.2.31 (not stale D-38 value 0.2.26) — uses Cargo.toml value at execution time per RESEARCH F-07"
  - "Pre-release gate ran sequentially (fmt → clippy → build → test → doc) — all 5 commands exit 0"
metrics:
  duration: "448s"
  completed: "2026-05-13T18:16:25Z"
  tasks_completed: 5
  files_modified: 4
---

# Phase 153 Plan 06: Integration Test + Docs + CHANGELOG + Pre-release Gate Summary

**One-liner:** D-31 integration test proves replay round-trip end-to-end; audit-log.md ships the 11-section doc page; CHANGELOG records the v0.2.31 initial release; all 5 pre-release gate commands exit 0; Task 6 (first publish) awaits operator bootstrap.

## What Was Built

### Task 1 — `ferro-audit/tests/replay_round_trip.rs` (commit `711dbcb0`)

Single `#[tokio::test]` proving the D-31 design promise: a 5-entry inventory unit lifecycle (created → 3 adjustments → status_changed) is written via the public `AuditEntry::record(...).write(&conn)` API, then queried with `history_for_target`, then folded with `reconstruct_state`. Final assertion:

```rust
assert_eq!(reconstructed, json!({ "id": "abc", "quantity": 30, "status": "low_stock" }));
```

Test result: `test replay_round_trip_inventory_unit_lifecycle ... ok` (4.42s).

### Task 2 — `docs/src/database/audit-log.md` (commit `695bd660`)

178-line documentation page with 11 sections mirroring `atomic-updates.md`:

1. H1 + opening paragraph (historical twin / ferro-events metaphor)
2. The Anti-Pattern (tracing::info! + ad-hoc JSON column)
3. The Replacement (one-call builder example)
4. API (builder methods table + query helpers table + retention helper)
5. `AuditActor` (5-variant table with DB column representation)
6. `AuditTarget` (struct fields, dotted-namespace convention)
7. Schema (12-column table + 2 indexes)
8. Replay (shallow-merge semantics + None / tombstone cases)
9. Retention and Pruning (prune_older_than + GDPR + PII caller responsibility)
10. Errors (3-variant table)
11. Postgres vs SQLite (dialect note)

Tone: neutral, no marketing language, code blocks use `rust,ignore`. PII caller responsibility documented in Retention section (T-153-03 mitigation).

### Task 3 — `docs/src/SUMMARY.md` nav entry (commit `d38e7f36`)

Added `- [Audit Log](database/audit-log.md)` at line 35, immediately after `- [Atomic Updates](database/atomic-updates.md)`.

### Task 4 — Pre-release gate (no commit — verification only)

All five commands exit 0:

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | OK (no output) |
| `cargo clippy --all --all-targets -- -D warnings` | OK (ferro-audit compiled cleanly) |
| `cargo build --workspace` | OK (14.74s, all 24 workspace members) |
| `cargo test --all-features` | OK — ferro-audit: 27 unit tests + 1 integration test = 28 tests passed |
| `cargo doc --no-deps -p ferro-audit` | OK (0 warnings, generated index.html) |

### Task 5 — `CHANGELOG.md` ferro-audit section (commit `59ec2763`)

Added `## ferro-audit` section at the top (above `## ferro-orm`), with:
- `### [0.2.31] — 2026-05-13`
- "Initial release. Phase 153 — `ferro-audit` crate…"
- 9 `#### Added` bullets covering the full public surface

Version is `0.2.31` (Cargo.toml value at execution time), not the stale `0.2.26` from CONTEXT D-38 (superseded by RESEARCH F-07).

### Task 6 — First-publish bootstrap (CHECKPOINT — awaiting operator)

`cargo publish -p ferro-audit` must be run from a local terminal with a personal `publish-new`-scoped crates.io token. CI's `CARGO_REGISTRY_TOKEN` has `publish-update` scope only and cannot create new crates. This is the same pattern as Phase 151 (`ferro-wallet`) and Phase 152 (`ferro-orm`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `async_trait` not resolved in integration test via bare `MigrationTrait`/`MigratorTrait` import**

- **Found during:** Task 1 (first compile attempt)
- **Issue:** Integration test used `use sea_orm_migration::{MigrationTrait, MigratorTrait};` — the bare import does not bring `async_trait` into scope. The `#[async_trait::async_trait]` attribute on `impl MigratorTrait for TestMigrator` failed with `E0433: use of unresolved module or unlinked crate async_trait`.
- **Fix:** Changed to `use sea_orm_migration::prelude::*;` — matches the pattern already established in the unit test modules (`entry.rs`, `query.rs`, `prune.rs`).
- **Files modified:** `ferro-audit/tests/replay_round_trip.rs`
- **Commit:** `711dbcb0` (fixed before first passing run)

**2. [Rule 1 - Bug] `cargo fmt` required line length adjustment in integration test**

- **Found during:** Task 1 (fmt gate before compile)
- **Issue:** Import block and `.expect()` chain exceeded rustfmt line width.
- **Fix:** Reformatted imports to single line and chained `.expect()` on same line as function call per rustfmt output.
- **Files modified:** `ferro-audit/tests/replay_round_trip.rs`
- **Commit:** `711dbcb0`

## Known Stubs

None. All files are complete implementations.

## Auth Gates

Task 6 is a `checkpoint:human-action` — not an auth gate in the technical sense, but a first-publish bootstrap requiring a personal crates.io token. Documented as expected flow per Phase 151 and Phase 152 precedents.

## Threat Surface Scan

No new network endpoints, auth paths, or schema changes introduced. The only security-sensitive moment in this plan is Task 6 (personal token for `cargo publish`) — mitigated by T-153-06-01 (token used only from local terminal, never committed or piped through CI).

The documentation page explicitly states PII caller responsibility (T-153-03 mitigation):
> **PII responsibility**: `ferro-audit` does NOT automatically redact `before` / `after` payloads. The caller must remove or hash PII fields BEFORE passing JSON to the builder.

## Version Note

CONTEXT D-38 referenced `0.2.26` as the target version — this was stale at gather time (Phase 152 had already bumped to `0.2.30`). RESEARCH F-07 correctly identified the current version as `0.2.30` and predicted `0.2.31` for Phase 153. The CHANGELOG entry and published version both use `0.2.31` (the actual Cargo.toml value at execution time).

## What Remains (Task 6 — operator action)

```bash
# Step 1: Confirm version
grep -E '^version = ' Cargo.toml | head -1

# Step 2: Sanity-check name availability
cargo search ferro-audit | head -5

# Step 3: First publish (personal token with publish-new scope)
cargo publish -p ferro-audit --token <PERSONAL_PUBLISH_TOKEN>

# Step 4: Verify at https://crates.io/crates/ferro-audit

# Step 5: Push commits to master
# Step 6: Confirm GH Actions publish.yml is green
```

Resume signal: type "published" with the actual version string after `https://crates.io/crates/ferro-audit` shows the version.

## Self-Check: PASSED

Files created/exist:
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-audit/tests/replay_round_trip.rs` — FOUND
- `/Users/alberto/repositories/albertogferrario/ferro/docs/src/database/audit-log.md` — FOUND

Commits exist (from `git log --oneline -5`):
- `711dbcb0` test(153-06): add D-31 replay_round_trip integration test — FOUND
- `695bd660` docs(153-06): add audit-log.md user-facing documentation page — FOUND
- `d38e7f36` docs(153-06): add Audit Log nav entry to docs/src/SUMMARY.md — FOUND
- `59ec2763` docs(153-06): add ferro-audit initial release CHANGELOG entry — FOUND
