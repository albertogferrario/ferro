---
phase: 240
slug: crud-input-schema-derivation-list-query-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-23
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
| TBD | TBD | TBD | CRUD-01 | — | `create_<svc>` listed when `creatable=true`, absent when false | unit | `cargo test -p ferro-mcp-server test_crud_tool_listing` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-01 | — | create schema excludes Identifier/CreatedAt/tenant/Sensitive/list | unit | `cargo test -p ferro-mcp-server test_create_schema_exclusions` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-01 | — | create excludes Status under SM; includes Status when no SM | unit | `cargo test -p ferro-mcp-server test_create_schema_status_sm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-01 | — | `is_write_excluded_field` predicate correctness | unit | `cargo test -p ferro-projections test_write_excluded_field` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-02 | — | update schema: identifier required, data fields optional (patch) | unit | `cargo test -p ferro-mcp-server test_update_schema_patch_semantics` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-02 | — | update excludes Status under SM | unit | `cargo test -p ferro-mcp-server test_update_schema_status_sm` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-04 | — | `__gt/__gte/__lt/__lte` emitted for Integer/Float/DateTime/Date | unit | `cargo test -p ferro-mcp-server test_range_params_in_schema` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-04 | — | `__ne/__in` emitted for all `is_filter_field` fields | unit | `cargo test -p ferro-mcp-server test_ne_in_params_in_schema` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-04 | — | `sort` param emitted | unit | `cargo test -p ferro-mcp-server test_sort_param_in_schema` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-04 | — | range filters return correct rows (SQLite in-memory) | integration | `cargo test -p ferro-mcp-server range_filter_returns_correct_rows` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-04 | — | `__in` array filter returns correct rows | integration | `cargo test -p ferro-mcp-server in_filter_returns_correct_rows` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-04 | — | `sort=field` / `sort=-field` orders rows correctly | integration | `cargo test -p ferro-mcp-server sort_orders_rows` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-04 | — | back-compat: existing equality filters unchanged | integration | `cargo test -p ferro-mcp-server equality_filter_backcompat` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CRUD-01/02 (Ph205 guard) | — | `create_/update_/delete_` calls return valid `CallToolResult` (NTI envelope, not `-32601`) | integration | `cargo test -p ferro-mcp-server crud_tool_call_parses_as_valid_mcp_content` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky. Task IDs filled by the planner.*

---

## Wave 0 Requirements

All test functions below are new; existing test idioms in each file provide the fixture
patterns. No new test files or framework config needed.

- [ ] `ferro-projections/src/service.rs` — table tests for `is_write_excluded_field`
- [ ] `ferro-mcp-server/src/schema.rs` — `build_create/update/delete_input_schema`, `is_range_filter_field`, extended `build_input_schema` (range/sort params)
- [ ] `ferro-mcp-server/src/renderer.rs` — CRUD tool emission in `render_exposed_tools`
- [ ] `ferro-mcp-server/src/dispatch.rs` — SQLite in-memory range/sort/`__in` + back-compat
- [ ] `ferro-mcp-server/src/jsonrpc.rs` — Phase 205 guard extension for CRUD verb calls (NTI envelope)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | All phase behaviors have automated verification. |

*Create/update/delete *execution* over `:8090/mcp` is Phase 243 (out of scope here);
Phase 240's write tools are listed-but-not-callable, validated by the NTI-envelope test above.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
