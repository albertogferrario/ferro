---
phase: 242
slug: write-authorization-tenant-injection-non-disclosure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-24
---

# Phase 242 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust workspace; `#[tokio::test]` for async dispatch tests; sqlite-in-memory via sea-orm) |
| **Config file** | none — existing workspace test harness |
| **Quick run command** | `cargo test -p ferro-projections -p ferro-mcp-server` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~120–300 seconds (full); ~30s (quick) |

---

## Sampling Rate

- **After every task commit:** Run the relevant crate's `cargo test -p <crate>`
- **After every plan wave:** Run `cargo test --all-features` for touched crates
- **Before `/gsd-verify-work`:** Full suite (fmt + clippy + test) must be green
- **Max feedback latency:** ~300 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 242-01-* | 01 | 1 | CRUD-05 | T-242-02 | derive_crud_plan fills `tenant_column` from `svc.tenant_column` | unit | `cargo test -p ferro-projections tenant_column` | ✅ | ⬜ pending |
| 242-02-* | 02 | 2 | CRUD-05 | T-242-02/03 | execute_crud_plan binds tenant_id (INSERT col on create; `AND <col>=?` on update/delete) | dispatch (sqlite-in-mem) | `cargo test -p ferro framework::write` | ✅ | ⬜ pending |
| 242-02-* | 02 | 2 | CRUD-05 | T-242-03 | cross-tenant + soft-deleted update/delete → RecordNotFound (non-disclosure) | dispatch (sqlite-in-mem) | `cargo test -p ferro framework::write` | ✅ | ⬜ pending |
| 242-03-* | 03 | 3 | CRUD-05 | T-242-01 | write-ability fail-closed: `write_authorized != Some(true)` denies before dispatch | framing | `cargo test -p ferro-mcp-server write` | ✅ | ⬜ pending |
| 242-03-* | 03 | 3 | CRUD-05 | T-242-01 | read-scope key rejected on write tools (verify existing scope gate holds) | framing | `cargo test -p ferro-mcp-server scope` | ✅ | ⬜ pending |
| 242-04-* | 04 | 3 | CRUD-07 | T-242-04 | validate() rejects CRUD-verb-without-mcp_write_ability at boot | unit | `cargo test -p ferro-projections validate` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. (Test patterns already exist:
sqlite-in-memory dispatch tests in `framework/src/write/mod.rs`, framing tests in
`ferro-mcp-server`, and `validate_rejects_*` unit tests in `ferro-projections/src/service.rs`.)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Gate-deny against the running `:8090/mcp` host with a real `read_write` key failing `mcp_write_ability` | CRUD-05 | End-to-end host wiring (`app/src/controllers/mcp.rs`) is exercised fully in Phase 243's e2e; Phase 242 covers it at the unit/framing layer | Deferred to Phase 243 app-integration e2e |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 300s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
