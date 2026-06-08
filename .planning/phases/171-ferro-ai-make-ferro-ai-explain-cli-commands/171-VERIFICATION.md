---
phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands
verified: 2026-06-08T22:00:00Z
status: human_needed
score: 5/6 must-haves verified (1 requires human)
overrides_applied: 0
human_verification:
  - test: "Run: ferro ai:make \"track customer orders with pending/paid/shipped states\" --dry-run (with FERRO_AI_PROVIDER + key + model set, inside a ferro project)"
    expected: "Printed ServiceDef JSON references real FieldMeaning values (Status, Money, EntityName), includes a state_machine with pending/paid/shipped states, references model/field names that exist in the project, is a single ServiceDef — not a handler/route/migration bundle. With --dry-run, no files are written."
    why_human: "SC#6 / SC#2: output quality (ferro-consistency of generated ServiceDef referencing project-actual models) is non-deterministic and requires a live LLM provider plus a real ferro project to evaluate. No unit test can substitute."
  - test: "Run: ferro ai:explain <existing-service-name> (with a real projection in the project)"
    expected: "Prose is projection-framed: names the service Intents, fields whose FieldMeanings drive rendering, ActionDefs under GuardDefs, and StateMachine transitions if present. References only what the service actually defines — no invented fields or actions."
    why_human: "SC#4: projection-framed prose quality is a live-LLM output property. Automated tests prove the prompt is assembled correctly and the LLM is called — but whether the actual prose output is genuinely projection-framed (vs. generic code prose) requires human review of real LLM output."
---

# Phase 171: ferro ai:make & ferro ai:explain CLI Commands — Verification Report

**Phase Goal:** Ship the killer-feature CLI commands. `ferro ai:make <description>` produces a typed `ferro_projections::ServiceDef` (the universal projection contract) using live ferro-mcp introspection loaded in-process (not subprocess), context filtered to relevant items, via `ferro_ai::complete_with::<ServiceDef>()` through the Phase 166 ServiceDef-aware schema normalizer; single commit-ready ServiceDef artifact (NO multi-file scaffold, NO ScaffoldPlan intermediary); `--dry-run` prints without registering. `ferro ai:explain <route|model|service>` returns a projection-framed explanation (Intent/FieldMeaning/ActionDef+GuardDef/StateMachine) via in-process ferro-mcp; prose fallback only when no ServiceDef found. Both respect `FERRO_AI_MAX_TOKENS_PER_COMMAND` and `--dry-run`. Neither generates non-ferro code.
**Verified:** 2026-06-08T22:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC#1 | `ferro ai:make` calls ferro-mcp functions in-process (list_routes, list_models, db_schema, generation_context, existing ServiceDefs); context filtered before prompt | VERIFIED | `ai_make.rs:493-541` — five in-process tool calls (`list_models`, `generation_context`, `list_projections` sync; `list_routes`, `database_schema` via `rt.block_on`). `relevance::select_relevant()` applied to candidates; `generation_context` prepended unconditionally. `INPUT_BUDGET_CHARS=8000` gate limits total context size. |
| SC#2 | `ferro ai:make` produces a typed ServiceDef via `complete_with::<ServiceDef>()` with ServiceDef-aware schema normalizer; single commit-ready file; `--dry-run` prints without registering | VERIFIED | `ai_make.rs:655` — `rt.block_on(ferro_ai::complete_with::<ServiceDef>(client.as_ref(), &prompt, CompleteOptions {...}))`. `render_output()` writes exactly one `src/projections/<snake>.rs` (or DryRun path). `service.validate()` called before write. `--dry-run` path returns `OutputResult::DryRun(json)` with no file write. Unit test `dry_run_no_file_write` passes. |
| SC#3 | `ferro ai:make` does NOT write a multi-file scaffold bundle | VERIFIED | Grep for `make_model`, `make_handler`, `make_route`, `make_migration`, `migrations/` in `ai_make.rs` returns no matches. `render_output()` writes only `src/projections/<snake>.rs` + `src/projections/mod.rs`. No handler/route/migration creation anywhere in the command. |
| SC#4 | `ferro ai:explain` resolves in-process; projection-framed when ServiceDef found; prose fallback otherwise — and actual live output quality is projection-framed | human_needed | Resolution: `resolve_target()` checks service → route → model in that order (lines 110-124); `build_service_prompt()` assembles framing from `ProjectionDetail` vocabulary (Intent hints, FieldMeaning, ActionDef, StateMachine strings — lines 141-207). `schema: None` raw completion confirmed (line 358). Unit test `explain_service_prompt_contains_projection_vocabulary` verifies prompt contains "Intent", "FieldMeaning", "Action", "StateMachine". Whether the LLM produces genuinely projection-framed prose requires human verification with a live provider. |
| SC#5 | Both commands respect `FERRO_AI_MAX_TOKENS_PER_COMMAND`; both support `--dry-run` | VERIFIED | `ai_make.rs:374-377` — `resolve_max_tokens()` reads `FERRO_AI_MAX_TOKENS_PER_COMMAND`, default 8192. `ai_explain.rs:267-272` — `resolve_max_tokens_with_default(2048)`. Both `--dry-run` paths confirmed: `ai:make` early-returns with `OutputResult::DryRun`; `ai:explain` line 336 returns before LLM call. Tests `max_tokens_env_applied` and `explain_max_tokens_env_applied` pass. Clap registration confirms `--dry-run` flag on both commands (smoke-check evidence in 171-04-SUMMARY). |
| SC#6 | Neither command generates non-ferro code; produced ServiceDef references project introspection, not generic templates — verifiable as structural guarantee; output quality requires human | human_needed | Structural: `emit_service_def_source()` emits only `ferro::{ ServiceDef, DataType, FieldMeaning, ... }` builder chains — no handler/route/migration code. System prompt instructs "reference ONLY the supplied facts" and description is wrapped in `<description>...</description>` with `sanitize_description()` stripping injection attempts. `complete_with::<ServiceDef>()` schema constrains LLM output to valid ServiceDef shapes via the Phase 166 normalizer. Whether a real LLM invocation actually produces project-grounded output (not generic templates) requires live human verification. |

**Score: 5/6** (SC#4 and SC#6 share the same human_needed item — both are counted once in score; SC#4 is the prose quality check, SC#6 is the ServiceDef grounding check. Two human verification items recorded.)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-ai/src/complete.rs` | `CompleteOptions` struct + `complete_with::<T>()` + `complete::<T>()` delegate | VERIFIED | Lines 49-128 — `CompleteOptions` struct with `Default` (max_tokens: 4096, system: None, model_override: None); `complete_with()` routes through `schema::for_structured_output`; `complete()` is one-line delegate `complete_with(client, prompt, CompleteOptions::default()).await`. Three unit tests pass. |
| `ferro-ai/src/lib.rs` | Crate-root re-export of `complete_with` + `CompleteOptions` | VERIFIED | Line 65: `pub use complete::{complete, complete_with, CompleteOptions};` |
| `ferro-cli/src/naming.rs` | `pub(crate) fn is_valid_identifier` + `to_snake_case` | VERIFIED | Both functions present; rejects path traversal, `../../etc/passwd`, path separators; converts PascalCase to snake_case. Unit tests pass. |
| `ferro-cli/src/relevance.rs` | `pub(crate) fn tokenize` + `select_relevant` + `Candidate` struct + `INPUT_BUDGET_CHARS` | VERIFIED | All present — `tokenize()` splits on whitespace, `_`, and CamelCase transitions; `select_relevant()` scores by set intersection, sorts by (score desc, tier desc), gates on `INPUT_BUDGET_CHARS=8000`. |
| `ferro-cli/src/commands/ai_make.rs` | Full `ai:make` command with emitter, introspection, relevance filter, `complete_with::<ServiceDef>`, dry-run, sanitization | VERIFIED | `emit_service_def_source()` with explicit match arms for all DataType/FieldMeaning/Cardinality/Intent variants; `run()` fully wired; `render_output()`, `resolve_projection_path()`, `sanitize_description()`, `resolve_max_tokens()`, `ai_config_error_message()` factored and testable. 14+ unit tests. |
| `ferro-cli/src/commands/ai_explain.rs` | Full `ai:explain` command with service→route→model resolution, projection-framed prompt, raw schema:None completion, dry-run | VERIFIED | `resolve_target()` service-first; `build_service_prompt()` uses `ProjectionDetail` strings; `run()` with raw `CompletionRequest { schema: None }`; `resolve_max_tokens_with_default(2048)`. 10 unit tests. |
| `ferro-cli/src/main.rs` | `AiMake` + `AiExplain` clap variants + dispatch arms (cfg-gated) | VERIFIED | Lines 267-279: both variants declared with `#[cfg(feature = "projections")]`. Lines 663-675: dispatch arms call `commands::ai_make::run` and `commands::ai_explain::run`. `--help` smoke checks confirmed (171-04-SUMMARY). |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ferro-ai/src/complete.rs::complete` | `complete_with` | `complete_with(client, prompt, CompleteOptions::default()).await` | WIRED | Line 127 — exact delegation pattern confirmed. Unit test `complete_delegates_to_complete_with` asserts default values forwarded. |
| `ferro-ai/src/complete.rs::complete_with` | `schema::for_structured_output` | Schema normalization before request build | WIRED | Line 89 — `let normalized = schema::for_structured_output(raw_schema);` present in `complete_with`. ServiceDef-aware normalizer path preserved (unchanged from Phase 166). |
| `ferro-cli/src/commands/ai_make.rs` | `ferro_mcp::tools::{list_routes,list_models,database_schema,generation_context,list_projections}` | In-process execute() calls | WIRED | Lines 493-541 — all five tool calls present; async ones via `rt.block_on`. DB-down graceful via `unwrap_or_else`. |
| `ferro-cli/src/commands/ai_make.rs` | `ferro_ai::complete_with::<ServiceDef>` | Typed completion with CompleteOptions | WIRED | Line 655 — `rt.block_on(ferro_ai::complete_with::<ServiceDef>(client.as_ref(), &prompt, ferro_ai::CompleteOptions { max_tokens, system: Some(system_prompt), model_override: None }))` |
| `ferro-cli/src/commands/ai_make.rs` | `src/projections/<snake>.rs` + `mod.rs` | `fs::write` + `update_mod_file` (reused) | WIRED | Lines 438-458 — `emit_service_def_source()` + `fs::write` + `crate::commands::make_projection::update_mod_file()` called in `render_output()`. |
| `ferro-cli/src/main.rs::AiMake` | `commands::ai_make::run` | Dispatch match arm | WIRED | Line 667 — `commands::ai_make::run(description, dry_run)` in `#[cfg(feature = "projections")]` block. |
| `ferro-cli/src/commands/ai_explain.rs` | `ferro_mcp::tools::{inspect_projection,explain_route,explain_model}` | In-process resolution, service-first | WIRED | Lines 87-124 — `inspect_projection::execute` first, then `explain_route::execute`, then `explain_model::execute`. Service-first order confirmed. |
| `ferro-cli/src/commands/ai_explain.rs` | `client.complete(CompletionRequest{ schema: None })` | Raw prose completion | WIRED | Lines 349-361 — `CompletionRequest { schema: None, ... }`. No `complete_with::<String>` anywhere in the file (grep confirms zero matches). |
| `ferro-cli/src/main.rs::AiExplain` | `commands::ai_explain::run` | Dispatch match arm | WIRED | Line 675 — `commands::ai_explain::run(target, r#type, dry_run)`. |

---

### Data-Flow Trace (Level 4)

Both CLI commands are not UI components — they are command-line entry points producing console output or file writes, not components rendering from a data store. Level 4 data-flow trace is not applicable in the traditional "renders dynamic data from DB" sense.

The relevant data-flow analogs verified statically:

| Flow | Source | Produces Real Data | Status |
|------|--------|--------------------|--------|
| `ai:make` introspection → prompt | 5 ferro-mcp tool calls (list_routes, list_models, db_schema, generation_context, list_projections) | Yes — live project data via in-process MCP | FLOWING (by construction; graceful fallback for DB-down) |
| `ai:make` LLM → ServiceDef | `complete_with::<ServiceDef>()` via provider | Yes — typed ServiceDef from LLM; validated before write | FLOWING (live quality: human_needed) |
| `ai:explain` introspection → prompt | `inspect_projection::execute` → `ProjectionDetail` parsed strings | Yes — project projection vocabulary | FLOWING |
| `ai:explain` LLM → prose | `client.complete(req { schema: None })` | Yes — unstructured prose; printed to stdout | FLOWING (live quality: human_needed) |

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `ai:make --help` lists `--dry-run` | `cargo run -p ferro-cli -- ai:make --help` | Output contains `--dry-run` (171-04-SUMMARY confirmed) | PASS |
| `ai:explain --help` lists `--dry-run` and `--type` | `cargo run -p ferro-cli -- ai:explain --help` | Output contains `--dry-run` and `--type` (171-04-SUMMARY confirmed) | PASS |
| `complete_with` forwards max_tokens | Unit test `complete_with_uses_provided_max_tokens` | 95 ferro-ai tests pass (171-04-SUMMARY) | PASS |
| `ai:make` dry-run writes no file | Unit test `dry_run_no_file_write` | 554 ferro-cli tests pass (550 pre-review + 4 fixes) | PASS |
| Path traversal rejected | Unit test `ai_make_rejects_path_traversal` | Included in 554 passing ferro-cli tests | PASS |
| `ai:explain` dry-run makes no LLM call | Unit test `explain_dry_run_no_llm_call` | Included in passing test suite | PASS |
| Full CI gate | `cargo fmt + clippy -D warnings + test --all-features` | All pass: 95 ferro-ai + 550 ferro-cli + 46 ferro-json-ui + 27 ferro-mcp + 25 framework (171-04-SUMMARY) | PASS |

---

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|--------------|-------------|--------|----------|
| AICLI-01 | Plans 02, 04 | `ferro ai:make` produces typed ServiceDef from NL; in-process ferro-mcp; single file output; not a scaffold bundle | SATISFIED | `ai_make.rs` fully wired; 5 MCP tool calls in-process; `complete_with::<ServiceDef>()`; single file write; no scaffold paths |
| AICLI-02 | Plans 01, 02, 04 | `ai:make` uses structured outputs (AISDK-02) directly into ServiceDef; no ScaffoldPlan; ServiceDef-aware schema normalizer | SATISFIED | `complete_with` routes through `schema::for_structured_output` (Phase 166 path). No `ScaffoldPlan` type anywhere in ai_make.rs |
| AICLI-03 | Plans 03, 04 | `ferro ai:explain` with projection-framed primary path; prose fallback; ferro-mcp context | SATISFIED (structural) / human_needed (quality) | Resolution order service→route→model confirmed; projection-framed prompt built from `ProjectionDetail` strings; raw `schema:None` completion; live prose quality awaits human verification |

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact | Notes |
|------|---------|----------|--------|-------|
| `ferro-cli/src/commands/ai_explain.rs:291-295` (WR-03) | Misleading doc comment was present — `run` comment claimed dry-run validates config but code did not | Previously: Warning | Fixed in commit `03024580` — comment now accurately states "In dry-run mode, AI config is NOT checked" | RESOLVED |
| `ferro-cli/src/commands/ai_make.rs:27-29` (WR-01) | Function name in generated source used raw LLM-controlled service name (PascalCase leak) | Previously: Warning | Fixed in commit `8a209714` — `emit_service_def_source` now derives `fn_name` from `to_snake_case(name)`. Test `emitter_pascal_case_name_produces_snake_case_function` verifies | RESOLVED |
| `ferro-cli/src/commands/ai_make.rs` + `ai_explain.rs` (WR-02) | Module-local `ENV_LOCK` instances per test module — could allow env-var test races | Previously: Warning | Fixed in commit `03024580` — single `pub(crate) static ENV_LOCK` in `commands/mod.rs`; both test modules use `crate::commands::ENV_LOCK` | RESOLVED |
| `ferro-cli/src/commands/ai_make.rs` (IN-01) | `</description>` in user input could close XML delimiter early | Previously: Info | Fixed in commit `8a209714` — `sanitize_description()` helper strips `</description>` and `<description>` patterns. Three unit tests added | RESOLVED |
| `ferro-cli/src/commands/make_projection.rs` (IN-03) | `FieldMeaning::Custom` emitter used `{}` instead of `{:?}` | Previously: Info | Fixed in commit `6db83412` — aligned to `{:?}` debug formatting | RESOLVED |
| `ferro-cli/src/naming.rs` (IN-02) | `to_snake_case` does not handle hyphens; hyphenated names silently rejected by `is_valid_identifier` | Info | Skipped per REVIEW-FIX.md instructions — current behavior (rejection) is safe; no user path to exploit | DEFERRED (safe) |

No remaining blockers or warnings. All review findings that were in scope were fixed; one info finding was explicitly skipped (IN-02) as safe.

---

### Human Verification Required

#### 1. Live ai:make quality check (SC#2, SC#6)

**Test:** In a ferro project directory with at least one model, configure a real LLM provider and run:

```
export FERRO_AI_PROVIDER=<anthropic|openai|ollama>
export FERRO_AI_API_KEY=<key>
export FERRO_AI_MODEL=<model>

ferro ai:make "track customer orders with pending/paid/shipped states" --dry-run
```

**Expected:**
- Printed ServiceDef JSON uses real `FieldMeaning` values (e.g. Status, Money, EntityName) — not invented strings
- Includes a `state_machine` with pending/paid/shipped states
- References model/field names that actually exist in the sample project (not generic templates)
- Is a single ServiceDef — no handler/model/route/migration sections appear in the output
- (Optional) Re-run without `--dry-run` and confirm only `src/projections/<name>.rs` is written, no other files

**Why human:** SC#6 — whether the generated ServiceDef references actual project models (not generic templates) is a live-LLM output quality property that cannot be asserted by a unit test. The structural guarantees (schema normalization, system prompt, introspection pipeline) are verified; the semantic grounding requires real LLM output against a real project.

#### 2. Live ai:explain quality check (SC#4)

**Test:** With a real projection registered in a ferro project, run:

```
ferro ai:explain <service-name>
```

**Expected:** The prose output is projection-framed:
- Names the service's Intents (Browse, Focus, Collect, Process, Summarize, Analyze, Track)
- Identifies which fields' FieldMeanings drive rendering
- Describes the ActionDefs and which GuardDefs they sit under
- Describes StateMachine transitions if present
- References only what the service actually defines — no invented fields or behaviours

**Why human:** SC#4 — whether the LLM produces genuinely projection-framed prose (vs. generic code-level description) requires reading actual LLM output against a known projection. The prompt assembly is verified (unit tests confirm projection vocabulary is in the prompt); the LLM's adherence to the projection framing can only be assessed from the live output.

---

## Gaps Summary

No automated gaps found. All structural must-haves are verified: the SDK extension (`complete_with`/`CompleteOptions`) is implemented and wired, both CLI commands are fully implemented with in-process ferro-mcp introspection, the ServiceDef emitter uses correct explicit match arms for all type variants, the single-file output constraint is enforced, path sanitization and dry-run are tested, the CI gate passed at 550+ tests with zero failures, and all code review findings (WR-01, WR-02, WR-03, IN-01, IN-03) were fixed.

Two human verification items remain open (live LLM quality checks SC#4 and SC#6) that cannot be resolved autonomously. These are expected given the phase explicitly records them as PENDING in 171-04-SUMMARY.md.

---

_Verified: 2026-06-08T22:00:00Z_
_Verifier: Claude (gsd-verifier)_
