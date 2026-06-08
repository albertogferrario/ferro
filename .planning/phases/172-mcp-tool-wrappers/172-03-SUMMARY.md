---
phase: 172-mcp-tool-wrappers
plan: "03"
subsystem: ferro-mcp
tags: [ai-scaffold, ai-explain, mcp-tool-registration, service-def, agent-sufficient-descriptions]
dependency_graph:
  requires: [ferro-mcp/src/tools/ai_scaffold.rs, ferro-mcp/src/tools/ai_explain_core.rs]
  provides: [AiScaffoldParams, AiExplainParams, ai_scaffold #[tool], ai_explain #[tool]]
  affects: [ferro-mcp/src/service.rs]
tech_stack:
  added: []
  patterns: [tool-registration, params-struct-derive, match-on-result-error-encoding]
key_files:
  created: []
  modified:
    - ferro-mcp/src/service.rs
decisions:
  - "AiScaffoldParams and AiExplainParams derive Debug/Clone/Deserialize/Serialize/JsonSchema — identical to TestClassifierParams (required by rmcp Parameters<T>)"
  - "ai_scaffold success path returns ServiceDef directly (not wrapped in success:true) per D-02"
  - "Error path encodes { success: false, error } JSON — never panics, never exits (D-04, T-172-CRASH)"
  - "Tool descriptions carry all four SC#4 markers: when-to-use, returns, no-write semantics, zero-LLM-token structured branch, token-cost note, cross-links"
metrics:
  duration: 105s
  completed: "2026-06-08"
  tasks_completed: 1
  files_changed: 1
---

# Phase 172 Plan 03: MCP Tool Wrappers — service.rs Registration Summary

**One-liner:** Registered `ai_scaffold` and `ai_explain` as in-process MCP tools on `FerroMcpService` with agent-sufficient descriptions and no-panic error encoding, surfacing the Plan 02 async cores as callable tools (AICLI-05).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add AiScaffoldParams + AiExplainParams structs and the two #[tool] methods | ef20559a | ferro-mcp/src/service.rs |

## Verification

- All grep acceptance criteria: passed
- `cargo build -p ferro-mcp --all-features`: clean
- `cargo test -p ferro-mcp --all-features --no-run`: clean
- `cargo fmt -p ferro-mcp -- --check`: clean
- `cargo clippy -p ferro-mcp --all-features -- -D warnings`: clean

## Acceptance Criteria

- [x] `grep -q "pub struct AiScaffoldParams" ferro-mcp/src/service.rs` exits 0
- [x] `grep -q "pub struct AiExplainParams" ferro-mcp/src/service.rs` exits 0
- [x] `grep -q 'name = "ai_scaffold"' ferro-mcp/src/service.rs` exits 0
- [x] `grep -q 'name = "ai_explain"' ferro-mcp/src/service.rs` exits 0
- [x] `grep -q "ai_scaffold::scaffold_core" ferro-mcp/src/service.rs` exits 0
- [x] `grep -q "ai_explain_core::explain_core" ferro-mcp/src/service.rs` exits 0
- [x] `grep -q "Does NOT write files" ferro-mcp/src/service.rs` exits 0 — SC#4
- [x] `grep -q "ZERO LLM tokens" ferro-mcp/src/service.rs` exits 0 — SC#4
- [x] `cargo build -p ferro-mcp --all-features` exits 0
- [x] `cargo test -p ferro-mcp --all-features --no-run` exits 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rustfmt line-length violation on scaffold_core call**
- **Found during:** Task 1 fmt check
- **Issue:** `tools::ai_scaffold::scaffold_core(&params.0.description, &self.project_root).await` exceeded the 100-char line limit; `cargo fmt -- --check` reported a diff.
- **Fix:** Split the call onto two lines: `let result =\n    tools::ai_scaffold::scaffold_core(...).await;`
- **Files modified:** ferro-mcp/src/service.rs
- **Commit:** ef20559a (incorporated before commit)

## Known Stubs

None. Both tool methods are fully wired:
- `ai_scaffold` calls `tools::ai_scaffold::scaffold_core` (implemented in Plan 02) and returns the `ServiceDef` value directly.
- `ai_explain` calls `tools::ai_explain_core::explain_core` (implemented in Plan 02) with all three parameters.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes. All Plan 03 threat mitigations verified:

| Threat ID | Mitigation Status |
|-----------|-------------------|
| T-172-PI | DELEGATED — `sanitize_description` runs inside `scaffold_core` on every invocation regardless of caller; this method forwards `params.0.description` directly with no bypass |
| T-172-CRASH | IMPLEMENTED — both methods match on `Result`; `serde_json::to_string_pretty` failure falls back to a literal string; no `unwrap()` on core result, no `panic!`, no `process::exit` |
| T-172-INFO | ACCEPTED — error strings surface config/LLM context to local MCP client only; no remote auth boundary |

## Self-Check: PASSED

- `ferro-mcp/src/service.rs` contains `pub struct AiScaffoldParams` ✓
- `ferro-mcp/src/service.rs` contains `pub struct AiExplainParams` ✓
- `ferro-mcp/src/service.rs` contains `name = "ai_scaffold"` ✓
- `ferro-mcp/src/service.rs` contains `name = "ai_explain"` ✓
- `ferro-mcp/src/service.rs` contains `ai_scaffold::scaffold_core` ✓
- `ferro-mcp/src/service.rs` contains `ai_explain_core::explain_core` ✓
- `ferro-mcp/src/service.rs` contains `Does NOT write files` ✓
- `ferro-mcp/src/service.rs` contains `ZERO LLM tokens` ✓
- Commit ef20559a exists ✓
- `cargo build -p ferro-mcp --all-features` clean ✓
- `cargo clippy -p ferro-mcp --all-features -- -D warnings` clean ✓
