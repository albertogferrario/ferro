---
phase: 239
slug: soft-delete-data-model-deleted-at-migration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-23
---

# Phase 239 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` (async, sqlite in-memory) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-projections -p ferro-mcp-server 2>&1 \| tail -5` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~quick: <30s scoped; full: minutes (workspace) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-projections -p ferro-mcp-server`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite green + `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings`
- **Max feedback latency:** ~30 seconds (scoped run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 239-01-01 | 01 | 1 | SC#1 | — | migration adds nullable `deleted_at`; no data exposure | integration | `cargo run -p app -- db migrate` (SQLite + Postgres) | ❌ W0 | ⬜ pending |
| 239-02-01 | 02 | 1 | SC#2 | — | resolver defaults match existing dispatch behavior (no scope widening) | unit | `cargo test -p ferro-projections resolved_` | ❌ W0 | ⬜ pending |
| 239-02-02 | 02 | 1 | SC#4 | T-239-01 | `is_server_injected_field` true for Identifier/CreatedAt/tenant col → kept out of agent write inputs | unit | `cargo test -p ferro-projections server_injected` | ❌ W0 | ⬜ pending |
| 239-03-01 | 03 | 2 | SC#3 | T-239-02 | soft-deleted row invisible to reads by construction (data-layer, not per-tool) | unit (sqlite in-memory) | `cargo test -p ferro-mcp-server soft_delete` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `app/src/migrations/m20260623_add_deleted_at_to_orders.rs` — new additive migration (SC#1); register in `app/src/migrations/mod.rs`
- [ ] `ferro-projections/src/service.rs` — `resolved_table()`, `resolved_soft_delete_column()`, `is_server_injected_field()` + table tests (SC#2, SC#4)
- [ ] `ferro-mcp-server/src/dispatch.rs` — extend the in-file test `CREATE TABLE orders (...)` seed with a nullable `deleted_at`, seed a soft-deleted row, add a `soft_delete`-named exclusion test (SC#3)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Postgres `db:migrate` applies clean | SC#1 | CI/local SQLite is the default test backend; Postgres path needs a running PG instance | Run the app's migrate command against a Postgres `DATABASE_URL`; confirm `deleted_at` column exists and is nullable. (If CI already exercises a Postgres matrix, prefer that over manual.) |

*SQLite migration application is automatable in the default test path; Postgres is manual unless a PG test matrix exists.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
