---
phase: 218-write-tool-rendering-from-actiondef
audited: 2026-06-13
asvs_level: 1
status: SECURED
threats_total: 3
threats_closed: 3
threats_open: 0
block_on: high
---

# Phase 218 Security Audit

**Phase:** 218 — Write-Tool Rendering from ActionDef
**ASVS Level:** 1
**Auditor:** gsd-security-auditor
**Date:** 2026-06-13

## Threat Verification

| Threat ID | Category | Disposition | Status | Evidence |
|-----------|----------|-------------|--------|----------|
| T-218-01 | Information Disclosure | mitigate | CLOSED | `ferro-mcp-server/src/schema.rs:140-141` — `if matches!(input.meaning, FieldMeaning::Sensitive) { continue; }` in `build_action_input_schema` skips the input entirely; it is absent from both `properties` and `required[]`. Tested by `test_action_schema_excludes_sensitive_input` (schema.rs:351). Identifier injection path (lines 119-136) uses only `FieldMeaning::Identifier`-tagged fields, never Sensitive. |
| T-218-02 | Elevation of Privilege (visibility misread as auth) | mitigate | CLOSED | `ferro-mcp-server/src/renderer.rs:125-127` — `render_action_tool` carries an explicit doc comment: "This guard check is a VISIBILITY filter, NOT an authorization gate — a hidden tool is simply not listed, not 'uncallable'. Server-side guard enforcement is Phase 219; the 217 scope gate is the read/write boundary. Do not treat this as the security boundary." No authorization decision is made. Six tests (renderer.rs:322-453) pin presence/absence semantics only, never "uncallable". |
| T-218-03 | Tampering (malformed def breaks strict MCP clients) | mitigate | CLOSED | `ferro-mcp-server/src/jsonrpc.rs:289-351` — `write_tools_definitions_parse_as_valid_mcp_tool` test deserialized all three tool definitions via `serde_json::from_value::<rmcp::model::Tool>(...)` (line 312); asserts non-empty names, inputSchema has `"type"` or `"properties"` keys, and annotation values are exact. Reported GREEN in 218-02-SUMMARY.md. |

## No-Dispatch Verification

Write-tool `tools/call` correctly returns `-32601 Method not found` in Phase 218. The dispatch path in `handle_tools_call` (jsonrpc.rs:66) uses `tool_name.strip_prefix("list_").unwrap_or(tool_name)`, which for a write-tool name such as `submit_order` produces `submit_order` — a string that matches no `ServiceDef.name`, causing the service lookup at line 84-92 to fall through to `-32601`. An explicit Phase 219 routing comment at lines 63-65 documents this as intended. No write executor exists in this phase.

## WR-01 Security Impact Assessment

REVIEW.md WR-01 notes that `disambiguate_write_tool_collisions` (renderer.rs:102-119) counts total tool-name occurrences across the tagged Vec rather than distinct services. Under the misfiring condition — a single service declaring two `ActionDef`s with identical names — both tools would be renamed to `<name>_on_<service>` rather than left bare. This is a naming correctness issue: the collision rename fires unnecessarily and produces two tools with identical post-rename names (both `<name>_on_<same_service>`). The consequence is a confusing `tools/list` output, not a security event. The issue touches neither the Sensitive-exclusion path (T-218-01), nor the guard-filter visibility semantics (T-218-02), nor the strict-deserialization gate (T-218-03). Security impact: **none**. The `ActionDef` builder API does not prevent duplicate names, so the condition is theoretically reachable by a misconfigured `ServiceDef`, but the worst outcome is tool-name confusion visible only in `tools/list` output — no data exposure, no authorization bypass, no schema injection.

## Unregistered Threat Flags

SUMMARY.md `## Threat Flags` section: "None. No new network endpoints, auth paths, or schema changes at trust boundaries introduced by this plan." No unregistered flags to record.

## Accepted Risk Log

None. All threats carry disposition `mitigate` and are verified closed.

## Notes

- The scope gate (`ctx.scope == "read"` check at jsonrpc.rs:71-82) that rejects write-tool calls for read-scoped keys was introduced in Phase 217 and is already active in this phase; Phase 218 does not weaken it. The gate fires before the service lookup, so it applies even to write-tool names that would produce `-32601` anyway.
- IN-01 (REVIEW): The scope-rejection error code is `-32603` rather than a more semantically accurate code. This is a client-usability issue, not a security gap — the error still rejects the call. Recorded here for completeness; classification is informational, not a threat.
