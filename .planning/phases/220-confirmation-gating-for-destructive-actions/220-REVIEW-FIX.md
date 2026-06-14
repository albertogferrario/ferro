---
phase: 220-confirmation-gating-for-destructive-actions
fixed_at: 2026-06-14T00:00:00Z
review_path: .planning/phases/220-confirmation-gating-for-destructive-actions/220-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 220: Code Review Fix Report

**Fixed at:** 2026-06-14
**Source review:** .planning/phases/220-confirmation-gating-for-destructive-actions/220-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: Internal error strings from `ConfirmationStore` leaked in `handle_request_confirm`

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** afd646d1
**Applied fix:** Replaced `format!("failed to store confirmation: {e}")` with the fixed string `"failed to store confirmation token"`. Changed `Err(e)` binding to `Err(_e)` (the variable was kept with underscore prefix to satisfy the compiler; the value is intentionally unused after redaction). Matches the 219 CR-01 redaction discipline applied to Database/Serialization/Auth variants.

### WR-02: Internal error strings from `ConfirmationStore` leaked in `handle_confirm`

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** afd646d1
**Applied fix:** Replaced `format!("confirmation store error: {e}")` with `"confirmation store error"` and changed `Err(e)` to `Err(_)`. No test assertions referenced the leaked message text; all confirmation tests assert on `error_kind` values and pass unchanged.

### WR-03: Guard error detail at confirm time leaks guard name and error string

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** afd646d1
**Applied fix:** Replaced `format!("precondition '{guard_name}' error at confirm time: {e}")` with `"precondition not met at confirm time"` and changed `Err(e)` to `Err(_)`. This is symmetric with the dispatch_write guard-error redaction at the handler boundary. The `if !passes` branch below (line 729) retains `format!("precondition '{guard_name}' not met at confirm time")` — that string contains no internal error detail, only the guard name which is app-defined and agent-visible by design.

### WR-04: TOCTOU window between token storage and response delivery (accept + document)

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** afd646d1
**Applied fix:** Added an inline comment at the `store.request_confirmation()` call site (lines 590–598) documenting: (a) the behavior — retried `request_confirm` calls mint a new token each time because the store is keyed on the token, not the `(tenant, action, record)` tuple; (b) why it is acceptable — each token is single-use, TTL-bounded (≤600 s), and bound to `(tenant_id, action_name, record_id)`, and `dispatch_write` idempotency prevents double-execution even on a race; (c) the hardening path — re-key on `(tenant, action, record)` when a persistent/DB-backed store replaces `InMemoryConfirmationStore`. No store keying was changed, as directed.

## Verification

**Clippy:** `cargo clippy -p ferro-mcp-server --features confirmation --all-targets -- -D warnings` — clean (0 warnings).

**Tests:** `cargo test -p ferro-mcp-server --features confirmation` — 40/40 passed (including all SC#1–SC#4, guard-at-confirm, expired-token, and two-step-flow confirmation tests).

---

_Fixed: 2026-06-14_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
