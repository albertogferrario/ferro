---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
plan: 03
subsystem: ui
tags: [json-ui, resolve, iteration, directives, killer-feature, ferro-json-ui]

# Dependency graph
requires:
  - phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
    plan: 01
    provides: "EachDirective wire-format + Element.each field"
  - phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
    plan: 02
    provides: "Element.if_: Option<Visibility> field"
provides:
  - "pub fn expand_directives(spec: &mut Spec) in ferro_json_ui::resolve"
  - "Resolve-time materialization of $each into N clones with auto-suffixed IDs"
  - "Resolve-time removal of $if-falsy elements from Spec.elements"
  - "Correlated child indexing for sibling templates over the same {path, as}"
  - "Idempotent expansion (clones have each/if_ stripped; second pass is a no-op)"
  - "JsonUi::resolve pipeline wires expand_directives BEFORE resolve_actions"
  - "Public re-export ferro_json_ui::expand_directives for framework/downstream"
affects:
  - 163-04
  - 163-05+ (any subsequent plan that consumes JsonUi::render with $each/$if specs)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Three-pass directive expansion: (1) $if removal → (2) $each clone insertion → (3) parent children rewriting. Each pass is a small focused function with one responsibility."
    - "Snapshot-then-mutate iteration: templates are collected into a Vec before any HashMap mutation so sibling lookup during correlated-child rewriting reads pre-expansion state."
    - "Row-scoped path resolution at clone time: /{as}/X $data markers and {/{as}/X} $template placeholders are pre-resolved against the current row during expansion, so resolve_expressions downstream sees literal values, not loop-variable references."
    - "D-04 honored: zero parallel evaluator helpers in resolve.rs (grep confirmed); the single .evaluate( call delegates to Visibility::evaluate which already handles Condition/And/Or/Not semantics."
    - "Idempotency via field stripping: every clone has each=None / if_=None set before insertion, so a second expand_directives pass is structurally a no-op (verified by snapshot equality test)."

key-files:
  created: []
  modified:
    - "ferro-json-ui/src/resolve.rs"
    - "ferro-json-ui/src/lib.rs"
    - "framework/src/json_ui/mod.rs"

key-decisions:
  - "$if evaluated FIRST when co-occurring with $each on the same element. Locks planner decision #5. Implementation: remove_if_falsy runs before expand_each; a falsy $if removes the template before clones can be produced."
  - "Correlated child indexing: when a templated element's child also has $each over the same {path, as}, the i-th parent clone references the i-th child clone. Implementation: snapshot template directives BEFORE mutation, then per-clone child rewriting compares directives for matching {path, as}."
  - "Non-matching child each directives are LEFT LITERAL at runtime. The validator in Plan 04 will reject this statically as SpecError::MismatchedEach. Plan 03's resolver does not fail at runtime — it produces a clone whose children list references a non-existent ID, which the render layer renders as a missing-id comment."
  - "Empty / missing / non-array $each.path resolves to an empty Vec → zero clones. The template is still removed from elements; the parent's children list still loses the templated ID. Plan 04's validator will catch non-array statically; this is the runtime fallback."
  - "Pre-resolved /{as}/... paths produce literal values BEFORE resolve_expressions runs. resolve_expressions is single-pass and would not re-resolve inside a previously-resolved $data result, so paths must be inlined at $each expansion time."
  - "$template markers with /{as}/X placeholders are interpolated inline during $each expansion; non-row {/...} markers are LEFT IN the $template payload for downstream resolve_expressions. This preserves the single-pass invariant of expression.rs while making row-scoped templates work."
  - "Visibility::evaluate is the SOLE predicate evaluator. No fn evaluate_if / fn check_if_predicate / fn if_predicate_eval helpers added; the negative grep returns 0. The single .evaluate( call appears in remove_if_falsy."
  - "Both JsonUi::resolve and JsonUi::resolve_with_errors call expand_directives FIRST. The two pipelines stay symmetric so render_with_errors handlers benefit from $each/$if just like render handlers."

patterns-established:
  - "Three-pass directive expansion (remove $if-falsy → expand $each → rewrite parent children) — extensible to future directives by appending a 4th pass."
  - "Snapshot-then-mutate for HashMap iteration when the mutation depends on sibling state — used by expand_each to look up sibling directives while inserting new entries."
  - "Idempotent clone construction — every clone has its directive fields cleared at insertion time, so the same expansion can be run multiple times on the same spec without further effect (verified by snapshot-equality test)."
  - "Pipeline ordering is documented at the JsonUi::resolve call site, not just in the implementation crate. The framework's mod.rs comments name each pass explicitly so future maintainers see the ordering at the consumption point."

requirements-completed: []

# Metrics
duration: 9min
completed: 2026-05-16
---

# Phase 163 Plan 03: `expand_directives` resolve-time pass for `$each` / `$if` — Summary

**Ships the killer feature for Phase 163: a single resolve-time pass that materializes `$each` directives into N concrete elements with auto-suffixed IDs and removes `$if`-falsy elements from the spec map entirely. Wired BEFORE `resolve_actions` / `resolve_expressions` in `JsonUi::resolve` so all downstream resolution operates on the expanded element set. The wire-format types from Plans 01 + 02 are no longer inert — they now do something.**

## Performance

- **Duration:** ~9 min
- **Started:** 2026-05-16T21:03:47Z
- **Completed:** 2026-05-16T21:12:55Z
- **Tasks:** 1 (TDD)
- **Files modified:** 3

## Accomplishments

- Added `pub fn expand_directives(spec: &mut Spec)` to `ferro-json-ui/src/resolve.rs` with full rustdoc covering pass order, `$if`-vs-`$each` precedence, correlated child indexing rule, idempotency contract, and pipeline-position requirement.
- Decomposed the pass into three focused sub-passes: `remove_if_falsy` → `expand_each` → `rewrite_parent_children`. Plus three private helpers for row-scoped expression resolution (`inline_resolve_row_paths`, `inline_walk`, `interpolate_row_template`) and two micro-helpers (`contains_template_marker`, `value_to_string`).
- Honored the D-04 reuse mandate: `Visibility::evaluate` is the sole predicate evaluator (zero parallel `fn evaluate_if`/`fn check_if_predicate`/`fn if_predicate_eval` helpers — negative grep verified).
- Re-exported `expand_directives` from `ferro-json-ui/src/lib.rs` so the framework crate (and any downstream consumer) can call it.
- Wired the new pass into BOTH `JsonUi::resolve` AND `JsonUi::resolve_with_errors` in `framework/src/json_ui/mod.rs`. Pipeline ordering is now: `expand_directives` → `resolve_actions` → `resolve_expressions` (and then `resolve_errors` for the error variant). Documented the ordering in rustdoc at the call site.
- Added 12 inline tests covering every behavior in `must_haves.truths` plus an idempotency snapshot test:
  1. `expand_if_falsy_deletes_element`
  2. `expand_if_truthy_retains_element` (verifies `if_` stripped post-expansion)
  3. `expand_if_uses_visibility_evaluate` (compound `And` predicate)
  4. `expand_each_produces_n_elements`
  5. `expand_each_auto_suffixes_ids` (verifies `each` / `if_` stripped)
  6. `expand_each_pre_resolves_row_paths`
  7. `expand_each_correlates_child_indexes` (sibling templates over same `{path, as}`)
  8. `expand_parent_children_rewritten_for_each`
  9. `expand_parent_children_pruned_for_if`
  10. `expand_if_first_then_each` (planner-locked ordering #5)
  11. `expand_each_empty_array_produces_zero_clones`
  12. `expand_directives_idempotent` (snapshot equality after re-run)

## Task Commits

1. **Task 1 RED — Failing tests for `expand_directives` (`test`)** — `b0bf708e`
2. **Task 1 GREEN — `expand_directives` implementation + lib re-export + framework wiring (`feat`)** — `6b35a3a5`

REFACTOR was evaluated and skipped — the diff is already minimal (one public function, five small private helpers, each with a single responsibility). No duplication arose between the sub-passes; `inline_walk` and `interpolate_row_template` deliberately mirror the structure of `expression.rs::resolve_value` / `substitute_template` (which is the right shape for a single-pass row-scoped resolver) but cannot be merged because the row-scoped path semantics differ from the global path semantics.

## Files Created/Modified

- `ferro-json-ui/src/resolve.rs` — Added `pub fn expand_directives` plus 5 private sub-pass helpers and 12 inline unit tests. New section header `// Section — Directive expansion (Phase 163: $each, $if)`.
- `ferro-json-ui/src/lib.rs` — Added `expand_directives` to the `pub use crate::resolve::{...}` re-export list, keeping alphabetical ordering against the existing four exports.
- `framework/src/json_ui/mod.rs` — Added `expand_directives` to the `use ferro_json_ui::{...}` import list. Wired `expand_directives(&mut resolved)` as the FIRST call in both `JsonUi::resolve` and `JsonUi::resolve_with_errors`. Updated both functions' rustdoc to document the Phase 163 pipeline ordering.

## Decisions Made

- **`$if` before `$each` co-occurrence ordering.** Planner-locked at decision #5. The implementation drops a falsy-`$if` template before `$each` cloning ever runs. Test `expand_if_first_then_each` pins the contract.
- **Correlated child indexing implementation strategy.** Snapshot the template directives map BEFORE any HashMap mutation, then per-clone use that snapshot to look up sibling each directives by ID. This keeps the `{path, as}` comparison stable even as new clones are inserted into `spec.elements`.
- **Non-matching child each directives are NOT a runtime error.** Plan 04's validator will catch this statically as `SpecError::MismatchedEach`. Plan 03's resolver leaves the literal ID in the clone's children; the render layer will emit a missing-id comment. This preserves the "validation runs before expansion" invariant from RESEARCH §"Pass invariants".
- **Row-scoped path resolution at clone time, not at `resolve_expressions` time.** `resolve_expressions` is single-pass (the inner-platform firewall from 118 D-07) and would not re-resolve the result of a previous `$data` substitution. By pre-resolving `/{as}/...` paths inside `expand_each`, the downstream pass sees literal values and respects its single-pass contract.
- **`$template` partial substitution.** A template that mixes `/{as}/...` placeholders with global `/...` placeholders has the row-scoped placeholders inlined immediately but the global placeholders LEFT in the `$template` payload for downstream `resolve_expressions`. `contains_template_marker` is the heuristic that decides whether to collapse the value to a `String` or keep it as a `{"$template": "..."}` object.
- **Both `JsonUi::resolve` AND `JsonUi::resolve_with_errors` get the new pass.** The plan called out only the `resolve` method; symmetry required updating `resolve_with_errors` too so the validation-error path benefits from `$each`/`$if` in the same way as the success path. This is technically a Rule 2 deviation (missing critical functionality if the symmetry breaks) but it is a trivial mirror — recorded below.
- **No return-type cascade.** The plan's `<key_architecture>` block discussed `JsonUi::resolve` possibly needing to become `Result<Spec, SpecError>`. After implementation review, the resolver pass is INFALLIBLE: `Visibility::evaluate` is infallible per `visibility.rs:55–59`, `data::resolve_path` returns `Option`, and non-array `$each.path` falls back to empty `Vec` (the validator in Plan 04 catches this statically, not at runtime). No `?` propagation is needed. The 5 public API methods keep their existing signatures.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Missing critical functionality] Wired `expand_directives` into `JsonUi::resolve_with_errors` as well**
- **Found during:** Task 1 GREEN — implementing the framework call site.
- **Issue:** The plan's `<action>` step 3 only mentions `JsonUi::resolve` (line 37–45 in `framework/src/json_ui/mod.rs`). But `JsonUi::resolve_with_errors` (line 191–197) is the parallel pipeline used by `render_with_errors` / `render_validation_error` / `render_json_validation_error` — the form-error rendering path. If `$each`/`$if` only worked in the success path, form-submission re-renders would not match the initial render shape, breaking field-error attachment on `$each`-cloned form fields.
- **Fix:** Added the same `expand_directives(&mut resolved)` call as the FIRST step in `resolve_with_errors`. Mirrors the contract in `resolve`. Documented the ordering in rustdoc.
- **Files modified:** `framework/src/json_ui/mod.rs` (one additional 3-line change inside `resolve_with_errors`).
- **Verification:** All 510 framework tests pass; including `render_with_errors_resolves_expressions_then_applies_errors` and `render_json_with_errors_returns_resolved_spec_with_errors` which exercise the error-rendering path with expression resolution.
- **Committed in:** `6b35a3a5` (Task 1 GREEN — bundled with the main change because the two `JsonUi::resolve*` methods are a coherent pipeline and splitting them across commits would have introduced a brief asymmetric state).

---

**Total deviations:** 1 auto-fixed (1 Rule 2 — symmetry between `resolve` and `resolve_with_errors`)
**Impact on plan:** Three additional lines + one rustdoc update in `framework/src/json_ui/mod.rs`. No scope creep — the addition is the natural symmetric pair of the planned change.

## Issues Encountered

- **Harness friction (documented in 163-02-SUMMARY.md):** The `Edit` tool produced "READ-BEFORE-EDIT" hook warnings AFTER successfully applying each edit. The warnings were spurious — each edit DID land on disk, verified by post-edit `grep` against the file. No actual no-ops occurred; the workaround documented in 163-02-SUMMARY.md (verify via `grep` / `cargo build`) confirmed every change made it through. Did not require any python-via-Bash fallbacks for this plan.
- **Workspace name surprise:** The framework crate's `[package].name` is `ferro-rs`, not `framework`. `cargo build -p framework` fails with "cannot specify features for packages outside of workspace". Corrected to `cargo build -p ferro-rs --all-features`. This is a name-vs-directory distinction (the crate lives at `framework/` but publishes as `ferro-rs`). Worth noting in case future plans inherit the same confusion.

## TDD Gate Compliance

- `test(163-03): …` commit at `b0bf708e` (RED gate — 12 new tests added; `cargo build -p ferro-json-ui --tests` failed with 13 × `E0425 cannot find function 'expand_directives' in this scope` as expected).
- `feat(163-03): …` commit at `6b35a3a5` (GREEN gate — same 12 tests now pass; full ferro-json-ui suite of 455 unit + 11 + 8 + 5 doctests green; framework suite of 510+ tests green).
- REFACTOR gate evaluated and intentionally skipped (no duplication or simplification opportunity in the minimal diff).

## User Setup Required

None.

## Next Phase Readiness

- **Plan 04 (validation):** With `expand_directives` operational, the validator can now check `$each` / `$if` statically AND has a clear runtime contract to document failure modes against. Specifically:
  - `SpecError::EachPathNotArray` — when `$each.path` resolves to a non-array (currently silently falls back to zero clones; the validator should reject this at parse time).
  - `SpecError::EachAsReservedName` — when `$each.as` collides with a reserved name (`data`, `root`, `_root`, `_each`, `this`, `self`).
  - `SpecError::MismatchedEach` — when a templated element's child has a `$each` with a different `{path, as}` (currently leaves the literal ID at runtime).
  - `SpecError::NestedEach` — when a templated element's descendants ALSO have `$each` (currently undefined runtime behavior; the validator should reject statically).
  - `SpecError::IfPathMissing` — distinguishable from `$if`-falsy when the predicate's path resolves to no value at all. (Visibility::evaluate already returns `false` for missing-path in most operators per `visibility.rs:55–59`, so this is an authoring-warning, not a runtime concern.)
- **Plan 05+ (consumer migration):** Consumer specs (cassa/orders kanban, cassa/products magazzino_links_rows, etc. from FRICTION) can now use `$each` end-to-end. The runtime contract matches the worked examples in `163-RESEARCH.md` §"Auto-suffix rule".

## Threat Flags

None. The threat register from 163-03-PLAN was fully addressed:

- **T-163-03-01 (DoS via huge data arrays):** Accept disposition unchanged. No hard cap imposed — same posture as the rest of the resolve pipeline. Documented in `expand_each` rustdoc and the original Plan 01 `EachDirective` rustdoc.
- **T-163-03-02 (path collision via auto-suffix):** Mitigation in place. Auto-suffix uses deterministic `format!("{tmpl_id}-{i}")` where `tmpl_id` is pre-validated by `validate_ids` (spec.rs:417). Collisions would surface as `DuplicateId` during the parse-time validation that runs BEFORE expansion. Plan 04 will add explicit validation against templates that collide with sibling literal IDs that would conflict post-expansion.
- **T-163-03-03 (predicate engine bypass):** Mitigation verified. `grep -c "fn evaluate_if\|fn check_if_predicate\|fn if_predicate_eval"` returns 0. The single `.evaluate(` call in `resolve.rs` is at the `remove_if_falsy` site and delegates to `Visibility::evaluate`.

No NEW threat surface introduced — `$each` clones inherit the trust posture of their template, and `$data` row-path resolution is bounded by `data::resolve_path` (which already gates against malicious paths via segment parsing).

## Self-Check: PASSED

- `ferro-json-ui/src/resolve.rs` — FOUND (contains `pub fn expand_directives`, 5 sub-pass helpers, 12 `expand_*` inline tests, all pass)
- `ferro-json-ui/src/lib.rs` — FOUND (contains `expand_directives` in the `pub use crate::resolve::{...}` re-export list)
- `framework/src/json_ui/mod.rs` — FOUND (contains `expand_directives` in the import list and as the FIRST call in both `JsonUi::resolve` and `JsonUi::resolve_with_errors`)
- Commit `b0bf708e` (RED) — FOUND in `git log --oneline`
- Commit `6b35a3a5` (GREEN) — FOUND in `git log --oneline`
- Acceptance criteria — all 6 pass:
  - `grep -c "pub fn expand_directives" ferro-json-ui/src/resolve.rs` = 1 ✓
  - `grep -cE "expand_directives," ferro-json-ui/src/lib.rs` = 1 ✓
  - `grep -c "expand_directives" framework/src/json_ui/mod.rs` = 5 (≥2) ✓
  - Pass ordering: expand_directives appears BEFORE resolve_actions and resolve_expressions ✓
  - `grep -c "fn evaluate_if\|fn check_if_predicate\|fn if_predicate_eval" ferro-json-ui/src/resolve.rs` = 0 ✓ (D-04 reuse mandate)
  - `grep -c "\.evaluate(" ferro-json-ui/src/resolve.rs` = 1 (≥1) ✓ (uses Visibility::evaluate)
- `cargo test -p ferro-json-ui --lib expand_ --all-features` — 12/12 pass ✓
- `cargo test -p ferro-json-ui --all-features` — 455 + 11 + 8 + 5 doctests pass ✓
- `cargo test -p ferro-rs --all-features` — 510+ pass ✓
- `cargo build -p ferro-json-ui --all-features` — clean ✓
- `cargo build -p ferro-rs --all-features` — clean ✓
- `cargo fmt --all -- --check` — clean ✓
- `cargo clippy --all --all-targets -- -D warnings` — clean ✓

---
*Phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test*
*Plan: 03*
*Completed: 2026-05-16*
