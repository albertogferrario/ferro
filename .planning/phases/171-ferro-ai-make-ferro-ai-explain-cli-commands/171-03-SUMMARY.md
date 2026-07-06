---
phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands
plan: 03
subsystem: cli
tags: [ai, cli, projections, introspection, explain, ferro-mcp, ferro-ai, rust, clap]

requires:
  - phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands-plan-02
    provides: "ai:make command, ai_config_error_message helper, resolve_max_tokens pattern"

provides:
  - "ferro ai:explain <target> CLI command: resolves target in service→route→model order, projection-framed prompt from inspect_projection parsed vocabulary, prose fallback via explain_route/explain_model, raw CompletionRequest{ schema: None } completion, --dry-run, fail-fast on missing AI config"

affects:
  - 171-04
  - ferro-cli

tech-stack:
  added: []
  patterns:
    - "Service-first resolution: inspect_projection (sync) → explain_route (async) → explain_model (async); type_override skips auto-detect"
    - "Projection-framed prompt: built from ProjectionDetail string vocabulary (intent_hints, fields+meanings, actions, relationships, has_state_machine) — no live ServiceDef, no derive_intents"
    - "Raw prose completion: CompletionRequest{ schema: None } — not complete_with::<String>()"
    - "resolve_max_tokens_with_default(2048): parameterized cost guard, reads FERRO_AI_MAX_TOKENS_PER_COMMAND"
    - "Dry-run: prints system---user prompt text and returns before client.complete()"
    - "Reuse: ai_config_error_message from ai_make (pub(crate)); same tokio bridge pattern"

key-files:
  created:
    - ferro-cli/src/commands/ai_explain.rs
  modified:
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs

key-decisions:
  - "resolve_kind_priority is a pure function (no introspection calls) used by unit tests; real resolve_target calls the tools in the same service→route→model order"
  - "Lifetime elision applied to resolve_kind_priority — clippy::needless_lifetimes enforced with -D warnings"
  - "Dry-run skips AiConfig::from_env() check — prompt printing is valid without credentials"
  - "10 unit tests cover: 4 resolution order cases, service prompt vocabulary (SC#4+SC#6), route prompt facts, dry-run no-call guarantee, max_tokens default+env, fail-fast error message env var names"

requirements-completed: [AICLI-03]

duration: ~315s
completed: 2026-06-08
---

# Phase 171 Plan 03: ai:explain Command Summary

**`ferro ai:explain <target>` ships: resolves any route/model/service in-process, builds a projection-framed prompt from inspect_projection's parsed vocabulary when a ServiceDef projection exists, and produces prose via a raw schema:None LLM call — the consume half of the milestone killer feature**

## Performance

- **Duration:** ~315s (~5 min)
- **Started:** 2026-06-08T18:59:51Z
- **Completed:** 2026-06-08T19:05:06Z
- **Tasks:** 2 (both TDD, implemented together in one file)
- **Files created/modified:** 3

## Accomplishments

- `ferro-cli/src/commands/ai_explain.rs` created with full implementation:
  - `resolve_target()`: auto-detects in service→route→model order; `--type` override forces a specific tool
  - `resolve_kind_priority()`: pure testable helper for resolution logic (no introspection side effects)
  - `build_service_prompt()`: projection-framed from `ProjectionDetail` parsed strings (Intent, FieldMeaning, ActionDef, StateMachine) — `derive_intents` is not called (no live ServiceDef)
  - `build_route_prompt()` / `build_model_prompt()`: prose-fallback from explain_route/explain_model facts
  - `resolve_max_tokens_with_default(2048)`: parameterized cost guard extending the Plan 02 pattern
  - `run()`: full command entry point — fail-fast, tokio bridge, resolve, prompt, dry-run gate, raw `CompletionRequest{ schema: None }` completion
- Clap `AiExplain` variant + dispatch arm registered in `main.rs` (cfg-gated on `projections` feature)
- 10 unit tests: all pass, clippy clean

## Task Commits

1. **Tasks 1+2: ai:explain implementation (resolution, prompts, run, clap registration)** — `8a83d266` (feat)

## Files Created/Modified

- `ferro-cli/src/commands/ai_explain.rs` — Full command: target resolution, prompt builders, run(), 10 unit tests
- `ferro-cli/src/commands/mod.rs` — Added `pub mod ai_explain;`
- `ferro-cli/src/main.rs` — Added `AiExplain` clap variant + dispatch arm (both `#[cfg(feature = "projections")]`)

## Decisions Made

- `resolve_kind_priority` is a pure function (bool inputs, no introspection) used directly by unit tests; `resolve_target` calls the real tools in the same order, preserving testability without mocking the MCP layer.
- Dry-run skips `AiConfig::from_env()` validation — assembling and printing the prompt is valid without credentials; the user sees the prompt even if no LLM is configured.
- Lifetime elision applied: `resolve_kind_priority<'a>` → `resolve_kind_priority` after clippy `needless_lifetimes` lint fired with `-D warnings`.
- `resolve_max_tokens_with_default` is a new parameterized variant of Plan 02's `resolve_max_tokens` (which hardcodes 8192); the parameterized version avoids duplicating env-var reading logic while allowing ai:explain's 2048 default to differ from ai:make's 8192.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Explicit lifetime on resolve_kind_priority triggered clippy::needless_lifetimes**
- **Found during:** clippy -D warnings run after initial implementation
- **Issue:** `fn resolve_kind_priority<'a>(..., type_override: Option<&'a str>) -> &'a str` — explicit lifetime where Rust can infer it
- **Fix:** Elided lifetime to `fn resolve_kind_priority(..., type_override: Option<&str>) -> &str`
- **Files modified:** ferro-cli/src/commands/ai_explain.rs
- **Committed in:** 8a83d266

**2. [Rule 1 - Bug] Test imports included unused `FieldExplanation`/`ModelExplanation`**
- **Found during:** compilation warning pass
- **Issue:** Test `use` block imported `FieldExplanation` and `ModelExplanation` but neither was used in any test body
- **Fix:** Removed unused imports from test module
- **Files modified:** ferro-cli/src/commands/ai_explain.rs
- **Committed in:** 8a83d266

**3. [Rule 1 - Bug] Comments containing literal grep targets (`derive_intents`, `complete_with::<String>`) failed acceptance grep checks**
- **Found during:** acceptance criteria checks (the plan specifies `grep -q 'derive_intents'` must return exit 1)
- **Issue:** Doc comments used the exact strings as examples of what NOT to do, making the negative-grep acceptance checks fail
- **Fix:** Replaced comment text with equivalent descriptions that don't contain the banned strings
- **Files modified:** ferro-cli/src/commands/ai_explain.rs
- **Committed in:** 8a83d266

---

**Total deviations:** 3 auto-fixed (all Rule 1 — clippy lint, unused import, comment phrasing to satisfy acceptance grep gates)
**Impact on plan:** All fixes were minor. No scope change.

## Known Stubs

None — `run()` is fully wired. The LLM call is live (requires provider env vars); `--dry-run` is unconditional.

## Threat Surface Scan

All threats in the plan's `<threat_model>` are mitigated as implemented:

| Threat | Mitigation | Verified |
|--------|-----------|---------|
| T-171-EX-DoS token exhaustion | `FERRO_AI_MAX_TOKENS_PER_COMMAND` cap (default 2048) + single resolved target context | `explain_max_tokens_env_applied` test |
| T-171-EX-INFO prompt grounding | Prompt assembled only from ferro-mcp introspection facts for the resolved target (SC#6); ai:explain writes nothing to disk | `explain_service_prompt_contains_projection_vocabulary` verifies only introspected fields appear |
| T-171-EX-PI target in prompt | If no match, `ResolvedTarget::NotFound` — target never reaches LLM as free instruction | Resolution path verified by `explain_resolution_order_not_found` |

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `ferro-cli/src/commands/ai_explain.rs` | FOUND |
| Commit `8a83d266` | FOUND |
| `171-03-SUMMARY.md` | FOUND |
