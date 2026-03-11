---
phase: 95
slug: multi-tenant-middleware
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 95 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `tokio::test` |
| **Config file** | none (workspace-level) |
| **Quick run command** | `cargo test -p ferro-rs --lib tenant` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-rs --lib tenant`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 95-01-01 | 01 | 1 | MT-01 | unit | `cargo test -p ferro-rs --lib tenant::resolver::tests::subdomain` | ❌ W0 | ⬜ pending |
| 95-01-02 | 01 | 1 | MT-02 | unit | `cargo test -p ferro-rs --lib tenant::resolver::tests::header` | ❌ W0 | ⬜ pending |
| 95-01-03 | 01 | 1 | MT-03 | unit | `cargo test -p ferro-rs --lib tenant::resolver::tests::path` | ❌ W0 | ⬜ pending |
| 95-01-04 | 01 | 1 | MT-04 | unit | `cargo test -p ferro-rs --lib tenant::middleware::tests` | ❌ W0 | ⬜ pending |
| 95-01-05 | 01 | 1 | MT-05 | unit | `cargo test -p ferro-rs --lib tenant::context::tests::outside_scope` | ❌ W0 | ⬜ pending |
| 95-01-06 | 01 | 1 | MT-06 | unit | `cargo test -p ferro-rs --lib tenant::scope::tests` | ❌ W0 | ⬜ pending |
| 95-01-07 | 01 | 1 | MT-07 | unit | `cargo test -p ferro-rs --lib tenant::tests::from_request` | ❌ W0 | ⬜ pending |
| 95-01-08 | 01 | 1 | MT-08 | unit | `cargo test -p ferro-rs --lib tenant::middleware::tests::not_found` | ❌ W0 | ⬜ pending |
| 95-01-09 | 01 | 1 | MT-09 | unit | `cargo test -p ferro-rs --lib tenant::lookup::tests::caching` | ❌ W0 | ⬜ pending |
| 95-01-10 | 01 | 1 | MT-10 | integration | `cargo test -p ferro-rs --test tenant_isolation` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `framework/src/tenant/mod.rs` — module stub with public re-exports
- [ ] `framework/src/tenant/context.rs` — task_local + current_tenant() stub
- [ ] `framework/src/tenant/resolver.rs` — TenantResolver trait stub
- [ ] `framework/src/tenant/middleware.rs` — TenantMiddleware stub
- [ ] `framework/src/tenant/scope.rs` — TenantScope stub
- [ ] `framework/tests/tenant_isolation.rs` — integration test placeholder

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Concurrent request isolation under load | MT-10 | Requires multi-threaded tokio runtime with real concurrent requests | Run `cargo test -p ferro-rs --test tenant_isolation` with `#[tokio::test(flavor = "multi_thread")]` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
