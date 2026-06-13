---
phase: 219-write-dispatch
fixed_at: 2026-06-14T00:00:00Z
review_path: .planning/phases/219-write-dispatch/219-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 219: Code Review Fix Report

**Fixed at:** 2026-06-14T00:00:00Z
**Source review:** `.planning/phases/219-write-dispatch/219-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (CR-01, CR-02, WR-01, WR-02, WR-03)
- Fixed: 5
- Skipped: 0

**Verification:** All tests run after each fix; final state: `cargo test -p ferro-mcp-server` (48 tests, 0 failed), `cargo test -p app` (19 tests, 0 failed), `cargo clippy -p ferro-mcp-server -p app --all-targets -- -D warnings` (clean).

---

## Fixed Issues

### CR-01: Execution errors leak internal detail to agent response

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** `0daa9b1a`
**Applied fix:** Replaced the single catch-all `Err(e)` arm in `handle_write_call` with two explicit arms. `Err(ref e @ crate::Error::Validation(_))` and `Err(ref e @ crate::Error::ActionNotFound(_))` pass their messages through (these contain only caller-supplied or intent-defined strings with no internal DB state). All other variants (`Database`, `Serialization`, `Auth`, `Render`, `InvalidFilter`, and any future additions) are caught by `Err(_)` and return the generic string `"write operation failed"` to the agent. The `GuardFailed` arm was already separate above and is unaffected. No logging facility was introduced (none existed in the crate).

### CR-02: Unknown guard names are silently allowed (fail-open for future guards)

**Files modified:** `app/src/controllers/mcp.rs`, `app/src/tests/mcp_write_dispatch.rs`
**Commit:** `127c3ada`
**Applied fix:** Changed `_ => Ok(true)` to `_ => Err(ferro_mcp_server::Error::GuardFailed(format!("unknown guard '{guard_name}': no evaluator registered")))` in both the production `make_write_dispatcher` guard evaluator and the test `make_test_write_dispatcher`. All existing tests were verified GREEN after this change: the only guard name used in any test that goes through these dispatchers is `"is_manager"`, which remains explicitly matched. The `idempotent_write_e2e` test uses its own inline `|_, _, _, _| Ok(true)` evaluator (not `make_test_write_dispatcher`) and is unaffected. SC#2, SC#3, SC#4 all stayed GREEN.

**Note:** This fix is security-critical. The fail-closed invariant is now established as a pattern: any new `ActionDef` with a guard name not registered in the evaluator will be denied, not silently passed. This requires future authors to explicitly add each guard name to the `match` before the action can execute.

### WR-01: `idempotency_key` extracted from inputs but not declared in ActionDef, bypasses validation

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** `2f3784f1`
**Applied fix:** Added an explicit length check in `dispatch_write` before the idempotency lookup: if `inputs["idempotency_key"]` is present and its string length exceeds 128 characters, `dispatch_write` returns `Err(crate::Error::Validation("idempotency_key must not exceed 128 characters"))` immediately. This error surfaces through `handle_write_call`'s new `Validation` arm (CR-01 fix) as an agent-visible `execution_error` with the validation message. The cap is 128 characters, sufficient for any UUID, hash, or client-generated correlation id.

### WR-02: Audit `after` field stores raw executor result — may contain PII

**Files modified:** `ferro-mcp-server/src/write_dispatch.rs`
**Commit:** `2f3784f1`
**Applied fix:** Added a doc-comment block `# Audit contract` to the `ExecutorFn` type alias explicitly stating: the returned `Value` is stored verbatim in the append-only `audit_log` table and MUST NOT contain secrets, credentials, PII, or any field that should not appear in a forensic log. Executors are responsible for returning only audit-safe fields (typically identifiers and status values). A full PII scrub at the `dispatch_write` call site is not performed; the executor is the enforcement point. The current executor in `app/src/controllers/mcp.rs` already returns only `{"id": ..., "status": ...}`, which conforms to this contract.

### WR-03: Double service-lookup in controller adds Gate overhead for write-tool calls

**Files modified:** `app/src/controllers/mcp.rs`
**Commit:** `b3f4ff02`
**Applied fix:** Applied option (b) — explicit boundary documentation with structural enforcement. The `tools/call` branch was restructured so that:
- The Gate check (`service` lookup, `User::find_by_id`, `mcp_ability` check, `Gate::authorize_for`) runs only for tools whose name starts with `"list_"`, inside an `if let Some(service_name) = tool_name.strip_prefix("list_")` block.
- Write tools (all other names) skip the Gate block entirely and fall through to `handle_tools_call`, which enforces authorization via the scope gate (Phase 217) and `dispatch_write` guard re-evaluation (D-02 / SC#1).
- A 14-line comment block at the routing fork names the two authorization layers for each tool class and explains why Gate is not applied to write tools (it is service-oriented and does not map cleanly to action names; the scope gate + live guards are the complete write authorization surface).

Previously, write tools silently hit `None => 32601 Method Not Found` because `strip_prefix("list_")` on `"submit"` returns `"submit"`, which does not match the service name `"order"`. The fix makes the routing intentional and documented rather than a silent structural accident.

---

_Fixed: 2026-06-14T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
