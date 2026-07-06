---
phase: 243
slug: app-integration-e2e-envelope-guard-catalog-docs
status: secured
threats_open: 0
threats_closed: 6
asvs_level: 1
created: 2026-06-24
---

# SECURITY — Phase 243: App Integration, E2E, Envelope Guard & Catalog/Docs

**Phase:** 243 — App integration, e2e, envelope guard & catalog/docs
**Milestone:** v16.3 MCP CRUD Data Surface (Track A)
**ASVS Level:** default
**Audited:** 2026-06-24
**Threats Closed:** 6/6
**Result:** SECURED

---

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| P01-T-243-01 | E (Elevation) | mitigate | CLOSED | `app/src/projections/order.rs:15-18` — all three write flags paired with `.mcp_write_ability("manage-orders")`; `view-orders` read ability and `tenant_column("tenant_id")` retained at lines 13-14. Boot-validate test at lines 62-70 proves `validate()` accepts the combination and would reject write-flags-without-ability (CRUD-07). No write verb added via `.action(...)`. |
| P01-T-243-02 | T (Tampering) | mitigate | CLOSED | `app/src/projections/order.rs:26-41` — `StateMachine` declaration present; `tenant_column("tenant_id")` at line 13. `derive_crud_plan` (called in `ferro-mcp-server/src/write_dispatch.rs:236`) reads both to exclude `status`/`tenant_id`/`id`/`created_at` from write inputs. No CRUD verb name appears in any `.action(...)` call (confirmed: the only `.action(...)` calls at lines 43-52 are `submit`, `approve`, `ship`). `crud_cycle_create_list_update_delete` in `app/src/tests/crud_e2e.rs:445-453` asserts `status="draft"` is set server-side and `tenant_id` equals the context value, not an agent-supplied value. |
| P02-T-243-01 | E/S (Elevation/Spoofing) | mitigate | CLOSED | Three sub-proofs all present in `app/src/tests/crud_e2e.rs`: (a) auth gate — `crud_write_requires_write_authorization` (lines 510-536) calls with `write_authorized: None` and asserts `error.code == -32603` with "write ability denied"; gate implementation at `ferro-mcp-server/src/write_dispatch.rs:157-164`. (b) cross-tenant non-disclosure — `crud_cross_tenant_non_disclosure` (lines 549-582) asserts `isError==true` and `sc["result"].as_object().is_none()` plus confirms the foreign row is unmutated. (c) write-ability gating — `crud_verb_opted_in` at `write_dispatch.rs:205-210` gates each prefix on the matching boolean flag before routing. |
| P02-T-243-02 | I (Information Disclosure) / test-only bypass | mitigate | CLOSED | `write_authorized: Some(true)` appears only in `call_crud_tool` helper inside the `#[cfg(test)] mod tests` block in `app/src/tests/crud_e2e.rs:264-266`. The unauthorized variant `call_crud_tool_unauthorized` explicitly uses `write_authorized: None` (line 332) and is the vehicle for the auth-gate proof test. No production code path sets `write_authorized`. The `crud_write_requires_write_authorization` test (lines 509-536) proves the `None` path returns `-32603`, confirming the gate is real and not masked by the test's authorized default. |
| P03-T-243-01 | I (Misconfiguration) | mitigate | CLOSED | `ferro-mcp/src/tools/code_templates.rs:1635-1683` — `projection_crud_templates()` returns a template with `mcp_write_ability`, `.creatable(true)`, `.updatable(true)`, `.deletable(true)`, `.soft_delete_column("deleted_at")` matching `order.rs` exactly; test guard at line 1730 asserts `categories.contains("projection_crud")`. `ferro-mcp/src/tools/generation_context.rs:80-101` — `crud_handler` field extended with Option B documenting the opt-in prerequisites (tenant_column, deleted_at, mcp_write_ability, StateMachine status exclusion). `docs/src/features/projections.md:644-701` — `## MCP CRUD Opt-In` section present with prerequisites table (including CRUD-07 validate() note), derived tool set, authorization note (`read_write` scope + `write_authorized: Some(true)`), confirmation flow (`request_confirm_delete_<svc>` / `confirm_delete_<svc>`), and D-09 separation note. Builder chain in docs uses `"view-orders"`/`"manage-orders"` matching the shipped projection. |
| P03-T-243-02 | T (Tampering) | mitigate | CLOSED | D-10: `ferro-json-ui/src/catalog.rs:1101` asserts `BUILTIN_TYPES.len() == 47` (unchanged); `ferro-mcp/src/tools/json_ui_catalog.rs:292-297` asserts `catalog.components.len() == 47` (unchanged). D-09: `git diff --quiet ferro-mcp/src/tools/crud_operations.rs` exits 0 — file is byte-for-byte unchanged. |

---

## Unregistered Flags

None. Threat flags from all three SUMMARY.md files map cleanly to registered threats:

- 243-01-SUMMARY.md flags: "CRUD tool surface widening gated by mcp_write_ability" → P01-T-243-01; "status/tenant_id exclusion upheld by derive_crud_plan" → P01-T-243-02.
- 243-02-SUMMARY.md flags: "auth gate test and cross-tenant test prove production gates are real" → P02-T-243-01 and P02-T-243-02. "No new network endpoints or auth paths; all test code cfg(test)-gated" → informational, no unregistered surface.
- 243-03-SUMMARY.md flags: "Documentation only; no new network endpoints, auth paths, or schema changes" → informational.

---

## Accepted Risks Log

None. All threats are dispositioned `mitigate` and all are closed.

---

## Notes

**WR-01 (from REVIEW.md):** The `dispatch_write` doc example in `docs/src/features/projections.md` previously omitted the trailing `crud_plan` argument. The REVIEW.md finding records this was fixed in commit `8a4b98e6` (line 606: `None, // crud_plan: None on the transition path`). Confirmed present in the file — not an open gap.

**IN-01 (from REVIEW.md):** The `order.rs:18` comment states delete is "confirmation-gated" without the feature qualifier. Behavior is correct and both feature paths are tested. Comment imprecision only; not a security gap.

**Human UAT outstanding:** A live `:8090/mcp` CRUD drive with a seeded `read_write` bearer key is declared as `human_needed` in 243-VERIFICATION.md. This exercises the HTTP transport layer and bearer-key authentication, which cannot be fully automated in CI. It is confirmation-of-integration, not discovery — the in-process harness already exercises the same `McpContext` authorization path and the same `execute_crud_plan` kernel. This is not a security gap; it is a UAT smoke item.
