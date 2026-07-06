---
phase: 172-mcp-tool-wrappers
verified: 2026-06-08T12:00:00Z
status: human_needed
score: 5/5
overrides_applied: 0
human_verification:
  - test: "Invoke ai_scaffold via MCP with FERRO_AI_PROVIDER/FERRO_AI_API_KEY/FERRO_AI_MODEL configured against the sample app"
    expected: "Returns a ServiceDef JSON object with name, fields (FieldMeaning annotations), intent_hints, actions — no files written"
    why_human: "Requires real LLM API credentials + network call; non-deterministic output cannot be asserted programmatically"
  - test: "Invoke ai_explain via MCP targeting a known service name that has a ServiceDef"
    expected: "Returns structured projection JSON (name, fields with meaning/readable/writable, relationships, actions, has_state_machine, intent_hints) with zero LLM token spend"
    why_human: "Requires a live project with at least one ServiceDef and MCP server running; zero-token branch is deterministic but not invokable without environment"
  - test: "Invoke ai_explain via MCP targeting a route path that has no backing ServiceDef"
    expected: "Returns { \"prose\": \"...\" } — a prose LLM explanation of the route"
    why_human: "Requires real LLM API credentials + network call for the prose fallback branch"
---

# Phase 172: MCP Tool Wrappers Verification Report

**Phase Goal:** Expose `ai_scaffold` and `ai_explain` as ferro-mcp tools so agents can invoke `ServiceDef` production and projection-framed explanation logic in-process without shelling out to the CLI.
**Verified:** 2026-06-08T12:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `ai_scaffold` MCP tool accepts `description: String` and returns `ServiceDef` JSON (no ScaffoldPlan, no disk write) | VERIFIED | `AiScaffoldParams { description: String }` at `service.rs:346-349`; `scaffold_core` returns `Result<ServiceDef, String>` with no file I/O; tool method serializes via `serde_json::to_string_pretty`; `service.rs:1706-1735` |
| 2 | `ai_explain` MCP tool accepts `target: String` and returns structured projection JSON or `{ "prose": ... }` fallback | VERIFIED | `AiExplainParams { target: String, type_override: Option<String> }` at `service.rs:352-357`; `explain_core` two-branch: `ResolvedTarget::Service(detail)` → `serde_json::to_value(&detail)` (zero LLM tokens); Route/Model → `serde_json::json!({ "prose": prose })`; `ai_explain_core.rs:319-340` |
| 3 | Both tools share the same logic path as the CLI — no duplicate implementation | VERIFIED | `ferro-cli/src/relevance.rs` MISSING (confirmed deleted); `ai_make.rs` calls `ferro_mcp::tools::ai_scaffold::scaffold_core` (confirmed at line 469); `ai_explain.rs` imports from `ferro_mcp::tools::ai_explain_core` (confirmed at line 27-28); `ferro-mcp/Cargo.toml` has no `ferro-cli` dependency (confirmed empty grep) |
| 4 | MCP tool descriptions are accurate and agent-sufficient; `docs/src/features/ai.md` documents both tools | VERIFIED | `#[tool(description=...)]` strings at `service.rs:1707-1720` and `1740-1753` cover when-to-use, return shape, token-cost note, FERRO_AI_* env vars, cross-links to `list_projections`/`inspect_projection`/`ai_explain`/`ai_scaffold`; `docs/src/features/ai.md` lines 344-370 document both tools with returns, notes, cross-links |
| 5 | `ferro-mcp` version bumped to 0.2.47; full gate passes | VERIFIED | `Cargo.toml:36` = `"0.2.47"`; ferro-mcp uses `version.workspace = true`; Plan 04 SUMMARY reports fmt + clippy -D warnings + test --all-features all green (commits 60072ac9, fb630cc1, d978ce64 confirmed in git); targeted `cargo build -p ferro-mcp` confirmed clean (9.15s, v0.2.47) |

**Score:** 5/5 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-mcp/src/tools/ai_scaffold.rs` | `scaffold_core` returning `Result<ServiceDef, String>`, no disk write | VERIFIED | File exists, 359 lines; `async fn scaffold_core(description: &str, project_root: &Path) -> Result<ServiceDef, String>`; no file I/O in function body |
| `ferro-mcp/src/tools/ai_explain_core.rs` | `explain_core` two-branch: ServiceDef→structured JSON; Route/Model→prose | VERIFIED | File exists, 594 lines; `async fn explain_core(...) -> Result<serde_json::Value, String>`; `ResolvedTarget::Service(detail)` → `serde_json::to_value(&detail)`; prose branches return `serde_json::json!({ "prose": prose })` |
| `ferro-mcp/src/tools/relevance.rs` | Relocated relevance filter (was `ferro-cli/src/relevance.rs`) | VERIFIED | File exists in ferro-mcp; `ferro-cli/src/relevance.rs` confirmed MISSING; no `mod relevance` in `ferro-cli/src/lib.rs` |
| `ferro-mcp/src/service.rs` — `ai_scaffold` tool method | Registered with `#[tool(name = "ai_scaffold", description = ...)]` | VERIFIED | Lines 1706-1735; params struct `AiScaffoldParams`; calls `tools::ai_scaffold::scaffold_core` |
| `ferro-mcp/src/service.rs` — `ai_explain` tool method | Registered with `#[tool(name = "ai_explain", description = ...)]` | VERIFIED | Lines 1739-1770; params struct `AiExplainParams`; calls `tools::ai_explain_core::explain_core` |
| `ferro-cli/src/commands/ai_make.rs` | Thin wrapper calling into `ferro_mcp::tools::ai_scaffold::scaffold_core` | VERIFIED | Module doc says "Thin CLI wrapper"; `rt.block_on(ferro_mcp::tools::ai_scaffold::scaffold_core(...))` at line 469; no `complete_with::<ServiceDef>` in file (confirmed empty grep) |
| `ferro-cli/src/commands/ai_explain.rs` | Thin wrapper importing from `ferro_mcp::tools::ai_explain_core` | VERIFIED | Module doc says thin wrapper; imports `resolve_target`, `build_*_prompt`, `resolve_max_tokens_with_default`, `ResolvedTarget` from ferro_mcp; `schema: None` at line 103 (prose-only path preserved) |
| `docs/src/features/ai.md` — MCP tools section | Documents both `ai_scaffold` and `ai_explain` | VERIFIED | Lines 344-370; covers returns, token-cost note, does-not-write-files note, cross-links |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-cli/src/commands/ai_make.rs` | `ferro-mcp/src/tools/ai_scaffold::scaffold_core` | `rt.block_on(...)` | WIRED | Line 469 of ai_make.rs calls the relocated core with tokio runtime bridge (CLI-side only) |
| `ferro-cli/src/commands/ai_explain.rs` | `ferro-mcp/src/tools/ai_explain_core::*` | `use ferro_mcp::tools::ai_explain_core::{...}` | WIRED | Lines 27-28 import resolve_target, build_*_prompt, resolve_max_tokens_with_default, ResolvedTarget |
| `ferro-mcp/src/service.rs::ai_scaffold` | `ferro-mcp/src/tools/ai_scaffold::scaffold_core` | `tools::ai_scaffold::scaffold_core(...)` | WIRED | Direct same-crate async call at service.rs:1724 |
| `ferro-mcp/src/service.rs::ai_explain` | `ferro-mcp/src/tools/ai_explain_core::explain_core` | `tools::ai_explain_core::explain_core(...)` | WIRED | Direct same-crate async call at service.rs:1756-1760 |
| `ferro-mcp` dependency | `ferro-cli` | None | VERIFIED ABSENT | grep for "ferro-cli" in ferro-mcp/Cargo.toml returns empty — no reverse dependency cycle |
| `ferro-cli` dependency | `ferro-mcp` | `ferro-mcp = { path = "../ferro-mcp" }` | WIRED | ferro-cli/Cargo.toml line 45 confirms correct unidirectional dependency |

---

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `ai_scaffold.rs::scaffold_core` | `ServiceDef` returned by `complete_with::<ServiceDef>()` | LLM via `ferro_ai::complete_with` + live introspection from `list_models`, `list_routes`, `database_schema`, `list_projections`, `generation_context` | Yes — live introspection then LLM completion | FLOWING |
| `ai_explain_core.rs::explain_core` | `serde_json::Value` — either `ProjectionDetail` or `{ "prose": ... }` | `inspect_projection::execute` (ServiceDef found) or `call_llm_prose` (route/model fallback) | Yes — either deterministic projection data or LLM completion | FLOWING |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| ferro-mcp compiles at v0.2.47 | `cargo build -p ferro-mcp` | Finished dev in 9.15s, no warnings | PASS |
| `ai_scaffold` tool registered in service (grep) | `grep "name = \"ai_scaffold\"" ferro-mcp/src/service.rs` | Line 1706 | PASS |
| `ai_explain` tool registered in service (grep) | `grep "name = \"ai_explain\"" ferro-mcp/src/service.rs` | Line 1739 | PASS |
| `ferro-cli/src/relevance.rs` deleted | `ls ferro-cli/src/relevance.rs` | MISSING (exit 1) | PASS |
| No `complete_with::<ServiceDef>` in ferro-cli ai_make.rs | grep | Empty result | PASS |
| No `fn resolve_target` in ferro-cli | grep | Empty result | PASS |
| Workspace version 0.2.47 | `grep "^version" Cargo.toml` | `version = "0.2.47"` | PASS |
| Plan 04 commits exist in git | `git log --oneline d978ce64 fb630cc1 60072ac9` | All three present | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| AICLI-05 | 172-01 through 172-04 | MCP tools `ai_scaffold` and `ai_explain` in `ferro-mcp` wrap the CLI command logic for in-process agent consumption. No parallel surface. | SATISFIED | Both tools registered in service.rs; CLI files call into ferro-mcp cores; single definition site enforced by compile-time structure; `ferro-cli/src/relevance.rs` deleted |

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `ferro-cli/src/commands/ai_explain.rs` | 61 | `resolve_target(Path::new("."), ...)` — relative path instead of `current_dir()` | Warning (WR-02 from REVIEW) | Latent: if MCP server or test sets CWD away from project root, ai_explain CLI silently looks in wrong place. ai_make uses `current_dir()` correctly. Not a goal blocker — MCP tool path is unaffected (uses `self.project_root`). |
| `ferro-cli/src/commands/ai_make.rs` | 488-493 | `eprintln!` + `process::exit(0)` for `AlreadyExists` path | Warning (WR-03 from REVIEW) | Misleading signal for scripting callers: stderr output with exit 0. CLI-side only; MCP tool unaffected. Not a goal blocker. |
| `ferro-mcp/src/tools/ai_scaffold.rs` | 35-39 | `sanitize_description` strips only `<description>` and `</description>` tags | Info (IN-01 from REVIEW) | Narrow scope: other XML tag names pass through. Low risk for current prompt structure. |

All three anti-patterns were identified in the code review (172-REVIEW.md). None block the MCP tool wrappers goal — the MCP tool path is unaffected by WR-02 (uses `self.project_root`) and WR-03 (CLI-only presentation). WR-02 and WR-03 were explicitly not fixed in Plan 04 (the fix scope in Plan 04 SUMMARY shows auto-fixed issues only).

---

### Human Verification Required

#### 1. Live `ai_scaffold` end-to-end via MCP

**Test:** With `FERRO_AI_PROVIDER`, `FERRO_AI_API_KEY`, and `FERRO_AI_MODEL` configured, invoke the `ai_scaffold` MCP tool with a natural-language description (e.g., "an invoice service for managing customer billing records").
**Expected:** Returns a `ServiceDef` JSON object with `name`, `fields` containing `FieldMeaning` annotations, `intent_hints`, `actions` — no files created in `src/projections/`.
**Why human:** Requires real LLM API credentials, network call, and token spend. Non-deterministic output — can only assert structural shape, not content.

#### 2. Live `ai_explain` structured branch via MCP

**Test:** With a project containing at least one `ServiceDef` projection, invoke `ai_explain` with the service name as the target.
**Expected:** Returns structured projection JSON containing `name`, `fields` (with `meaning`, `readable`, `writable` per field), `relationships`, `actions`, `has_state_machine`, `intent_hints` — produced with zero LLM token spend (no `FERRO_AI_*` env vars needed for this branch).
**Why human:** Requires a live project with a ServiceDef and the MCP server running. The zero-token deterministic path cannot be exercised via static checks alone.

#### 3. Live `ai_explain` prose fallback branch via MCP

**Test:** Invoke `ai_explain` with a route path or model name that has no backing `ServiceDef` (e.g., a built-in framework route).
**Expected:** Returns `{ "prose": "..." }` with an LLM-generated explanation of the route or model in plain prose.
**Why human:** Requires real LLM API credentials for the prose fallback path.

---

### Gaps Summary

No gaps blocking goal achievement. All 5 success criteria are verified by static codebase analysis and a scoped compilation check. The three human verification items require a live environment and cannot be asserted programmatically — they represent the final quality confirmation of the LLM integration, not structural defects.

The two code-review warnings (WR-02, WR-03) are CLI-side presentation issues that do not affect the MCP tool path or the phase goal. They can be addressed in a future cleanup phase.

---

_Verified: 2026-06-08T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
