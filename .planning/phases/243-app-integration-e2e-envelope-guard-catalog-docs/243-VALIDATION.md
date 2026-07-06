---
phase: 243
slug: app-integration-e2e-envelope-guard-catalog-docs
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-24
validated: 2026-06-24
---

# Phase 243 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust; `#[tokio::test]` async; in-process MCP e2e via in-memory SQLite + `Migrator::up`) |
| **Config file** | none — existing app test harness (`app/src/tests/`) |
| **Quick run command** | `cargo test -p app <new_e2e_filter>` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~120–300s (full); ~30s (app e2e) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p app <touched_filter>` (or `-p ferro-mcp` for catalog/docs tasks)
- **After every plan wave:** `cargo test -p app` for the e2e plan
- **Before `/gsd-verify-work`:** full suite (fmt + clippy + test) green
- **Max feedback latency:** ~300s

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------------|-----------|-------------------|-------------|--------|
| 243-01-* | 01 | 1 | CRUD-01/05/07 | order projection flips to CRUD; validate() passes at boot (mcp_write_ability present) | unit/boot | `cargo test -p app order` → `order_projection_validates_after_crud_flip` | ✅ | ✅ green |
| 243-02-* | 02 | 2 | CRUD-01/02/03/06 | create→list→update→delete e2e through MCP with read_write McpContext (write_authorized: Some(true)) | app in-process e2e | `cargo test -p app crud_e2e` → `crud_cycle_create_list_update_delete` | ✅ | ✅ green |
| 243-02-* | 02 | 2 | CRUD-06 | same CrudPlan succeeds on visual surface (MCP↔visual parity, shared kernel) | parity | `cargo test -p app crud_e2e` → `crud_mcp_visual_single_source_parity` | ✅ | ✅ green |
| 243-02-* | 02 | 2 | CRUD-01/02/03 | each create/update/delete result is a well-formed Phase 205 content[] envelope + write-auth gate (`crud_write_requires_write_authorization`) + cross-tenant non-disclosure (`crud_cross_tenant_non_disclosure`) | envelope/auth/tenant guard | `cargo test -p app crud_e2e` | ✅ | ✅ green |
| 243-02-* | 02 | 2 | CRUD-03 | delete without token → confirmation_required (+request_tool); with token → soft-delete, gone from list_ | confirm e2e (feature-gated) | `cargo test -p app --features confirmation crud_e2e` → `delete_order_confirmation_flow` | ✅ | ✅ green |
| 243-03-* | 03 | 3 | CRUD-01..07 (docs) | ferro-mcp code_templates + generation_context + docs/src document the CRUD opt-in + derived tools; component drift-guards unchanged at 47 | doc-accuracy | `cargo test -p ferro-mcp code_templates` → `test_all_categories_present` (asserts `projection_crud`) | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The in-process MCP e2e harness
(`app/src/tests/mcp_write_dispatch.rs`: `setup_db()` + `seed_two_tenants()` + `handle_tools_call`),
the envelope-assertion pattern (`mcp_tenant_isolation.rs`), and the MCP↔visual parity pattern
(`single_source.rs`) all already exist. No new framework or test scaffolding to install.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live `:8090/mcp` create→list→update→delete with a seeded `read_write` bearer key | CRUD-05 (SC#1) | Live HTTP server + bearer auth is out of the CI harness scope (D-02); the in-process harness exercises the same kernel + scope path | Boot the app on `:8090`, seed a `read_write` API key, drive the cycle via the reusable `:8090` + chrome-mcp harness; record as HUMAN-UAT |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 300s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** validated 2026-06-24 — all 6 mapped requirements COVERED by existing green automated tests; the single live-`:8090/mcp` item remains Manual-Only by design (D-02).

---

## Validation Audit 2026-06-24

| Metric | Count |
|--------|-------|
| Requirements audited | 6 |
| COVERED (green automated) | 6 |
| PARTIAL | 0 |
| MISSING (gaps filled) | 0 |
| Manual-only (by design) | 1 (live `:8090/mcp` drive — CRUD-05 SC#1) |

All mapped test functions were cross-referenced against the source tree and exist:
`order_projection_validates_after_crud_flip` (order.rs), `crud_cycle_create_list_update_delete` /
`crud_write_requires_write_authorization` / `crud_cross_tenant_non_disclosure` /
`crud_mcp_visual_single_source_parity` / `delete_order_confirmation_flow` (crud_e2e.rs), and
`test_all_categories_present` asserting the `projection_crud` category (code_templates.rs). The full
`cargo test --all-features` gate ran green at phase execution; no new tests needed (no auditor spawn).
