---
phase: 30-cli-scaffolding
plan: 02
subsystem: cli
tags: [anthropic, api, prompt-engineering, caching, ai]

# Dependency graph
requires:
  - phase: 30-01
    provides: AI-powered make:json-view command with Anthropic API client
provides:
  - Optimized Anthropic API call with system prompt caching, prefill, temperature control
  - Structured prompt with few-shot example for consistent code generation
  - Cost-efficient Sonnet default model
affects: [31-mcp-ui-tools]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "System/user prompt separation for Anthropic API"
    - "Assistant prefill to force code-only output"
    - "Prompt caching via cache_control on system block"

key-files:
  created: []
  modified:
    - ferro-cli/src/ai.rs
    - ferro-cli/src/commands/make_json_view.rs

key-decisions:
  - "Sonnet default instead of Opus for ~5x cost reduction"
  - "Assistant prefill //! eliminates markdown fence stripping"
  - "Single system block with cache_control for entire static content"

patterns-established:
  - "System prompt: role + rules + catalog + few-shot example (cacheable)"
  - "User prompt: dynamic project context only"

# Metrics
duration: 3min
completed: 2026-02-09
---

# Phase 30 Plan 02: Anthropic API Best Practices Summary

**System prompt with cache_control, assistant prefill, Sonnet default, few-shot example, and 60s timeout for make:json-view AI generation**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-09T09:30:50Z
- **Completed:** 2026-02-09T09:34:20Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Restructured `call_anthropic()` with system/user separation, cache_control, temperature 0.2, prefill, and 60s timeout
- Restructured `build_view_context()` to return (system, user) tuple with few-shot example in system prompt
- Removed `strip_markdown_fences()` workaround (assistant prefill handles format)
- Changed default model from claude-opus-4-6 to claude-sonnet-4-5

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Restructure API call and prompt** - `1a9fa23` (feat) + `8d19e1f` (feat)
2. **Task 3: Remove strip_markdown_fences** - `49b246a` (refactor)

## Files Created/Modified
- `ferro-cli/src/ai.rs` - Restructured call_anthropic() and build_view_context() with Anthropic best practices
- `ferro-cli/src/commands/make_json_view.rs` - Updated caller, removed strip_markdown_fences

## Decisions Made
- Sonnet default instead of Opus for cost efficiency (env var override still works)
- Assistant prefill `//!` eliminates need for markdown fence stripping
- Single system block with cache_control wrapping entire static content (role + rules + catalog + example)
- Removed dead `to_title_case` from ai.rs (no longer needed after prompt restructure)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed dead to_title_case function from ai.rs**
- **Found during:** Task 2 (prompt restructure)
- **Issue:** `to_title_case` was only used by old `build_view_context()` to generate a title in the prompt. New version no longer uses it, causing dead_code warning.
- **Fix:** Deleted the function
- **Files modified:** ferro-cli/src/ai.rs
- **Verification:** cargo build produces no warnings
- **Committed in:** 1a9fa23 (Task 1+2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Removed dead code to maintain clean build. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 30 (CLI Scaffolding) is complete with both plans finished
- `make:json-view` command now uses Anthropic API best practices
- Ready for Phase 31 (MCP UI Tools)

---
*Phase: 30-cli-scaffolding*
*Completed: 2026-02-09*
