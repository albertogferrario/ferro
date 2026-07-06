---
phase: 242-write-authorization-tenant-injection-non-disclosure
fixed_at: 2026-06-24T00:00:00Z
review_path: .planning/phases/242-write-authorization-tenant-injection-non-disclosure/242-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 3
skipped: 0
accepted_no_change: 2
status: all_fixed
---

# Phase 242: Code Review Fix Report

**Fixed at:** 2026-06-24
**Source review:** `.planning/phases/242-write-authorization-tenant-injection-non-disclosure/242-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed (code change + commit): 3 (CR-01, CR-02, WR-02)
- Accepted / no code change: 2 (WR-01, IN-01)
- Skipped: 0

**Gate result:** `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings` + `cargo test --all-features` all passed clean. Schema export side-effect (`docs/protocol/schemas/*.json`) was restored via `git checkout` and not committed.

---

## Fixed Issues

### CR-01: Transition-action write tools always denied — write-ability gate scope

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`, `app/src/controllers/mcp.rs`
**Commit:** `db230d41`
**Applied fix:**

Two coordinated changes:

**Channel (`write_dispatch.rs`):** Replaced the unconditional `if ctx.write_authorized != Some(true)` check with an `is_crud_write_tool` detector that strips optional `request_confirm_`/`confirm_` prefixes, then checks whether the remainder starts with exactly one of `create_`/`update_`/`delete_` and resolves to an mcp-exposed service with the matching CRUD flag set. The write-ability gate fires only when `is_crud_write_tool && ctx.write_authorized != Some(true)`. Transition-action tools (e.g. `"submit"`, `"approve"`) have no CRUD prefix so `is_crud_write_tool` is false and the gate is skipped — they proceed directly to `find_action` / `dispatch_write` with their existing scope + guard-re-eval authorization.

Added IN-01 clarifying comment at the gate site documenting that the `-32603` transport-level shape is intentional (consistent with all other authz denials) rather than an oversight.

**Host (`app/src/controllers/mcp.rs`):** Replaced the chained `trim_start_matches("create_").trim_start_matches("update_").trim_start_matches("delete_")` chain with a `find_map` over `["create_", "update_", "delete_"]` using a single `strip_prefix` per verb, guarded by the service's matching CRUD flag. Returns:
- `Some(true/false)` — CRUD verb tool (Gate result)
- `None` — read tool or transition-action tool (not gated by write-ability)

The semantic change is: transition-action tools now get `None` instead of `Some(false)`, which the channel interprets as "skip gate" rather than "deny".

**Regression test added (`write_dispatch.rs`):** `transition_action_not_denied_by_write_ability_gate` — calls `handle_write_call` with tool name `"submit"` (a transition-action, no CRUD prefix), `write_authorized: None`, and a service that has no CRUD flags. Asserts the response does NOT contain `"write ability denied"`. The existing `write_authorized_none_denies` and `write_authorized_false_denies` tests (which use `create_order` / `update_order` — genuine CRUD tools) continue to pass unchanged.

---

### CR-02: Post-INSERT SELECT not scoped to tenant

**Files modified:** `framework/src/write/mod.rs`
**Commit:** `63e5a4aa`
**Applied fix:**

Replaced the bare `SELECT * FROM {table} WHERE id = ?` post-INSERT SELECT in the SQLite arm of `execute_crud_plan` with a conditional:
- When `tenant_column` is `Some`: `SELECT * FROM {table} WHERE id = ? AND {tc_col} = {t_ph}` with two bound values — `inserted_id` and `tenant_id`. Uses `placeholder(backend, 2)` for the tenant placeholder to mirror the existing post-UPDATE SELECT pattern.
- When `tenant_column` is `None`: unchanged single-predicate query.

This mirrors the Phase 242 Pitfall 5 fix already applied to the post-UPDATE SELECT (lines 443-460). The Postgres arm uses `INSERT … RETURNING *` (single round-trip, no separate SELECT) so it was not affected.

---

### WR-02: `validate_rejects_*` tests missing message-content assertions

**Files modified:** `ferro-projections/src/service.rs`
**Commit:** `4a57149a`
**Applied fix:**

The three older per-verb tests (`validate_rejects_creatable_without_write_ability`, `validate_rejects_updatable_without_write_ability`, `validate_rejects_deletable_without_write_ability`) previously only asserted the error variant via `matches!`. Each now also calls `.unwrap_err()` and asserts `err.to_string().contains("mcp_write_ability")`, symmetric with the consolidated `validate_rejects_crud_verb_without_write_ability` test at line 2307. A rename of the field in the error message will now be caught uniformly across all four tests.

---

## Accepted / No Code Change

### WR-01: Denial audit not written for CRUD-verb guard failures

**File:** `ferro-mcp-server/src/write_dispatch.rs:336-349`
**Disposition:** ACCEPTED/DEFERRED
**Rationale:** CRUD verbs have no action-level guards today (the comment at line 436 explicitly marks the CRUD delete pre-check loop as a Phase 242 extension point with an empty `crud_guards` vec). There is no live denial audit gap to close. When CRUD guard support lands, the denial audit should be extracted into a shared helper and called from both the action path and the CRUD path at that time — the gap should not be papered over pre-emptively with dead code.

### IN-01: write_authorized denial envelope shape vs. tool-error result shape

**File:** `ferro-mcp-server/src/write_dispatch.rs:124-131`, `app/src/controllers/mcp.rs:166-178`
**Disposition:** ACCEPTED (by design)
**Rationale:** Authorization denials (scope gate in `jsonrpc.rs`, tenant fail-closed, write-ability gate) are uniformly transport-level `-32603` errors. Execution-level outcomes (guard, validation, not-found) use the `write_tool_error_result` shape. The distinction allows clients to separate auth failures from application errors without parsing the body. A clarifying comment was added at the gate site in `write_dispatch.rs` (included in the CR-01 commit) documenting this intent explicitly.

---

_Fixed: 2026-06-24_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
