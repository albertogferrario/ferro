---
status: complete
phase: 243-app-integration-e2e-envelope-guard-catalog-docs
source: [243-VERIFICATION.md]
started: 2026-06-24T10:59:14Z
updated: 2026-06-24T14:55:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Live `:8090/mcp` create→list→update→delete drive
expected: With the app's `order` projection flipped to CRUD and a seeded `read_write`
bearer key, an agent drives create → list → update → delete through the live
`:8090/mcp` endpoint, each verb returning a well-formed Phase 205
`CallToolResult::structured` envelope, and `delete_order` enforcing the
`request_confirm_delete_order` / `confirm_delete_order` token flow. Per 243-CONTEXT.md
D-01/D-02 this was intentionally designated a manual UAT smoke (not a CI gate); the
in-process `crud_e2e.rs` harness already exercises the same shared kernel and
authorization path automatically, so this live drive is confirmation, not the
primary gate.
result: pass
notes: |
  Driven live against the running `app` server on `:8090` (10-connection pool, file-backed
  SQLite) with a minted HS256 JWT (sub=901/alice, tenant_id=1/acme, aud={APP_URL}/mcp). The
  first drive surfaced TWO real defects that the in-process harness had masked; both were
  fixed and the full cycle then passed end-to-end:
    1. create_order  → ok envelope, new id=5, status="draft" (server-side), tenant_id=1 (injected)
    2. list_order    → [1, 2, 5] (acme only — tenant isolation holds)
    3. update_order  → ok envelope, customer_name persisted (no status field)
    4. delete_order  → confirmation_required, request_tool="request_confirm_delete_order", isError=true
    5. request_confirm_delete_order → cfm_-prefixed token (+ expires_in_seconds)
    6. confirm_delete_order (token) → ok envelope (soft-deleted)
    7. list_order    → [1, 2] (soft-deleted row excluded)
  audit_log recorded mcp.crud.create_order / update_order / delete_order. Every write returned
  the Phase 205 structured envelope (content[0].type==text, structuredContent.status==ok,
  action==tool_name, result object, isError!=true).

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

<!-- Both gaps below were found by the live drive and FIXED in this session; the live cycle
     now passes end-to-end. Retained as resolved records. -->

- truth: "An agent drives create → update → delete through the live :8090/mcp endpoint, each write returning a well-formed Phase 205 ok envelope"
  status: resolved
  reason: "Live drive #1: create_order → -32603 'authorization: write ability denied'; all CRUD writes denied while reads succeeded."
  severity: major
  test: 1
  root_cause: "app/src/projections/order.rs declares .mcp_write_ability('manage-orders') (Phase 243-01), but app/src/bootstrap.rs only defined the 'view-orders' Gate ability — 'manage-orders' was never Gate::define'd. The /mcp controller computes write_authorized via Gate::authorize_for(&user, 'manage-orders', None); an undefined ability returns AuthResponse::deny_silent() (framework/src/authorization/gate.rs — 'no matching ability found' → deny), so write_authorized = Some(false) and handle_write_call rejected every CRUD verb with -32603. The in-process crud_e2e.rs harness masked this by constructing McpContext { write_authorized: Some(true), .. } directly."
  fix: "Added Gate::define('manage-orders', ...) to app/src/bootstrap.rs, mirroring the existing 'view-orders' definition (any authenticated User → allow; tenant scoping enforced by dispatch)."
  artifacts:
    - path: "app/src/bootstrap.rs"
      issue: "Gate::define('manage-orders', ...) was missing — FIXED"
  debug_session: ""

- truth: "A live CRUD create on a pooled, file-backed SQLite connection returns the inserted row including its auto-generated id"
  status: resolved
  reason: "Live drive #2 (after gap-1 fix): create_order persisted the row but returned execution_error 'write operation failed'. Instrumentation showed last_insert_rowid()=0, so the post-INSERT SELECT (WHERE id=0) found no row."
  severity: major
  test: 1
  root_cause: "framework/src/write/mod.rs execute_crud_plan() used a SQLite-specific two-step create: INSERT, then a SEPARATE 'SELECT last_insert_rowid()', then SELECT *. On a pooled DatabaseConnection (default DB_MAX_CONNECTIONS=10) the INSERT and the last_insert_rowid() query may run on DIFFERENT physical connections; last_insert_rowid() is per-connection in SQLite, so it returned 0 and the follow-up SELECT WHERE id=0 found nothing → every CRUD create failed on a real pool. The in-process test masked this because in-memory SQLite uses a single connection. Postgres was unaffected (it already used INSERT … RETURNING *)."
  fix: "Unified both backends to 'INSERT … RETURNING *' (a single round-trip on the SAME connection; SQLite 3.35+ supports RETURNING). Removed the last_insert_rowid() step and the now-unnecessary post-INSERT tenant predicate (RETURNING yields exactly the inserted row). Verified: framework write-kernel tests (496) + app crud_e2e (3, confirmation on) pass; fmt + clippy --all-targets clean."
  artifacts:
    - path: "framework/src/write/mod.rs"
      issue: "execute_crud_plan create relied on cross-connection last_insert_rowid() — FIXED (INSERT … RETURNING *)"
  debug_session: ""
