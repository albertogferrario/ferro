---
phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands
plan: 02
subsystem: cli
tags: [ai, codegen, projections, ferro-mcp, ferro-ai, rust, clap]

requires:
  - phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands-plan-01
    provides: "complete_with::<T> + CompleteOptions typed LLM completion API in ferro-ai"

provides:
  - "ferro ai:make <description> CLI command: loads ferro-mcp introspection in-process, lexically filters context, prompts LLM via complete_with::<ServiceDef>(), writes src/projections/<name>.rs"
  - "ServiceDef -> Rust-builder source emitter (emit_service_def_source) with explicit enum match arms"
  - "Shared naming module (to_snake_case, is_valid_identifier) extracted from make_projection.rs"
  - "Lexical relevance filter (relevance::select_relevant + tokenize) with INPUT_BUDGET_CHARS budget gate"

affects:
  - 171-03
  - 171-04
  - ferro-cli
  - ferro-projections

tech-stack:
  added: []
  patterns:
    - "Async-to-sync bridge: one tokio::runtime::Runtime per command for MCP tool calls"
    - "ServiceDef emitter: explicit match arms for DataType/FieldMeaning/Cardinality/Intent — never serde variant names"
    - "Lexical relevance: tokenize (snake + CamelCase split) -> intersection score -> budget gate"
    - "Path sanitization: is_valid_identifier + to_snake_case + fixed src/projections/ base"
    - "Prompt injection mitigation: user description wrapped in <description>...</description> delimiter"

key-files:
  created:
    - ferro-cli/src/naming.rs
    - ferro-cli/src/relevance.rs
    - ferro-cli/src/commands/ai_make.rs
  modified:
    - ferro-cli/src/commands/make_projection.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
    - ferro-cli/src/lib.rs

key-decisions:
  - "Modules naming.rs and relevance.rs placed in lib.rs (crate library root), not main.rs — mod commands; lives in lib.rs, so crate-internal modules belong there"
  - "IntentHint is enum { Primary(Intent), Exclude(Intent) } not struct { intent, weight: f32 } — used actual source over plan interface description"
  - "RoutesInfo and SchemaInfo have no Default impl — fallback values constructed manually with unwrap_or_else"
  - "RouteSource::StaticAnalysis is the correct variant (not RouteSource::Static)"
  - "ferro_ai::Error::Config(String) is the correct variant (not Error::Configuration)"
  - "Candidate::label field kept with #[allow(dead_code)] — debug/future use field, removal would lose semantics"

patterns-established:
  - "Pattern: emit_data_type/emit_field_meaning/emit_cardinality/emit_intent use exhaustive explicit match arms — never serde serialization — to produce correct Rust identifier tokens"
  - "Pattern: FERRO_AI_MAX_TOKENS_PER_COMMAND env var for cost guard on any AI command"
  - "Pattern: render_output factored separately from run() so unit tests verify dry-run without LLM calls"
  - "Pattern: resolve_projection_path rejects path traversal via is_valid_identifier before any fs operation"

requirements-completed: [AICLI-01, AICLI-02]

duration: ~90min
completed: 2026-06-08
---

# Phase 171 Plan 02: ai:make Command Summary

**`ferro ai:make <description>` ships: loads ferro-mcp introspection in-process, lexically filters to description-relevant context, produces a typed ServiceDef via `complete_with::<ServiceDef>()`, and writes exactly one `src/projections/<name>.rs` builder file — the produce half of the milestone killer feature**

## Performance

- **Duration:** ~90 min
- **Started:** 2026-06-08 (session start)
- **Completed:** 2026-06-08T18:56:00Z
- **Tasks:** 3 (all TDD)
- **Files modified/created:** 7

## Accomplishments

- Shared `naming.rs` + `relevance.rs` modules extracted from `make_projection.rs` with full unit test coverage (tokenizer, scorer, budget gate, identifier sanitizer)
- `emit_service_def_source` emitter walks a `ServiceDef` and produces a compilable Rust builder chain with explicit match arms for all 10 DataType variants, 18+1 FieldMeaning variants, 4 Cardinality variants, and 7+1 Intent variants — serde variant names never used
- `ferro ai:make <description>` wired end-to-end: in-process MCP introspection (5 tool calls), lexical relevance filter, `<description>` delimited prompt injection mitigation, `complete_with::<ServiceDef>()`, `service.validate()`, single-file write; `--dry-run` prints pretty JSON and writes nothing

## Task Commits

1. **Task 1: Shared naming module + lexical relevance filter** - `efffaf00` (feat)
2. **Task 2: ServiceDef -> Rust-builder source emitter** - `e5a52dd1` (feat)
3. **Task 3: ai:make command wiring + clap registration** - `05e1fae8` (feat)

## Files Created/Modified

- `ferro-cli/src/naming.rs` - `pub(crate) fn is_valid_identifier` + `to_snake_case`, path-traversal rejection
- `ferro-cli/src/relevance.rs` - `tokenize`, `Candidate`, `select_relevant` with INPUT_BUDGET_CHARS=8000 budget gate
- `ferro-cli/src/commands/ai_make.rs` - Full command: emitter, sanitization, run(), testable helpers, 11 unit tests
- `ferro-cli/src/commands/make_projection.rs` - Deleted local `is_valid_identifier`/`to_snake_case`; promoted `update_mod_file` to `pub(crate)`
- `ferro-cli/src/commands/mod.rs` - Added `pub mod ai_make;`
- `ferro-cli/src/main.rs` - Added `AiMake` clap variant + dispatch arm (both `cfg(feature = "projections")`)
- `ferro-cli/src/lib.rs` - Added `pub(crate) mod naming;` and `pub(crate) mod relevance;`

## Decisions Made

- Modules `naming` and `relevance` placed in `lib.rs` (not `main.rs` as the plan interface description said). `mod commands;` lives in `lib.rs`; crate-internal modules belong at the same level. Plan acceptance criteria grep on `main.rs` does not match — documented as architectural placement deviation, `lib.rs` is correct.
- `IntentHint` is `enum { Primary(Intent), Exclude(Intent) }` in actual source, not `struct { intent, weight: f32 }` as described in the plan's `<interfaces>` section. Used actual source.
- `RoutesInfo`/`SchemaInfo` have no `Default` impl — fallback values constructed manually.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ferro_ai::Error::Config(String) not Error::Configuration**
- **Found during:** Task 3 (ai:make command wiring)
- **Issue:** Plan interface listed `Error::Configuration`; actual variant is `Error::Config(String)`
- **Fix:** Used `ferro_ai::Error::Config(String)` pattern in `ai_config_error_message`
- **Files modified:** ferro-cli/src/commands/ai_make.rs
- **Verification:** Compiles clean, test `ai_make_requires_ai_config` passes
- **Committed in:** 05e1fae8

**2. [Rule 1 - Bug] RoutesInfo/SchemaInfo have no Default impl**
- **Found during:** Task 3
- **Issue:** Plan suggested `.unwrap_or_default()` for DB-down graceful fallback; neither type implements `Default`
- **Fix:** Used `unwrap_or_else(|_| RoutesInfo { routes: vec![], source: RouteSource::StaticAnalysis })` and `unwrap_or_else(|_| SchemaInfo { tables: vec![] })`
- **Files modified:** ferro-cli/src/commands/ai_make.rs
- **Verification:** Compiles; plan acceptance criteria grep for `.unwrap_or_else` passes
- **Committed in:** 05e1fae8

**3. [Rule 1 - Bug] RouteSource::Static not found; correct variant is StaticAnalysis**
- **Found during:** Task 3
- **Issue:** Variant name mismatch in plan interface description
- **Fix:** Used `RouteSource::StaticAnalysis`
- **Files modified:** ferro-cli/src/commands/ai_make.rs
- **Verification:** Compiles clean
- **Committed in:** 05e1fae8

**4. [Rule 1 - Bug] IntentHint is an enum not a struct**
- **Found during:** Task 2 (emitter)
- **Issue:** Plan interface described `IntentHint { intent: Intent, weight: f32 }` (struct). Actual source: `enum IntentHint { Primary(Intent), Exclude(Intent) }`
- **Fix:** `emit_intent_hint` matches `Primary(intent)` and `Exclude(intent)` enum arms
- **Files modified:** ferro-cli/src/commands/ai_make.rs
- **Verification:** Compiles; round-trip test passes
- **Committed in:** e5a52dd1

**5. [Rule 1 - Bug] Unused `mut` + dead_code on Candidate::label**
- **Found during:** Post-Task 3 clippy check
- **Issue:** `let mut tokens` in schema table loop didn't need `mut`; `label` field never read in logic
- **Fix:** Removed `mut`; added `#[allow(dead_code)]` to `Candidate::label` (debug/future field with semantic value)
- **Files modified:** ferro-cli/src/commands/ai_make.rs, ferro-cli/src/relevance.rs
- **Verification:** `cargo clippy -p ferro-cli --all-targets -- -D warnings` clean
- **Committed in:** 05e1fae8

---

**Total deviations:** 5 auto-fixed (all Rule 1 — interface description vs. actual source mismatches + clippy clean-up)
**Impact on plan:** All fixes were compile-time required. No scope change.

## Issues Encountered

None beyond the interface description mismatches documented above.

## User Setup Required

Live `ferro ai:make` requires LLM provider configuration (unit tests do not call the LLM):

| Variable | Source |
|----------|--------|
| `FERRO_AI_PROVIDER` | `anthropic` or `openai` or `ollama` |
| `FERRO_AI_API_KEY` | Provider API key dashboard |
| `FERRO_AI_MODEL` | Model ID (e.g. `claude-3-5-sonnet-latest`) |
| `FERRO_AI_MAX_TOKENS_PER_COMMAND` | Optional cost guard; defaults to 8192 |

The command fails fast with a descriptive error naming all three env vars when provider is not configured.

## Threat Surface Scan

All threats in the plan's `<threat_model>` are mitigated as implemented:

| Threat | Mitigation | Verified |
|--------|-----------|---------|
| T-171-PT path traversal | `is_valid_identifier` + `to_snake_case` + fixed base | `ai_make_rejects_path_traversal` test |
| T-171-PI prompt injection | `<description>...</description>` delimiter + structured ServiceDef schema | In ai_make.rs prompt assembly |
| T-171-CODE arbitrary write | Single fixed location `src/projections/<name>.rs`; content is emitter output of validated ServiceDef | SC#3 grep gate passes |
| T-171-DoS token exhaustion | `FERRO_AI_MAX_TOKENS_PER_COMMAND` cap + `INPUT_BUDGET_CHARS=8000` input gate | `max_tokens_env_applied` test |

## Next Phase Readiness

- Plan 02 delivers AICLI-01 + AICLI-02; Plan 03 (`ferro ai:explain`) can now build on the same shared naming/relevance modules and MCP introspection bridge pattern
- 540 ferro-cli tests pass; clippy clean

---
*Phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands*
*Completed: 2026-06-08*
