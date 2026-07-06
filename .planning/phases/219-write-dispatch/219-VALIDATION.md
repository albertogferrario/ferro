---
phase: 219
slug: write-dispatch
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 219 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[tokio::test]` (in use across ferro-mcp-server) |
| **Config file** | None — inline in test modules + `tests/` |
| **Quick run command** | `cargo test -p ferro-mcp-server -p ferro-mcp-oauth` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-mcp-server -p ferro-mcp-oauth`
- **After every plan wave:** `cargo test --all-features` + `cargo clippy --all --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + clippy green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 219-00-01 | 00 | 0 | AMCP-04 | — | RED tests + compiling skeleton (WriteDispatcher, dispatch_write, migration, error variants) | unit/integration | `cargo test -p ferro-mcp-server -p ferro-mcp-oauth` | ❌ W0 | ⬜ pending |
| 219-01-01 | 01 | 1 | AMCP-04 | T-219-02 (guard bypass) | guard re-evaluated at call time via live `GuardEvaluator`; failing guard → `isError:true`, no execution (NOT `ctx.evaluated_guards`) | integration | `cargo test -p ferro-mcp-server guard_denied_at_call_time` | ❌ W0 | ⬜ pending |
| 219-01-02 | 01 | 1 | AMCP-04 | T-219-03 (retry double-write) | two calls, same `idempotency_key` → executor fires once; exactly one DB write | unit | `cargo test -p ferro-mcp-server idempotent_replay_does_not_re_execute` | ❌ W0 | ⬜ pending |
| 219-01-03 | 01 | 1 | AMCP-04 | — | every write result parses as `rmcp::model::CallToolResult`; structured for success + error | unit | `cargo test -p ferro-mcp-server write_tool_result_parses_as_valid_mcp_content` | ❌ W0 | ⬜ pending |
| 219-02-01 | 02 | 2 | AMCP-04 | T-219-01 (cross-tenant write) | sample-app executor + guard evaluator registered; `find_for_tenant` denies cross-tenant write | integration | `cargo test -p app cross_tenant_write_denied` | ❌ W0 | ⬜ pending |
| 219-02-02 | 02 | 2 | AMCP-04 | — | write call records a ferro-audit entry (tool, tenant, action, record id) recoverable after the fact | integration | `cargo test -p app write_call_produces_audit_entry` | ❌ W0 | ⬜ pending |
| 219-02-03 | 02 | 2 | AMCP-04 | T-219-03 | idempotent replay end-to-end through the app path (one mutation after two identical calls) | integration | `cargo test -p app idempotent_write_e2e` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-server/src/write_dispatch.rs` — RED unit tests (SC#1 guard, SC#3 idempotency, SC#5 result) + `WriteDispatcher`/`ExecutorFn`/`GuardEvaluatorFn` skeleton
- [ ] `ferro-mcp-server/tests/write_dispatch_integration.rs` — SC#1 guard-bypass fixture (guarded action hidden from list but rejected at call)
- [ ] `ferro-mcp-oauth/src/migration.rs` — `MigrationMcpIdempotencyKeys` (UNIQUE on `(tenant_id, idempotency_key)`)
- [ ] `app/src/tests/mcp_write_dispatch.rs` — SC#2 cross-tenant, SC#3 idempotency e2e, SC#4 audit
- [ ] `TenantScoped` impl on the sample `Order` model — prerequisite for the SC#2 fixture
- [ ] `ferro-audit` added as a `ferro-mcp-server` (or app) dependency + `CreateAuditLogTable` migration registered in the app; publish.yml wave ordering verified (`ferro-audit` before `ferro-mcp-server`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | — | All phase behaviors have automated coverage | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
