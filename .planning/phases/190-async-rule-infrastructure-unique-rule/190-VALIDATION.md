---
phase: 190
slug: async-rule-infrastructure-unique-rule
status: executed
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-09
validated: 2026-06-09
---

# Phase 190 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust) + `serial_test` (already in dev-deps) |
| **Config file** | none — uses existing workspace test harness |
| **Quick run command** | `cargo test -p ferro-rs --lib validation::` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30–90 s (quick); full suite minutes |

> DB-backed rules are exercised against in-memory SQLite via `DB::init_with` /
> `serial_test` fixtures (research §Test Strategy). SQLite is the CI default;
> Postgres-specific placeholder/quoting paths are covered by unit assertions on
> the generated SQL string (`build_sql`), not a live Postgres connection.
>
> NOTE: crate package name is `ferro-rs` (not `ferro`); all commands use
> `-p ferro-rs`.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-rs --lib validation::`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green + `cargo clippy --all --all-targets -- -D warnings` clean
- **Max feedback latency:** ~90 s (quick)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | SC | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|----|-----------|-----------------|-----------|-------------------|-------------|--------|
| 190-01-01 | 01 | 1 | VALID-03 | SC4 | T-190-01 (accept) | AsyncRule trait dyn-compatible; sync API untouched; __infra_error__ sentinel documented | compile | `cargo check -p ferro-rs --lib` | ✅ async_rule.rs | ✅ green |
| 190-01-02 | 01 | 1 | VALID-03 | — | — | In-memory SQLite fixture (DB::init_with + widgets scratch table) for downstream DB tests | compile | `cargo check -p ferro-rs --tests` | ✅ async_rule_fixture.rs | ✅ green |
| 190-02-01 | 02 | 2 | VALID-01, VALID-02 | SC1, SC2 | T-190-01 | unique struct + builders + identifier guard + value-conv helper; pure-unit tests | unit | `cargo test -p ferro-rs --lib validation::` | ✅ rules_async.rs | ✅ green (8 unit tests) |
| 190-02-02 | 02 | 2 | VALID-01, VALID-02 | SC1, SC2, SC5 | T-190-01, T-190-02, T-190-03 | per-backend COUNT (?/$N), exclude-self, identifier rejected before DB, DbErr→sentinel not field error, value bound as param | unit + async (SQLite) + SQL-string assert | `cargo test -p ferro-rs --lib validation::` | ✅ rules_async.rs | ✅ green (6 tests: 2 guard + 4 #[serial] DB) |
| 190-03-01 | 03 | 3 | VALID-03 | SC4 | — | AsyncValidationError enum (Validation vs Infra) + AsyncValidator builders | compile | `cargo check -p ferro-rs --lib` | ✅ async_validator.rs | ✅ green |
| 190-03-02 | 03 | 3 | VALID-03 | SC3, SC4, SC5 | T-190-02, T-190-04 | sync-first; async skipped on sync-failed/null-nullable field (no DB query); __infra_error__→Infra not field error | unit + async | `cargo test -p ferro-rs --lib validation::` | ✅ async_validator.rs | ✅ green (7 tests) |
| 190-04-01 | 04 | 4 | VALID-01, VALID-02, VALID-03 | — | T-190-01, T-190-05 | crate-root public re-exports (no bypass of guards) | compile | `cargo check -p ferro-rs --lib` | ✅ mod.rs + lib.rs | ✅ green |
| 190-04-02 | 04 | 4 | VALID-01, VALID-02, VALID-03 | SC1, SC2, SC3, SC4, SC5 | T-190-01, T-190-05 | end-to-end via public API: duplicate→Validation, free→Ok, exclude-self→Ok, sync-first, redirect-back shape; full quality gate | integration (SQLite) + fmt + clippy + full suite | `cargo test -p ferro-rs --test async_validation_integration` then `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ async_validation_integration.rs | ✅ green (5 integration tests + full gate) |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**SC legend (ROADMAP success criteria):** SC1 duplicate→field error not 500 ·
SC2 exclude-self unchanged value passes · SC3 sync-first fail-fast (no DB query
on sync-failed field) · SC4 sync API byte-compatible/unchanged · SC5 DB via
`DB::connection()` singleton, nothing threaded.

---

## Wave 0 Requirements

- [x] `framework/src/validation/async_rule.rs` — `AsyncRule` trait (Plan 01, Task 1)
- [x] `framework/tests/async_rule_fixture.rs` — in-memory SQLite fixture via `DB::init_with` + `widgets` scratch table (Plan 01, Task 2)
- [x] `serial_test = "3"` already in `[dev-dependencies]` (framework/Cargo.toml:79) — no new deps

*Existing `cargo test` infrastructure covers the harness; no framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Postgres live UNIQUE pre-check round-trip | VALID-01 (Postgres path) | SQLite-only CI default; Postgres not provisioned in `cargo test` | ✅ CLOSED 2026-06-09 — `framework/tests/async_validation_pg_gate.rs::pg_unique_rule_placeholder_and_quoting_path` ran green against live Postgres (postgres@localhost:5432): duplicate→Validation error, free→Ok, exclude-self→Ok, exercising `$1`/`$2` placeholders + double-quoted identifiers. Now an `#[ignore]`d runnable-on-demand test (`DATABASE_URL=… cargo test -p ferro-rs --test async_validation_pg_gate -- --ignored`). |

> Note: the Postgres path is also **generated-SQL-asserted** in automated unit
> tests (`build_sql(DatabaseBackend::Postgres, …)` string equality). The live-PG
> gate above is now an executable test, not a manual step.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (created in Plan 01)
- [x] No watch-mode flags
- [x] Feedback latency < 90s (quick command)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** executed & verified — all 8 tasks green, VERIFICATION.md 5/5 passed

---

## Validation Audit 2026-06-09

Retroactive Nyquist audit of executed phase (`/gsd-validate-phase 190`). VALIDATION.md
was at plan-time state (`status: planned`, all tasks `⬜ pending`); reconciled against
the four SUMMARY files, VERIFICATION.md (5/5), and on-disk test files.

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

Every requirement (VALID-01, VALID-02, VALID-03) has automated verification. Test
inventory confirmed on disk: `rules_async.rs` (14 tests), `async_validator.rs`
(7 tests), `async_validation_integration.rs` (5 `#[tokio::test]`), plus compile gates
on the trait and re-export tasks. Full quality gate (`fmt` + `clippy -D warnings` +
`cargo test --all-features`) green at commit `9c311935`. No tests generated — coverage
was already complete at execution time.

The single Manual-Only entry (live-Postgres `unique` round-trip) is unchanged: it
remains a confidence check, not the sole evidence — the Postgres SQL path is
generated-SQL-asserted in automated unit tests.
