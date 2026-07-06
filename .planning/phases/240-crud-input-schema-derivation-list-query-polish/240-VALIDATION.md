---
phase: 240
slug: crud-input-schema-derivation-list-query-polish
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-23
audited: 2026-06-23
---

# Phase 240 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` / `#[tokio::test]` (no external test runner) |
| **Config file** | `Cargo.toml` (workspace); `ferro-mcp-server` dev-deps include `tokio`, `async-trait` |
| **Quick run command** | `cargo test -p ferro-mcp-server -p ferro-projections` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30–90 seconds (quick); full workspace longer |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp-server -p ferro-projections`
- **After every plan wave:** Run `cargo test --all-features`
- **Before `/gsd-verify-work`:** Full gate green —
  `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Max feedback latency:** ~90 seconds (quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 240-01-01 | 01 | 1 | CRUD-01/02 | T-240-01/02/03 | `is_write_excluded_field` 5-gate correctness (server-injected, UpdatedAt, Sensitive, list, SM-Status) | unit | `cargo test -p ferro-projections is_write_excluded_field_gates` | ✅ | ✅ green |
| 240-02-01 | 02 | 2 | CRUD-04 | T-240-06 | `__gt/__gte/__lt/__lte` emitted for Integer/Float/DateTime/Date | unit | `cargo test -p ferro-mcp-server test_range_params_in_schema` | ✅ | ✅ green |
| 240-02-01 | 02 | 2 | CRUD-04 | T-240-06 | `__ne/__in` emitted for all `is_filter_field` fields | unit | `cargo test -p ferro-mcp-server test_ne_in_params_in_schema` | ✅ | ✅ green |
| 240-02-01 | 02 | 2 | CRUD-04 | T-240-06 | `sort` param emitted | unit | `cargo test -p ferro-mcp-server test_sort_param_in_schema` | ✅ | ✅ green |
| 240-02-02 | 02 | 2 | CRUD-01 | T-240-04 | create schema excludes Identifier/CreatedAt/tenant/Sensitive/UpdatedAt/list | unit | `cargo test -p ferro-mcp-server test_create_schema_exclusions` | ✅ | ✅ green |
| 240-02-02 | 02 | 2 | CRUD-01 | T-240-04 | create excludes Status under SM; includes Status when no SM | unit | `cargo test -p ferro-mcp-server test_create_schema_status_sm` | ✅ | ✅ green |
| 240-02-02 | 02 | 2 | CRUD-02 | T-240-05 | update schema: identifier required, data fields optional (patch) | unit | `cargo test -p ferro-mcp-server test_update_schema_patch_semantics` | ✅ | ✅ green |
| 240-02-02 | 02 | 2 | CRUD-02 | T-240-04 | update excludes Status under SM | unit | `cargo test -p ferro-mcp-server test_update_schema_status_sm` | ✅ | ✅ green |
| 240-02-02 | 02 | 2 | CRUD-01 | T-240-04 | delete schema: identifier required + optional confirmation token | unit | `cargo test -p ferro-mcp-server test_delete_schema` | ✅ | ✅ green |
| 240-03-01 | 03 | 3 | CRUD-01/02 | T-240-07 | `create_/update_/delete_<svc>` listed when flag set, absent when false; delete carries destructiveHint | unit | `cargo test -p ferro-mcp-server test_crud_tools_emitted_when_flags_set` | ✅ | ✅ green |
| 240-03-02 | 03 | 3 | CRUD-01/02 (Ph205 guard) | T-240-08/09 | CRUD verb call returns valid `CallToolResult` NTI envelope (not `-32601`) | integration | `cargo test -p ferro-mcp-server crud_tool_call_nti_parses_as_valid_mcp_content` | ✅ | ✅ green |
| 240-03-02 | 03 | 3 | CRUD-01/02 | T-240-07 | unflagged verb falls through to `-32601`, NOT a misleading NTI (WR-04 regression) | integration | `cargo test -p ferro-mcp-server crud_nti_not_returned_when_verb_flag_disabled` | ✅ | ✅ green |
| 240-04-02 | 04 | 3 | CRUD-04 | T-240-10/12 | range filters (`__gt`/`__lte`) return correct rows (SQLite in-memory) | integration | `cargo test -p ferro-mcp-server range_filter_returns_correct_rows` | ✅ | ✅ green |
| 240-04-02 | 04 | 3 | CRUD-04 | T-240-11/14 | `__in` array filter returns correct rows; empty array rejected | integration | `cargo test -p ferro-mcp-server in_filter_returns_correct_rows` | ✅ | ✅ green |
| 240-04-02 | 04 | 3 | CRUD-04 | T-240-12 | `sort=field` / `sort=-field` orders rows correctly | integration | `cargo test -p ferro-mcp-server sort_orders_rows` | ✅ | ✅ green |
| 240-04-02 | 04 | 3 | CRUD-04 | T-240-13 | back-compat: existing equality filters + tenant/`deleted_at` predicates unchanged | integration | `cargo test -p ferro-mcp-server equality_filter_backcompat` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. All 16 tests verified present and passing (full suite green: 277 ferro-projections + 56 ferro-mcp-server lib + integration suites, 0 failures).*

---

## Wave 0 Requirements

All test functions were written inline (TDD) alongside the implementation in each task — no
deferred Wave 0 stubs remained. All complete:

- [x] `ferro-projections/src/service.rs` — table test `is_write_excluded_field_gates` (9 cases)
- [x] `ferro-mcp-server/src/schema.rs` — `build_create/update/delete_input_schema`, `is_range_filter_field`, extended `build_input_schema` (range/ne/in/sort params)
- [x] `ferro-mcp-server/src/renderer.rs` — CRUD tool emission in `render_exposed_tools`
- [x] `ferro-mcp-server/src/dispatch.rs` — SQLite in-memory range/sort/`__in` + back-compat
- [x] `ferro-mcp-server/src/jsonrpc.rs` — Phase 205 guard extension + WR-04 NTI-gating regression test

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All phase behaviors have automated verification. |

*Create/update/delete *execution* over `:8090/mcp` is Phase 243 (out of scope here);
Phase 240's write tools are listed-but-not-callable, validated by the NTI-envelope test above.*

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (none remained — all tests written inline)
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-06-23

---

## Validation Audit 2026-06-23

| Metric | Count |
|--------|-------|
| Requirements (CRUD-01/02/04) | 3 |
| Requirements COVERED | 3 |
| Tests asserted present in code | 16 |
| Gaps found | 0 |
| Resolved | 0 (none needed) |
| Escalated to manual-only | 0 |

**Method:** State A audit. Cross-referenced every test named in the Per-Task Map against the
actual source (`grep fn <name>` in `ferro-projections/src` + `ferro-mcp-server/src`) — all 16
present. Full suite green (277 ferro-projections + 56 ferro-mcp-server lib + integration suites,
0 failures). All three phase requirements have automated, deterministic, sub-90s verification.
No `gsd-nyquist-auditor` spawn required — zero gaps. Phase is Nyquist-compliant.
