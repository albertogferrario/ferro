---
phase: 241
slug: derive-crud-plan-wire-crud-verbs-into-framework-write
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-23
---

# Phase 241 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust native `#[test]` + `#[tokio::test]` (cargo) |
| **Config file** | `Cargo.toml` (workspace) — no separate test config |
| **Quick run command** | `cargo test -p ferro-projections executor --all-features` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~varies (workspace); quick scoped run ~seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings`
- **After every plan wave:** Run `cargo test --all-features` (full workspace gate)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** scoped `cargo test -p <crate>` per task; full gate per wave

---

## Per-Task Verification Map

| # | Requirement | SC | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---|-------------|----|-----------------|-----------|-------------------|-------------|--------|
| 1 | CRUD-06 | — | `derive_crud_plan` Create plan correct | unit table | `cargo test -p ferro-projections derive_crud_plan_create --all-features` | ❌ W0 | ⬜ pending |
| 2 | CRUD-06 | — | `derive_crud_plan` Update plan correct | unit table | `cargo test -p ferro-projections derive_crud_plan_update --all-features` | ❌ W0 | ⬜ pending |
| 3 | CRUD-06 | — | `derive_crud_plan` Delete plan correct | unit table | `cargo test -p ferro-projections derive_crud_plan_delete --all-features` | ❌ W0 | ⬜ pending |
| 4 | CRUD-06 | — | Verb-not-enabled → error | unit table | `cargo test -p ferro-projections derive_crud_plan_verb_not_enabled --all-features` | ❌ W0 | ⬜ pending |
| 5 | CRUD-06 | — | `CrudPlan` serde round-trip | unit | `cargo test -p ferro-projections crud_plan_serde_round_trip --all-features` | ❌ W0 | ⬜ pending |
| 6 | CRUD-06 | SC#1 | CREATE inserts row, returns record | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_create_inserts_row --all-features` | ❌ W0 | ⬜ pending |
| 7 | CRUD-06 | SC#2 | UPDATE patches non-deleted row | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_update_patches_row --all-features` | ❌ W0 | ⬜ pending |
| 8 | CRUD-03 | SC#2 | UPDATE on soft-deleted row → not-found | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_update_soft_deleted_not_found --all-features` | ❌ W0 | ⬜ pending |
| 9 | CRUD-03 | SC#2 | DELETE sets `deleted_at` (soft) | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_delete_sets_deleted_at --all-features` | ❌ W0 | ⬜ pending |
| 10 | CRUD-03 | SC#2 | Soft-deleted row absent from `list_` | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_deleted_row_hidden_from_list --all-features` | ❌ W0 | ⬜ pending |
| 11 | CRUD-03 | — | Delete without token → `ConfirmationRequired` | kernel unit (feature=confirmation) | `cargo test -p ferro-rs crud_delete_requires_confirmation --all-features` | ❌ W0 | ⬜ pending |
| 12 | CRUD-06 | SC#3 | Override replaces generic plan | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_override_replaces_generic --all-features` | ❌ W0 | ⬜ pending |
| 13 | CRUD-06 | — | Idempotency on create | sqlite-in-memory dispatch | `cargo test -p ferro-rs crud_create_idempotent --all-features` | ❌ W0 | ⬜ pending |
| 14 | CRUD-03 | — | Delete 2-step flow (request+confirm) | framing integration | `cargo test -p ferro-mcp-server delete_two_step_flow --all-features` | ❌ W0 | ⬜ pending |
| 15 | CRUD-06 | SC#4 | Single `dispatch_write` definition, no second CRUD dispatcher | structural/grep | `grep -rn "fn dispatch_write" framework/src/` | ✅ grep | ⬜ pending |
| 16 | CRUD-06 | D-10 | Results route through `structured` envelope | framing test | `cargo test -p ferro-mcp-server crud_result_structured_envelope --all-features` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Tests in `ferro-projections/src/executor.rs` — CRUD derivation table tests (#1–#5)
- [ ] Tests in `framework/src/write/mod.rs` (or `framework/src/write/crud_tests.rs`) — sqlite-in-memory CRUD dispatch tests (#6–#13)
- [ ] Tests in `ferro-mcp-server/src/write_dispatch.rs` — delete confirmation framing + structured-envelope tests (#14, #16)
- [ ] `setup_db()` test helper extended with `CREATE TABLE orders (id INTEGER PRIMARY KEY, …, deleted_at TEXT)` fixture shared across kernel and framing tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | — | All Phase 241 behaviors have automated verification; e2e over `:8090/mcp` is Phase 243 | — |

*All phase behaviors have automated verification (SC#4 via grep structural check).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency acceptable (scoped per-crate runs)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
