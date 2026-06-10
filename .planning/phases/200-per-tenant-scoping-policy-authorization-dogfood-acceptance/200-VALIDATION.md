---
phase: 200
slug: per-tenant-scoping-policy-authorization-dogfood-acceptance
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-10
---

# Phase 200 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust; SQLite in-memory for crate tests, app integration tests) |
| **Config file** | none — workspace `Cargo.toml` + per-crate `#[cfg(test)]` |
| **Quick run command** | `cargo test -p ferro-mcp-server` (dispatch tenant scoping) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~quick: <30s per crate; full gate: several minutes (serialize — one CPU op at a time) |

---

## Sampling Rate

- **After every task commit:** Run the task's targeted `cargo test -p <crate>` (quick).
- **After every plan wave:** Run `cargo clippy --all --all-targets -- -D warnings` + the touched crates' tests.
- **Before `/gsd-verify-work`:** Full gate (`fmt + clippy + test --all-features`) must be green.
- **Max feedback latency:** quick test <30s; full gate run once per wave boundary, not per task (thermal/CPU discipline).

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 200-01-xx | 01 | 1 | AMCP-10 | T-200-01 (cross-tenant read) | `dispatch` with `tenant_id=Some(A)` emits `AND "tenant_id" = ?` bound to A; never unscoped when scoped | unit | `cargo test -p ferro-mcp-server dispatch` | ✅ | ⬜ pending |
| 200-01-xx | 01 | 1 | AMCP-10 | T-200-02 (fail-open) | tenant-scoped projection + `tenant_id=None` → zero rows / deny, never `SELECT *` | unit | `cargo test -p ferro-mcp-server` | ✅ | ⬜ pending |
| 200-0x-xx | 0x | 1 | AMCP-10 | — | `ServiceDef.tenant_column` is plain metadata; `ferro-projections` gains no framework/auth dep | unit | `cargo test -p ferro-projections` + `cargo tree -p ferro-projections` shows no framework dep | ✅ | ⬜ pending |
| 200-0x-xx | 0x | 2 | AMCP-11 | T-200-03 (authz bypass) | `Gate::authorize_for(&user, ability, None)` denies → MCP tool error, no dispatch, no rows | unit/integration | `cargo test -p app` (or app handler test) | ❌ W0 | ⬜ pending |
| 200-0x-xx | 0x | 2 | AMCP-11 | T-200-04 (data disclosure on deny) | deny tool-error message contains no rows/columns/filter values; `isError: true` | unit | `cargo test` asserting error body | ❌ W0 | ⬜ pending |
| 200-0x-xx | 0x | 2 | AMCP-10 | T-200-05 (tenant context parity) | `/mcp` stack `[BearerAuthMiddleware, TenantMiddleware(JwtClaimResolver("tenant_id"))]` sets `current_tenant()` identically to web surface | integration | `cargo test -p app` middleware-order test | ❌ W0 | ⬜ pending |
| 200-0x-xx | 0x | 2 | AMCP-10 | T-200-01 | two-tenant fixture: token A → only A's orders; token B → only B's orders | integration | `cargo test -p app` two-tenant isolation test | ❌ W0 | ⬜ pending |
| 200-0x-xx | 0x | 3 | AMCP-10/11 | — | live dogfood: real MCP client browser-login → `tools/list` → `tools/call` returns tenant rows | manual | see Manual-Only | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] App integration-test harness for `/mcp` with a two-tenant SQLite fixture (2 tenants, orders per tenant, a user per tenant) — needed for T-200-01 / T-200-05 isolation tests.
- [ ] `orders` + `tenants` migrations applied in the test DB (the projection's read path has never run against a real `orders` table).
- [ ] A `TenantLookup` test double or DB-backed lookup for `JwtClaimResolver`.

*If the app crate has no existing integration-test scaffold for routed handlers + middleware, Wave 0 installs it; otherwise extend the existing harness.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Dogfood GO/NO-GO end-to-end | AMCP-10/11, SC-4 | Requires a human browser login against a live, user-run server; not automatable unattended | 1. User starts the sample app with `APP_URL`, OAuth signing secret, and the two-tenant seed applied. 2. Run the checked-in scripted MCP client → discovery → DCR → `/authorize` (human logs in) → `/token` → `tools/list` → `tools/call` for `order`. 3. Confirm returned rows belong only to the authenticated tenant. 4. Repeat with Claude Desktop as the human-facing client. 5. Record GO/NO-GO in `200-ACCEPTANCE.md`; NO-GO blocks completion and triggers design revision. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (dogfood is the sole documented manual exception)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (two-tenant fixture + orders/tenants migrations)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s for quick crate tests
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
