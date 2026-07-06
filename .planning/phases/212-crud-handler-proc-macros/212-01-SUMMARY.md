---
phase: 212-crud-handler-proc-macros
plan: "01"
subsystem: framework/validation + framework/tenant
tags: [validation, tenant, crud, foundation]
dependency_graph:
  requires: []
  provides: [validate_or_redirect, TenantScoped]
  affects: [framework/src/validation/validator.rs, framework/src/tenant/scoped.rs, framework/src/lib.rs]
tech_stack:
  added: []
  patterns: [async_trait, validate-or-redirect chain, tenant-scoped lookup contract]
key_files:
  created:
    - framework/src/tenant/scoped.rs
  modified:
    - framework/src/validation/validator.rs
    - framework/src/tenant/mod.rs
    - framework/src/lib.rs
decisions:
  - "TenantScoped lives in framework/src/tenant/ (not ferro-orm) — lookup contract is tied to the tenant layer, no new crate dep"
  - "async_trait used (not AFIT) — consistent with every async trait in framework/src/"
  - "validate_or_redirect doclink uses full path crate::http::action::ActionError to satisfy cargo doc -Dwarnings gate"
metrics:
  duration: "~5 minutes"
  completed: "2026-06-13"
requirements: [CRUD-03, CRUD-04]
---

# Phase 212 Plan 01: Framework Foundations Summary

**One-liner:** `Validator::validate_or_redirect` composing `with_old_input + into_action_error`, plus `TenantScoped` async trait enforcing tenant-scoped lookup by signature.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Validator::validate_or_redirect (CRUD-03) | fd72efad | framework/src/validation/validator.rs |
| 2 | Add TenantScoped trait + facade re-export (CRUD-04) | 4c643bfe, cd02e5f2 | framework/src/tenant/scoped.rs, framework/src/tenant/mod.rs, framework/src/lib.rs |

## Verification

- `cargo test -p ferro-rs validation::validator` — 11 passed (3 new: pass / fail / old-input chain)
- `cargo build -p ferro-rs` — clean
- `cargo fmt --all -- --check` — clean
- `cargo clippy -p ferro-rs --all-targets -- -D warnings` — clean
- `cargo doc -p ferro-rs --no-deps` — no warnings

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustfmt signature collapse on TenantScoped**
- **Found during:** Task 2 fmt check
- **Issue:** Multi-line `find_for_tenant` signature in scoped.rs was reformatted by rustfmt to a single line (under 100-char limit)
- **Fix:** Applied rustfmt format, committed as style(212-01) fixup
- **Files modified:** framework/src/tenant/scoped.rs
- **Commit:** cd02e5f2

**2. [Rule 2 - Missing] rustdoc link path for ActionError**
- **Found during:** Task 1 `cargo doc` check
- **Issue:** `[ActionError]` bare link not resolvable from validator.rs scope; `cargo doc -Dwarnings` gate would fail
- **Fix:** Changed to `[crate::http::action::ActionError]` full path
- **Files modified:** framework/src/validation/validator.rs
- **Commit:** included in fd72efad (pre-commit)

## Known Stubs

None. Both additions are complete — `validate_or_redirect` is fully functional, `TenantScoped` is a trait contract (no stub implementations expected here; concrete impls live in consumer models).

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: tenant_isolation_contract | framework/src/tenant/scoped.rs | New trust boundary: `find_for_tenant(id, tenant_id)` is the seam where IDOR prevention is enforced. Rustdoc explicitly instructs implementers to include `AND tenant_id = ?`. T-212-01 mitigation in place. |

## Self-Check: PASSED

- framework/src/validation/validator.rs — FOUND
- framework/src/tenant/scoped.rs — FOUND
- commit fd72efad — FOUND
- commit 4c643bfe — FOUND
- commit cd02e5f2 — FOUND
