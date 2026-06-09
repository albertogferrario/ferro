---
phase: 190
slug: async-rule-infrastructure-unique-rule
status: planned
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-09
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
| 190-01-01 | 01 | 1 | VALID-03 | SC4 | T-190-01 (accept) | AsyncRule trait dyn-compatible; sync API untouched; __infra_error__ sentinel documented | compile | `cargo check -p ferro-rs --lib` | ❌ W0 (created here) | ⬜ pending |
| 190-01-02 | 01 | 1 | VALID-03 | — | — | In-memory SQLite fixture (DB::init_with + widgets scratch table) for downstream DB tests | compile | `cargo check -p ferro-rs --tests` | ❌ W0 (created here) | ⬜ pending |
| 190-02-01 | 02 | 2 | VALID-01, VALID-02 | SC1, SC2 | T-190-01 | unique struct + builders + identifier guard + value-conv helper; pure-unit tests | unit | `cargo test -p ferro-rs --lib validation::` | ✅ after 190-01 | ⬜ pending |
| 190-02-02 | 02 | 2 | VALID-01, VALID-02 | SC1, SC2, SC5 | T-190-01, T-190-02, T-190-03 | per-backend COUNT (?/$N), exclude-self, identifier rejected before DB, DbErr→sentinel not field error, value bound as param | unit + async (SQLite) + SQL-string assert | `cargo test -p ferro-rs --lib validation::` | ✅ after 190-01 | ⬜ pending |
| 190-03-01 | 03 | 3 | VALID-03 | SC4 | — | AsyncValidationError enum (Validation vs Infra) + AsyncValidator builders; ActionError conversion routes Infra→500 | compile | `cargo check -p ferro-rs --lib` | ✅ after 190-01/02 | ⬜ pending |
| 190-03-02 | 03 | 3 | VALID-03 | SC3, SC4, SC5 | T-190-02, T-190-04 | sync-first; async skipped on sync-failed/null-nullable field (no DB query); __infra_error__→Infra not field error | unit + async | `cargo test -p ferro-rs --lib validation::` | ✅ after 190-01/02 | ⬜ pending |
| 190-04-01 | 04 | 4 | VALID-01, VALID-02, VALID-03 | — | T-190-01, T-190-05 | crate-root public re-exports (no bypass of guards) | compile | `cargo check -p ferro-rs --lib` | ✅ after 190-03 | ⬜ pending |
| 190-04-02 | 04 | 4 | VALID-01, VALID-02, VALID-03 | SC1, SC2, SC3, SC4, SC5 | T-190-01, T-190-05 | end-to-end via public API: duplicate→Validation, free→Ok, exclude-self→Ok, sync-first, redirect-back shape; full quality gate | integration (SQLite) + fmt + clippy + full suite | `cargo test -p ferro-rs --test async_validation_integration` then `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` | ✅ after 190-03 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**SC legend (ROADMAP success criteria):** SC1 duplicate→field error not 500 ·
SC2 exclude-self unchanged value passes · SC3 sync-first fail-fast (no DB query
on sync-failed field) · SC4 sync API byte-compatible/unchanged · SC5 DB via
`DB::connection()` singleton, nothing threaded.

---

## Wave 0 Requirements

- [ ] `framework/src/validation/async_rule.rs` — `AsyncRule` trait (Plan 01, Task 1)
- [ ] `framework/tests/async_rule_fixture.rs` — in-memory SQLite fixture via `DB::init_with` + `widgets` scratch table (Plan 01, Task 2)
- [x] `serial_test = "3"` already in `[dev-dependencies]` (framework/Cargo.toml:79) — no new deps

*Existing `cargo test` infrastructure covers the harness; no framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Postgres live UNIQUE pre-check round-trip | VALID-01 (Postgres path) | SQLite-only CI default; Postgres not provisioned in `cargo test` | Run the unique-rule suite against a `DATABASE_URL` Postgres instance; confirm `$1`/`$2` placeholders + double-quoted identifiers execute. Sign off in VERIFICATION.md. |

> Note: the Postgres path is **generated-SQL-asserted** in automated tests
> (`build_sql(DatabaseBackend::Postgres, …)` string equality on the bifurcated
> query), so the manual step is a confidence check, not the sole evidence.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (created in Plan 01)
- [x] No watch-mode flags
- [x] Feedback latency < 90s (quick command)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** planned — pending execution
