---
phase: 191
slug: constraintmap-portable-unique-violation-detection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-09
---

# Phase 191 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust) + `tokio::test` + `serial_test` (already in dev-deps) |
| **Config file** | none — existing workspace test harness |
| **Quick run command** | `cargo test -p ferro-rs --lib validation::constraint_map` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30–90 s (quick); full suite minutes |

> SQLite path fully `cargo test`-able via in-memory SQLite with a real UNIQUE
> INDEX (reuse the Phase 190 `widgets` fixture pattern, extended with a unique
> index). The Postgres constraint-name path (`DatabaseError::constraint()`) is
> source-verified but cannot run under the SQLite-only `cargo test` default —
> covered by a documented manual gate (mirrors Phase 190).
>
> NOTE: crate package name is `ferro-rs`; all commands use `-p ferro-rs`.

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-rs --lib validation::constraint_map`
- **After every plan wave:** `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green + `cargo clippy --all --all-targets -- -D warnings` clean + Postgres manual gate signed in 191-VERIFICATION.md
- **Max feedback latency:** ~90 s (quick)

---

## Per-Task Verification Map

| Req / SC | Behavior | Test Type | Automated Command | File Exists | Status |
|----------|----------|-----------|-------------------|-------------|--------|
| VALID-04 / SC1 | `try_map` returns `Ok(ValidationError)` (field + message) on a matching UNIQUE violation | unit + integration (SQLite) | `cargo test -p ferro-rs --lib validation::constraint_map` | ❌ W0 | ⬜ pending |
| VALID-04 / SC2 | A non-UNIQUE `DbErr` (and an unregistered constraint) returns `Err(DbErr)` UNCHANGED — never swallowed, never panics; reaches `From<DbErr> for ActionError` (action.rs:196) | unit | `cargo test -p ferro-rs --lib validation::constraint_map` | ❌ W0 | ⬜ pending |
| VALID-05 / SC3 (SQLite) | SQLite identity match: parse `table.column` from `"UNIQUE constraint failed: …"` message | integration (SQLite) | `cargo test -p ferro-rs --lib validation::constraint_map` | ❌ W0 | ⬜ pending |
| VALID-05 / SC3 (Postgres) | Postgres identity match: structured constraint name via `DatabaseError::constraint()` (no message parse) | manual gate | 191-VERIFICATION.md sign-off | N/A | ⬜ pending |
| VALID-04 / SC4 | Concurrent-insert TOCTOU simulation: seed row + UNIQUE index → duplicate INSERT → `try_map(err)` → same field-level error a proactive failure produces | integration (SQLite) | `cargo test -p ferro-rs --lib validation::constraint_map` | ❌ W0 | ⬜ pending |
| VALID-05 / SC5 | `framework/src/validation/constraint_map.rs` holds no consumer-specific constraint/field/message literals (project-agnostic-crates) | audit (grep) | `! grep -nE '"pages"|"slug"|"_unique"' framework/src/validation/constraint_map.rs` (outside doc examples) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**SC2 fall-through (key correctness gate):** the registered-but-non-matching and
the not-a-UNIQUE-violation cases must both return the ORIGINAL `DbErr` by move, so
the caller's `?` reaches the existing `From<DbErr> for ActionError` passthrough.
A test must assert the returned error is the same variant/message, not a swallowed
`Ok` or a panic.

---

## Wave 0 Requirements

- [ ] `framework/src/validation/constraint_map.rs` — `ConstraintMap` + `try_map` + `MapConstraintExt` (the call-site ergonomic); covers VALID-04, VALID-05
- [ ] In-memory SQLite fixture with a real UNIQUE INDEX (extend the Phase 190 `widgets` fixture pattern, or a sibling `constraint_map` fixture) for the SQLite identity + TOCTOU-simulation tests

*Existing `cargo test` infrastructure covers the harness; no framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Postgres constraint-name identity match (`DatabaseError::constraint()` → protocol field `'n'`) | VALID-05 (Postgres path) | SQLite-only CI default; Postgres not provisioned in `cargo test` | Against a `DATABASE_URL` Postgres instance with a named UNIQUE constraint, trigger a duplicate insert and confirm `ConstraintMap::try_map` matches on the structured constraint name (not message parsing) and returns the field error. Sign off in 191-VERIFICATION.md. |

> The Postgres detection logic is source-verified (`DatabaseError::constraint()`
> dispatches on `dyn DatabaseError` at runtime — no downcast). Where feasible,
> assert the match logic against a constructed/fixture error so the manual step is
> a confidence check, not the sole evidence.

---

## Validation Sign-Off

- [ ] All SCs have an `<automated>` verify or the documented Postgres manual gate
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers the MISSING references (constraint_map.rs + UNIQUE-index fixture)
- [ ] No watch-mode flags
- [ ] Feedback latency < 90 s (quick command)
- [ ] `nyquist_compliant: true` set in frontmatter (after Wave 0 lands green)

**Approval:** pending — to be set when plans pass the checker
