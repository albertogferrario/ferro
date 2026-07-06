---
phase: 153
slug: ferro-audit-crate-structured-before-after-audit-log-with-rep
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-13
---

# Phase 153 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `tokio::test` + inline `#[cfg(test)]` modules + cargo integration tests |
| **Config file** | none — cargo's built-in test runner |
| **Quick run command** | `cargo test -p ferro-audit` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds (ferro-audit unit + integration) / ~3-5 minutes (full workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-audit`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds (quick), ~5 minutes (full workspace)

---

## Per-Task Verification Map

> All tests come from CONTEXT.md D-30 (unit) + D-31 (integration). `phase_req_ids` is null (feature-driven phase) — D-XX decisions are the must-haves.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 153-01-01 | 01 | 1 | D-01..D-04 | — | Crate manifest valid; compiles | unit | `cargo check -p ferro-audit` | ❌ W0 | ⬜ pending |
| 153-02-01 | 02 | 1 | D-04, D-39 | — | Crate registered in workspace; appears in Wave 1a | unit | `cargo check --workspace && grep -q 'ferro-audit' .github/workflows/publish.yml` | ❌ W0 | ⬜ pending |
| 153-03-01 | 03 | 2 | D-05..D-08, D-11..D-13 | T-153-01 | Typed actor/target compile and round-trip via serde | unit | `cargo test -p ferro-audit actor_target` | ❌ W0 | ⬜ pending |
| 153-04-01 | 04 | 2 | D-15..D-17 | — | `AuditError` enum with all variants; Display prefix `"audit: …"` | unit | `cargo test -p ferro-audit error_display` | ❌ W0 | ⬜ pending |
| 153-05-01 | 05 | 3 | D-18..D-22 | T-153-02 | Migration creates `audit_log` with all columns + 2 indexes | unit | `cargo test -p ferro-audit migration_creates_table_and_indexes` | ❌ W0 | ⬜ pending |
| 153-06-01 | 06 | 4 | D-09..D-14, D-30-1 | — | Builder happy path: `write()` returns entry with non-nil `id` + `created_at` | unit | `cargo test -p ferro-audit happy_path` | ❌ W0 | ⬜ pending |
| 153-06-02 | 06 | 4 | D-30-2, D-16 | — | Missing `action` → `AuditError::MissingAction` | unit | `cargo test -p ferro-audit missing_action` | ❌ W0 | ⬜ pending |
| 153-06-03 | 06 | 4 | D-30-3, D-10 | — | Missing `target` writes successfully + `tracing::warn!` emitted | unit | `cargo test -p ferro-audit missing_target_writes` | ❌ W0 | ⬜ pending |
| 153-06-04 | 06 | 4 | D-30-4, D-11 | — | `before` / `after` JSON round-trip preserved through DB | unit | `cargo test -p ferro-audit json_roundtrip` | ❌ W0 | ⬜ pending |
| 153-06-05 | 06 | 4 | D-30-5, D-05 | — | `AuditActor::System` / `Anonymous` persist `actor_id = NULL` | unit | `cargo test -p ferro-audit actor_null_id` | ❌ W0 | ⬜ pending |
| 153-07-01 | 07 | 5 | D-23, D-30-6 | — | `history_for_target` ordering (`created_at ASC`) | unit | `cargo test -p ferro-audit history_ordering` | ❌ W0 | ⬜ pending |
| 153-07-02 | 07 | 5 | D-23, D-30-7 | — | `recent_by_actor` ordering (`DESC`) + `limit` enforcement | unit | `cargo test -p ferro-audit recent_by_actor` | ❌ W0 | ⬜ pending |
| 153-07-03 | 07 | 5 | D-23 | — | `recent(limit)` returns N most recent entries | unit | `cargo test -p ferro-audit recent_global` | ❌ W0 | ⬜ pending |
| 153-08-01 | 08 | 5 | D-24, D-30-9 | — | `reconstruct_state` on empty → `None`; sequence → merged object | unit | `cargo test -p ferro-audit reconstruct_state` | ❌ W0 | ⬜ pending |
| 153-09-01 | 09 | 5 | D-26, D-30-8 | — | `prune_older_than` returns count + deletes only rows strictly older | unit | `cargo test -p ferro-audit prune_older_than` | ❌ W0 | ⬜ pending |
| 153-10-01 | 10 | 6 | D-31 | — | Integration: lifecycle → `history_for_target` → `reconstruct_state` equals expected | integration | `cargo test -p ferro-audit --test replay_round_trip` | ❌ W0 | ⬜ pending |
| 153-11-01 | 11 | 7 | D-35, D-36 | — | Module rustdoc + `docs/src/database/audit-log.md` page exists | unit | `cargo doc --no-deps -p ferro-audit && test -f docs/src/database/audit-log.md` | ❌ W0 | ⬜ pending |
| 153-12-01 | 12 | 8 | D-38..D-40 | — | Workspace version bumped + CHANGELOG entry present | unit | `grep -E '^version = "0\.2\.31"' Cargo.toml && grep -q 'ferro-audit' CHANGELOG.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

> Task IDs above are indicative — final plan numbering is the planner's call. Wave layout is also indicative. The validation contract is the test list and commands, not the plan structure.

---

## Wave 0 Requirements

- [ ] `ferro-audit/Cargo.toml` — new crate manifest (Wave 1a)
- [ ] `ferro-audit/src/lib.rs` — crate entry point with module-level rustdoc (D-35)
- [ ] `ferro-audit/src/error.rs` — `AuditError` enum (D-15..D-17)
- [ ] `ferro-audit/src/actor.rs` — `AuditActor` enum (D-05..D-06)
- [ ] `ferro-audit/src/target.rs` — `AuditTarget` struct (D-07..D-08)
- [ ] `ferro-audit/src/entry.rs` — `AuditEntry` SeaORM entity + builder API (D-09..D-14)
- [ ] `ferro-audit/src/migration.rs` — `CreateAuditLogTable` migration (D-18..D-22)
- [ ] `ferro-audit/src/query.rs` — `history_for_target` / `recent_by_actor` / `recent` (D-23)
- [ ] `ferro-audit/src/replay.rs` — `reconstruct_state` shallow-merge helper (D-24)
- [ ] `ferro-audit/src/prune.rs` — `prune_older_than` helper (D-26)
- [ ] `ferro-audit/tests/replay_round_trip.rs` — D-31 integration test
- [ ] `ferro-audit/README.md` — crate README
- [ ] `docs/src/database/audit-log.md` — user-facing doc page (D-36)
- [ ] `docs/SUMMARY.md` — nav entry under `Database`
- [ ] Workspace `Cargo.toml` — add `"ferro-audit"` to `[workspace.members]`; bump `[workspace.package] version = "0.2.31"` (per RESEARCH version-drift correction — D-38 says 0.2.26 but Phase 152 already bumped past it)
- [ ] `.github/workflows/publish.yml` — append `ferro-audit` to `WAVE1A_CRATES`
- [ ] `README.md` (workspace root) — workspace crates table row
- [ ] `CLAUDE.md` — Workspace Structure table row
- [ ] `CHANGELOG.md` — `ferro-audit` section (D-40)

*Indicative file-layout (matches CONTEXT.md `<code_context>` and Claude's Discretion D-allowances). Planner may consolidate `actor.rs` + `target.rs` into a single file if it improves clarity.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First crates.io publish (bootstrap) | D-39 | CI token has publish-update only; new-crate first publish requires personal-token from local terminal | After phase verifies + tag pushed: from local terminal run `cargo publish -p ferro-audit` with personal `CARGO_REGISTRY_TOKEN` set; verify at https://crates.io/crates/ferro-audit |

*All other phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s (quick) / < 5min (full)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
