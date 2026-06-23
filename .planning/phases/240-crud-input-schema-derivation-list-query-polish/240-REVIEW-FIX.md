---
phase: 240
slug: crud-input-schema-derivation-list-query-polish
source: 240-REVIEW.md
status: resolved
fixed: 4
deferred: 3
date: 2026-06-23
commit: 2b8eb6e6
---

# Phase 240 — Code Review Fix Summary

All 4 warnings from `240-REVIEW.md` resolved in commit `2b8eb6e6`. 3 info items
reviewed and deferred (rationale below). Gate green after fixes:
`cargo fmt --all -- --check` + `cargo clippy -p ferro-projections -p ferro-mcp-server
--all-targets -- -D warnings` + `cargo test` (56 ferro-mcp-server lib tests + 277
ferro-projections, 0 failures).

## Fixed

| ID | File | Fix |
|----|------|-----|
| WR-01 | `dispatch.rs` | Sort validation accepts range-filterable fields (`is_filter_field(f) \|\| is_range_filter_field(f)`). A field advertised with range ops (Money/Quantity/Percentage) is now also sortable — schema and dispatch agree. |
| WR-02 | `schema.rs` | Informative inputSchema property descriptions: create → `"Value for the {field} field"`, update → `"New value for the {field} field"`, delete identifier → `"ID of the {svc} record to delete"` (was raw field name / absent). |
| WR-03 | `schema.rs` | `build_update_input_schema` explicitly skips `FieldMeaning::Identifier` in the patch loop — robust against a future relaxation of `is_write_excluded_field` Gate A duplicating the `id` property. |
| WR-04 | `write_dispatch.rs` | NTI envelope gated on the matching opt-in flag (`creatable`/`updatable`/`deletable`). An unflagged service emits no such tool, so its call falls through to a genuine `-32601` instead of a misleading `not_yet_implemented`. New regression test `crud_nti_not_returned_when_verb_flag_disabled`. |

## Deferred (info items)

| ID | Rationale |
|----|-----------|
| IN-01 (`rand::thread_rng` breaks on rand 0.9) | Pre-existing, not introduced by Phase 240; workspace-wide dependency concern tracked separately. Out of this phase's scope. |
| IN-02 (`sort` schema lacks an `enum` constraint → allowlist/schema can drift) | Cosmetic; the dispatch-side allowlist is the enforced contract. An `enum` of sortable field names is a reasonable future polish (pairs with a Track D discovery/`generation_context` improvement), not a correctness issue. |
| IN-03 (`write_tool_error_result` implicit `message` key contract) | Internal helper contract; no caller currently violates it. Formalizing the envelope shape is better addressed when Phase 241 wires real CRUD execution through the same helper. |
