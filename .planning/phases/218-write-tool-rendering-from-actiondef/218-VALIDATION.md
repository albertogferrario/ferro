---
phase: 218
slug: write-tool-rendering-from-actiondef
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 218 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` |
| **Config file** | None (Cargo built-in test runner) |
| **Quick run command** | `cargo test -p ferro-mcp-server --lib` |
| **Full suite command** | `cargo test -p ferro-mcp-server` |
| **Estimated runtime** | ~30–60 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-mcp-server --lib`
- **After every plan wave:** `cargo test -p ferro-mcp-server` (incl. integration) + `cargo clippy --all --all-targets -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite + clippy green
- **Max feedback latency:** ~60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 218-00-01 | 00 | 0 | AMCP-03 | — | RED tests exist and fail before impl | unit/integration | `cargo test -p ferro-mcp-server` | ❌ W0 | ⬜ pending |
| 218-01-01 | 01 | 1 | AMCP-03 | T-218-01 (sensitive-field leak in schema) | `build_action_input_schema` derives props from `ActionDef.inputs`, injects Identifier, excludes `FieldMeaning::Sensitive` | unit | `cargo test -p ferro-mcp-server schema::` | ❌ W0 | ⬜ pending |
| 218-02-01 | 02 | 2 | AMCP-03 | T-218-02 (guard-filter visibility) | one write tool per `ActionDef`, name = `action.name`; `readOnlyHint:false`, `destructiveHint = transition_trigger.is_some()` | unit | `cargo test -p ferro-mcp-server renderer::` | ❌ W0 | ⬜ pending |
| 218-02-02 | 02 | 2 | AMCP-03 | T-218-02 | guard `Some(false)` → tool absent from `tools/list`; `None`/`Some(true)` → present | unit | `cargo test -p ferro-mcp-server renderer::` | ❌ W0 | ⬜ pending |
| 218-02-03 | 02 | 2 | AMCP-03 | — | every write-tool definition in `tools/list` deserializes strictly via `rmcp::model::Tool` (Phase 205 guard extended) | integration | `cargo test -p ferro-mcp-server write_tools_definitions_parse` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp-server/src/schema.rs` `#[cfg(test)]` — RED `build_action_input_schema` tests (props from inputs, Identifier injection, `FieldMeaning::Sensitive` excluded, `required[]`)
- [ ] `ferro-mcp-server/src/renderer.rs` `#[cfg(test)]` — RED write-tool render tests (one per action, name = `action.name`, annotations, guard-filter present/absent)
- [ ] Phase 205 regression extension — new strict-deser test asserting write-tool definitions parse as `rmcp::model::Tool` (parallel to `tools_call_result_parses_as_valid_mcp_content` at `jsonrpc.rs:188`)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | — | All phase behaviors have automated coverage | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
