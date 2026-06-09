# Project Research Summary

**Project:** ferro — v12.5 Projection Checkpoint (`checkpoint_projection` MCP tool)
**Domain:** Agent-facing verification tool / ferro-mcp introspection layer
**Researched:** 2026-06-09
**Confidence:** HIGH

## Executive Summary

The `checkpoint_projection` tool closes the one gap in ferro's generate→verify loop that no existing tool covers: cross-artifact seam coherence anchored on a projection. Individual validators (`validate_projection`, `json_ui_verify_action`, `validate_contracts`) operate per-artifact; an agent authoring a full intent slice must today orchestrate all of them in the correct order, aggregate their results, and know which seam each applies to. The canonical failure (v12.0 friction F11: `PageHeader.children` silent drop) is a seam failure that produces a 200 response with missing content and zero diagnostics — exactly the class of defect this milestone exists to prevent.

The recommended approach is purely additive inside `ferro-mcp`, with zero new dependencies. Seam 2 (field→column) is the only genuinely new check — everything else is a thin dispatch over existing validators. The implementation follows a strict orchestrator pattern: `checkpoint_projection` owns seam 2 plus aggregation; all other seams delegate to the tool that already owns that check. No validation logic is reimplemented inside the checkpoint. This boundary is both a correctness constraint and a conceptual coherence rule — duplicating check logic across the codebase creates two sources of truth and erodes the principle that each seam is owned by exactly one place.

The load-bearing trust invariant is that `not_checked` must never collapse to `pass`. One false `pass` on an unchecked seam destroys agent confidence in all future results. The field→column false-positive risk (relationship fields, computed fields flagged as missing columns) is the primary quality threat in P1 — a false positive rate above zero in any non-trivially complex projection trains agents to ignore findings and buries real defects in noise. The dogfood gate (poisoned fixture that must fire, live consumer that must produce at least one real finding) is the mechanism that prevents shipping a tool that always reports clean.

## Key Findings

### Recommended Stack

Zero new dependencies required. All primitives needed for `checkpoint_projection` already exist in the `ferro-mcp` crate graph: `syn` + `walkdir` for entity parsing, `serde`/`serde_json` for verdict serialization, `sea-orm` for the optional live-schema branch, `regex` for projection source parsing, and `tokio` for the async boundary in seam 3. The only new file is `ferro-mcp/src/tools/checkpoint_projection.rs`.

**Core primitives and where they live:**
- `list_models::execute(project_root)` — entity field extraction via `syn`; returns `Vec<ModelDetails>` where `FieldInfo.name` is the snake_case column name. Primary source for seam 2's column set.
- `projection_coverage.rs:76-79` predicate — `p.service_name.as_ref().is_some_and(|sn| sn.to_lowercase() == model_lower)` — copy verbatim for projection→model name resolution.
- `render_projection::reconstruct_service_def(service_name, display_name, content)` — `pub(crate)` canonical entry point for `ServiceDef` from source. Must be reused, not reimplemented.
- `json_ui_verify_action::find_handler` — `pub(crate)` pure function; fetch routes once via `list_routes::execute`, call `find_handler` per action to avoid repeated I/O.
- `validate_projection::execute_single`, `render_projection::execute`, `json_ui_validate_spec::execute`, `validate_contracts::execute` — existing validator entry points for seams 1, 4, 4b, 5.
- `database_schema::execute` — optional secondary column source when `DATABASE_URL` available; strengthening pass, not prerequisite.

**Do not add:** SeaORM reflection crates, migration AST parsers, diff/comparison libraries, ordered map crates, graph libraries. Column membership is `HashSet::contains`.

### Expected Features

**Must have (table stakes) — all P1:**
- Single-call entry point `checkpoint_projection { name: "Booking" }` anchored on projection name.
- Top-level `status: pass | warn | fail` for one-token agent branching.
- Per-seam `status` with distinct `not_checked` variant — the trust foundation. Never collapses to `pass`.
- Seam 2 (field→column, presence-only) — the only check no existing tool performs; primary new value.
- Ranked `next_steps`: failures rank above warnings; earlier seams before later within a rank; capped at 10 for P1, 5 by P3.
- `source` provenance per finding — identifies the producing validator.
- `subject` + `detail` + `fix` per finding — LSP-shaped minimum; `fix` on seam 2 findings is checkpoint-owned.

**Should have (differentiators) — P2:**
- Inline return from `generate_projection` / `json_ui_generate` in summary format (not full seam breakdown) — closes the loop by default.
- Wrapper seams 1, 3, 4, 5 — convenience dispatch; enables upstream-failure cascade.
- Ambient status (`unverified` / `failing` / `clean`) in `projection_coverage` and `application_info` via `last_status` reading `.ferro/checkpoints/{name}.json`.
- Seam failure → downstream `not_checked` with `reason: "seam_N_failed"` (distinct from absent-prerequisite `not_checked`).

**Defer (v2+):**
- Type compatibility checking in seam 2 — presence-only in P1; type mismatch as `warn` in P2 at earliest.
- Semantic deduplication of `next_steps` — exact-string-match sufficient at launch.
- Model-anchored fan-out (checkpoint every projection touching a given model).
- `cargo check` integration — breaks the read-only fast contract; remains a separate agent step.

### Architecture Approach

The checkpoint lives entirely within `ferro-mcp` as an orchestrator module. Dependency direction is fixed: `checkpoint_projection` depends on existing validator tools; generators and coverage tools depend on `checkpoint_projection` in P2; no reverse dependencies at any phase. The only write side effect is `.ferro/checkpoints/{name}.json` (runtime artifact, gitignored). Async boundary is managed by preferring `find_handler` (synchronous `pub(crate)`) over the async `json_ui_verify_action::execute` wrapper, keeping `run_for` synchronous-compatible.

**Major components:**
1. `checkpoint_projection::run_for(project_root, name)` — spine orchestrator; calls all seam functions, aggregates verdict, writes status cache.
2. `check_field_to_column(project_root, &service_def)` — private fn; the only new logic; uses `list_models::execute` + `reconstruct_service_def`.
3. `CheckpointVerdict` / `SeamResult` / `Finding` / `SeamStatus` / `CheckpointStatus` — plain `#[derive(Serialize)]` types; zero external dependency.
4. `last_status(project_root, name)` — reads `.ferro/checkpoints/{name}.json`; returns `Unverified` when absent.
5. Inline hooks in `generate_projection` and `json_ui_generate` — post-generation call to `run_for`; embed as `checkpoint: Option<CheckpointVerdict>` (P2).
6. `projection_coverage::ModelCoverage.checkpoint_status` and `ApplicationInfo::ProjectionCheckpointSummary` — read-only ambient consumers of `last_status` (P2).

### Critical Pitfalls

1. **`not_checked` collapsed to `pass`** — the trust-destroying invariant violation. Four-variant `SeamStatus` enum required; prerequisite-absent paths must explicitly return `not_checked`; aggregation excludes unchecked seams from `pass` determination. P1 unit test required: unresolvable model source → seam 2 is exactly `not_checked`, not `pass`. Must fail before the guard is implemented.

2. **Field→column false positives on relationship and computed fields** — `ServiceDef.relationships` contains relationship navigation fields; they must never enter the field→column loop. FK fields check the FK column, not the relationship name. Computed/virtual fields (`FieldMeaning::Custom("virtual")`) are exempt. False-positive rate on the synthetic catalog must be zero for any projection with a relationship or read-only aggregate before P3 dogfood runs. A failing dogfood (false positive on a known-clean projection) is a blocker, not a warning.

3. **`reconstruct_service_def` completeness assertion missing** — regex-based reconstructor silently drops unrecognized builder patterns. P1 must count field-builder invocations in raw source (`.field(`, `.optional_field(`, `.read_only_field(`, `.write_only_field(`) and compare against `ServiceDef.fields.len()`. Discrepancy → `warn: reconstruction may be incomplete` on seam 2, never a silent clean result.

4. **Dogfood gate is vacuous without a poisoned fixture** — projections derived from `ServiceDef::from_model()` always pass seam 2. The synthetic catalog must include at least one projection with a deliberately wrong field name. Acceptance criterion: poisoned fixture produces `fail` on exactly that field. Live consumer (gestiscilo) must produce at least one real finding. This is a go/no-go gate for P3, not a nice-to-have.

5. **Seam reimplementation instead of delegation** — the checkpoint must import and call existing validator functions, not reimplement their logic. `source: "checkpoint"` is valid only on seam 2 findings. Code review gate: no route-parsing logic inside `checkpoint_projection.rs`.

## Implications for Roadmap

### Phase 1: Tool + Seam 2 + Aggregation

**Rationale:** Seam 2 is the only check that does not exist anywhere today. The trust invariant and field→column exemption logic must both be correct before any wrapper seams are added — a false positive at this layer propagates through all downstream seams. Delivers a callable, trustworthy standalone tool.

**Delivers:** `checkpoint_projection` tool registered in the MCP dispatcher; seam 2 (field→column, presence-only); aggregation with ranked `next_steps`; `not_checked` coverage honesty; `last_status` / status cache write; all verdict types.

**Addresses:** Single-call entry point, `not_checked` invariant, seam 2 (primary new value), ranked `next_steps`, provenance, `fix` strings on seam 2 findings, reconstruction completeness assertion.

**Avoids:** Pitfall 1 (false confidence), Pitfall 2 (false positives on relationships), Pitfall 3 (false negatives), Pitfall 4 (type-mismatch scope creep — presence-only scope in tool description), Pitfall 6 (seam reimplementation).

**Spec gaps that must be resolved before P1 implementation begins:**
- Seam failure → downstream `not_checked` propagation rule must be stated explicitly (spec implies but does not define): `reason: "seam_N_failed"` distinct from absent-prerequisite.
- Reconstruction completeness assertion pattern list: specify exactly which builder call strings to count.
- Seam 2 model-resolver fallback on multi-match ambiguity: `not_checked` with `reason: "ambiguous_model_match"`.
- Fix string format for seam 2 findings: standardize before implementation so tests can assert exact string shape.

**P1 unit tests required (all must exist before the guard code they test):**
- Poisoned fixture: dangling field → seam 2 `fail` naming exactly that field.
- Unresolvable model source → seam 2 `not_checked` (not `pass`).
- Projection with `belongs_to` relationship + computed field → zero seam 2 findings.
- Field-builder count mismatch → `warn: reconstruction may be incomplete`.
- Mixed-seam fixture: seam 2 `fail` + seam 1 `warn` → `next_steps[0]` is the seam 2 finding.

### Phase 2: Inline Hook + Ambient Status + Wrapper Seams

**Rationale:** Once the standalone tool is trusted (P1 unit tests pass), the inline hook and ambient status consumers can be wired. Freshness strategy must be decided before implementation. Fix string normalization must be designed before wrapper seams are activated.

**Delivers:** `generate_projection` / `json_ui_generate` embed checkpoint in summary format; `projection_coverage.checkpoint_status`; `ApplicationInfo::ProjectionCheckpointSummary`; wrapper seams 1, 3, 4, 4b, 5 activated; upstream-failure cascade `not_checked`.

**Avoids:** Pitfall 9 (inline verdict noise — summary format only, not full seam breakdown, immediately post-generation).

**Spec gaps to resolve before P2:**
- Ambient status freshness strategy: cached (stale risk) vs. fresh lightweight check (I/O cost). Two different implementation paths; cannot be deferred past the P2 plan.
- Fix string normalization: sub-validators return incompatible shapes (`fix_suggestions[].details`, `message`/`candidate`, `mismatches[].details`); define normalization layer or document explicit passthrough with caveat.
- Intent + confidence in seam 4 finding: decide inclusion or omission.
- Method threading in seam 3: whether `ActionDef` carries HTTP method to thread through `find_handler` filter.

### Phase 3: Dogfood Gate + Hardening

**Rationale:** Wrapper seams earn their place against real defects. A seam that finds nothing in real apps ships as `not_checked` rather than active. Poisoned fixture must be written before the acceptance run.

**Delivers:** Dogfood acceptance against gestiscilo + synthetic catalog; `next_steps` capped to 5; seams conditionally demoted to `not_checked` if no real defect found; dedup stress testing.

**Acceptance criterion:** Poisoned fixture produces `fail` on exactly the planted field. Live consumer produces at least one finding (fail or warn). Any wrapper seam producing zero findings across all dogfood inputs is demoted to `not_checked` rather than forced active.

### Phase Ordering Rationale

- P1 before P2: inline hook requires the tool to exist; ambient status requires the status cache; wrapper seams require the aggregation pattern proven first.
- Seam 2 before wrapper seams: the only new check must be trusted before adding dispatches. A false positive before exemption logic is correct amplifies when all five seams fire simultaneously.
- Type mismatch deferred: the `ColumnType` → `DataType` mapping is not invertible for all cases; attempting type verification in P1 before presence-check trust is established collapses confidence in both checks.
- Dogfood in P3: poisoned fixtures and live consumer runs require a stable tool; running them before P1 tests pass is circular.

### Research Flags

Phases needing spec resolution before planning:
- **P1 — Seam failure cascade rule:** must be stated explicitly in the plan as a testable invariant.
- **P1 — Reconstruction completeness pattern list:** builder call strings to count must be enumerated from the `ServiceDef` source before implementation.
- **P2 — Ambient status freshness strategy:** two implementation paths; must be decided before any P2 code.
- **P2 — Fix string normalization:** sub-validator output shapes must be audited and a normalization layer designed before wrapper seam implementations.

Phases with well-established patterns (no additional research needed):
- **P1 — `not_checked` enum design:** trivial; the pattern and test shape are fully specified.
- **P1 — Seam 2 column set:** `list_models::execute` already returns the right data; predicate is a verbatim copy.
- **P2 — Inline hook wiring:** thin post-generate call following the `application_info` aggregator pattern exactly.
- **P3 — Dogfood fixture authoring:** write one projection file with a field name not matching any entity column; mechanical.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All primitives verified by direct source read; Cargo.toml confirmed; no dependency gaps |
| Features | HIGH | Design spec read directly; all validator source files read; LSP + MCP tool design literature corroborates |
| Architecture | HIGH | All tool modules read directly; dispatch pattern observed across multiple existing tools; dependency direction verified |
| Pitfalls | HIGH | Grounded in existing codebase, design spec, real friction data (F11–F14), and regex-based `reconstruct_service_def` implementation |

**Overall confidence:** HIGH

### Gaps to Address

- **Seam failure → downstream cascade specification:** spec implies but does not state. Must be written as a testable rule in the P1 plan.
- **Ambient status freshness strategy (P2):** cached vs. fresh. Two implementation paths. Cannot be deferred past P2 plan.
- **Fix string normalization for seams 1/3/4/5 (P2):** incompatible sub-validator result shapes. Audit + normalization design required before wrapper seam implementations.
- **Method threading in seam 3:** low impact (false negative only on handler name collision across HTTP methods), but must be decided before seam 3 implementation.
- **Inline verdict format (P2):** summary key vs. full `CheckpointVerdict` with empty seam arrays; must be specified to avoid the agent noise pitfall.

## Sources

### Primary (HIGH confidence)
- `ferro-mcp/src/tools/list_models.rs` — entity field extraction via `syn`
- `ferro-mcp/src/tools/projection_coverage.rs` — model↔projection name-match predicate (lines 76–79)
- `ferro-mcp/src/tools/render_projection.rs` — `reconstruct_service_def` entry point
- `ferro-mcp/src/tools/validate_projection.rs` — seam 1 delegation target
- `ferro-mcp/src/tools/database_schema.rs` — live DB column query
- `ferro-mcp/Cargo.toml` — dependency versions confirmed
- `ferro-projections/src/service.rs` — `ServiceDef`, `FieldDef`, builder variants, `validate()`
- `docs/superpowers/specs/2026-06-09-projection-checkpoint-design.md` — design spec (approved)

### Secondary (MEDIUM confidence)
- LSP Specification 3.17 Diagnostic interface — shaped the four-status enum
- Agent-aware MCP 10 patterns (community research, 2025) — `next_actions` embedding, capability advertisement, confidence thresholds
- Schema drift detection patterns (dbt, data contracts, OpenAPI drift tools) — per-field finding with provenance and repair step

### Tertiary (LOW confidence)
- GitHub Security Lab Taskflow Agent checkpoint pattern — verification tool false positive/negative design tradeoffs

---
*Research completed: 2026-06-09*
*Ready for roadmap: yes*
