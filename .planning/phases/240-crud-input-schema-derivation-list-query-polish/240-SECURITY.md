---
phase: 240
slug: crud-input-schema-derivation-list-query-polish
asvs_level: 1
audit_date: 2026-06-23
status: SECURED
threats_open: 0
threats_total: 14
---

# Phase 240 Security Audit

## Summary

All 14 threats in the phase threat register are CLOSED. No unregistered flags were raised
by the executor (all four SUMMARY.md files report "Threat Flags: None"). The WR-01
review-fix broadened the sort allowlist from `is_filter_field` to
`is_filter_field || is_range_filter_field`; this remains an allowlist — non-matching
fields return `InvalidFilter` at dispatch line 160-162, introducing no field-existence
oracle beyond the existing filter surface.

## Threat Verification

| Threat ID | Category | Disposition | Evidence |
|-----------|----------|-------------|----------|
| T-240-01 | Information Disclosure | mitigate | `ferro-projections/src/service.rs:264` — Gate C: `matches!(field.meaning, FieldMeaning::Sensitive)` returns `true` unconditionally |
| T-240-02 | Tampering | mitigate | `ferro-projections/src/service.rs:256` — Gate A delegates to `self.is_server_injected_field(field)`; Identifier/CreatedAt/tenant tested by `is_write_excluded_field_gates` |
| T-240-03 | Elevation of Privilege | mitigate | `ferro-projections/src/service.rs:272` — Gate E: `exclude_sm_status && matches!(field.meaning, FieldMeaning::Status)` |
| T-240-04 | Information Disclosure | mitigate | `ferro-mcp-server/src/schema.rs:255,317` — both `build_create_input_schema` and `build_update_input_schema` call `service.is_write_excluded_field(field, exclude_sm_status)` |
| T-240-05 | Tampering | mitigate | `ferro-mcp-server/src/schema.rs:289-306,314` — Identifier injected as sole required field; patch loop skips Identifier explicitly (WR-03) and omits all data fields from `required_fields` |
| T-240-06 | Information Disclosure | accept | Schema advertises `__ne/__in/__gt/__gte/__lt/__lte/sort` params; enforcement is the dispatch allowlist (T-240-12). Advertising a rejected param produces a non-disclosing `InvalidFilter`, not a data leak. Documented in 240-02-PLAN.md threat register. |
| T-240-07 | Tampering | mitigate | `ferro-mcp-server/src/write_dispatch.rs:155-180` — CRUD verb NTI detection loop returns before `find_action` (line 192); no write path is reached |
| T-240-08 | Information Disclosure | mitigate | `ferro-mcp-server/src/write_dispatch.rs:173-176` — NTI payload is `{ error_kind, message }` only; no column names, table names, or schema internals |
| T-240-09 | Denial of Service (protocol) | mitigate | `ferro-mcp-server/src/jsonrpc.rs` — `crud_tool_call_nti_parses_as_valid_mcp_content` test asserts `CallToolResult::structured` (not -32601); original `tools_call_result_parses_as_valid_mcp_content` still passes; `crud_nti_not_returned_when_verb_flag_disabled` regression test (WR-04) added |
| T-240-10 | Tampering (SQLi via op suffix) | mitigate | `ferro-mcp-server/src/dispatch.rs:177-189` — exhaustive match maps op to fixed SQL constant (`>`,`>=`,`<`,`<=`,`!=`,`IN`); unknown op → `InvalidFilter` at line 185; no suffix string is interpolated |
| T-240-11 | Tampering (SQLi via __in elements) | mitigate | `ferro-mcp-server/src/dispatch.rs:221-228` — IN list built as `(?,?,...)` via `(0..arr.len()).map(|i| placeholder(backend, idx+i))`; each element bound via `json_to_sea_value(item)`; no element is string-interpolated |
| T-240-12 | Information Disclosure (filter/sort on excluded field) | mitigate | `ferro-mcp-server/src/dispatch.rs:192-205,247-254` — base field for gt/gte/lt/lte validated against `is_range_filter_field`; ne/in against `is_filter_field`; sort against `is_filter_field \|\| is_range_filter_field` (WR-01); all return same non-disclosing `InvalidFilter` on miss. No field-existence oracle. |
| T-240-13 | Tampering (predicate stripping) | mitigate | `ferro-mcp-server/src/dispatch.rs:262-278` (tenant), `ferro-mcp-server/src/dispatch.rs:280-291` (soft-delete IS NULL) — both blocks unchanged after the `__op`/sort extension; verified by REVIEW-FIX.md WR acceptance criterion |
| T-240-14 | Denial of Service (malformed IN) | accept→mitigate | `ferro-mcp-server/src/dispatch.rs:213-216` — empty `__in` array rejected with `InvalidFilter("'__in' array for '{base}' must not be empty")`; `IN ()` SQL is never emitted |

## Unregistered Flags

None. All four executor SUMMARY.md files (`240-01` through `240-04`) report no threat flags.

## Accepted Risks Log

| Threat ID | Acceptance Rationale |
|-----------|----------------------|
| T-240-06 | Schema advertising is non-load-bearing. Runtime enforcement (allowlist in dispatch) is the security boundary. An agent passing an advertised-but-rejected param receives a non-disclosing `InvalidFilter` error. Documented in 240-02-PLAN.md. |
| T-240-14 | Disposition escalated from `accept` to `mitigate` — empty `__in` is rejected at the application layer, not merely tolerated. Retained here for traceability. |

## Notes

- WR-01 (REVIEW-FIX): sort allowlist broadened to `is_filter_field(f) || is_range_filter_field(f)`. Both predicates gate on `readable`, non-list, non-Sensitive, non-Json/Binary. The union is still a strict allowlist; unknown or non-matching fields always receive `InvalidFilter` (dispatch.rs:160-162), never a schema-internal error.
- WR-04 (REVIEW-FIX): NTI envelope now gated on the matching opt-in flag (`creatable`/`updatable`/`deletable`). An unflagged service's CRUD-verb call falls through to the genuine `-32601` path, preventing a misleading NTI response from implying tool availability.
