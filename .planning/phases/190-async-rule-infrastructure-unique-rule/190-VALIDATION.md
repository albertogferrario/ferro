---
phase: 190
slug: async-rule-infrastructure-unique-rule
status: draft
nyquist_compliant: false
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
| **Quick run command** | `cargo test -p ferro --lib validation::` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30–90 s (quick); full suite minutes |

> DB-backed rules are exercised against in-memory SQLite via `DB::init_with` /
> `serial_test` fixtures (research §Test Strategy). SQLite is the CI default;
> Postgres-specific placeholder/quoting paths are covered by unit assertions on
> the generated SQL string, not a live Postgres connection.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro --lib validation::`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green + `cargo clippy --all --all-targets -- -D warnings` clean
- **Max feedback latency:** ~90 s (quick)

---

## Per-Task Verification Map

> Planner fills this row-per-task. Anchor each task to a VALID-0x requirement
> and a success criterion (SC1–SC5 from ROADMAP).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 190-01-01 | 01 | 1 | VALID-03 | — | AsyncRule trait dyn-compatible; sync API unchanged (SC4) | unit | `cargo test -p ferro --lib validation::` | ❌ W0 | ⬜ pending |
| 190-0x-xx | 0x | x | VALID-01 | — | duplicate value → field error, not 500 (SC1) | unit | `cargo test -p ferro --lib validation::unique` | ❌ W0 | ⬜ pending |
| 190-0x-xx | 0x | x | VALID-02 | — | `.ignore(id)` does not reject unchanged own value (SC2) | unit | `cargo test -p ferro --lib validation::unique` | ❌ W0 | ⬜ pending |
| 190-0x-xx | 0x | x | VALID-01 | T-190-01 | identifier charset-guarded; per-backend placeholder/quoting | unit | assert generated SQL string per backend | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `framework/src/validation/` async-rule unit tests — stubs for VALID-01/02/03
- [ ] In-memory SQLite fixture (via `DB::init_with`) for uniqueness + exclude-self cases

*Existing `cargo test` infrastructure covers the harness; no framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Postgres live UNIQUE pre-check round-trip | VALID-01 (Postgres path) | SQLite-only CI default; Postgres not provisioned in `cargo test` | Run the unique-rule suite against a `DATABASE_URL` Postgres instance; confirm `$1` placeholder + quoted identifiers execute. Sign off in VERIFICATION.md. |

> Note: the Postgres path is **generated-SQL-asserted** in automated tests
> (string equality on the bifurcated query), so the manual step is a confidence
> check, not the sole evidence.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
