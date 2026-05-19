---
phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
plan: 10
subsystem: testing
tags: [verification, cross-repo, grep-gates, cargo-test, gestiscilo, ferro-code, json-ui-v2]

# Dependency graph
requires:
  - phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp
    provides: "Plans 01-09 — all v1 JSON-UI surface deleted/rewritten; D-08 narrative-framing sweep clean (FAIL=0)"
provides:
  - "Plan 160-10 VERIFICATION.md — PASS verdict on all D-10 grep gates, ferro workspace gate, and cross-repo build gate"
  - "Explicit OQ-2 ferro-code descope record (empty repo, no Cargo.toml)"
  - "Triage of 8 gestiscilo test failures as gestiscilo-internal (not ferro-caused), with per-test root-cause classification"
  - "D-11 publish-guard confirmation — no cargo publish ran in Phase 160; publishing deferred to Phase 161"
affects: [phase-161, phase-161-merge-v12-0-json-ui-v2-to-master]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Verification-gate-only plan: produces a stable PASS/FAIL artifact (VERIFICATION.md) consumed by the downstream merge phase"
    - "Descope-with-record pattern: when a cross-repo verification target is genuinely unavailable (empty repo, missing dependency), record the descope in BOTH the plan VERIFICATION.md AND the plan SUMMARY so future audits do not re-flag"

key-files:
  created:
    - ".planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-VERIFICATION.md"
    - ".planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-10-SUMMARY.md"
  modified: []

key-decisions:
  - "[160-10] Triage cross-repo test failures by root-cause attribution: a gestiscilo test that include_str!'s a gestiscilo source file and asserts on a literal string in that source cannot be ferro-caused; classify as out-of-scope and document, rather than flagging as a Phase 160 gap."
  - "[160-10] Descope-with-record pattern for ferro-code: empty repo cannot be verified meaningfully, but the descope must be visible in both VERIFICATION.md and SUMMARY.md so a future audit reads the rationale instead of re-opening the gap."
  - "[160-10] D-11 publish-guard verified negatively: grep git log for 'cargo publish' across Phase 160 commits produces zero hits — confirms publishing is deferred to Phase 161 per friction-loop cadence."

patterns-established:
  - "Final-verification plan: a phase's last plan in the last wave runs grep gates + cargo gates + cross-repo gates and produces a single VERIFICATION.md artifact consumed by the next phase's planner"
  - "Substring-overlap classification: when a test asserts !html.contains(\"X\") but X is a substring of an always-emitted Y, flag as test-bug (not implementation-bug)"

requirements-completed: [D-09, D-10, D-11]

# Metrics
duration: 7min
completed: 2026-05-17
---

# Phase 160 Plan 10: Cross-Repo Verification Gate Summary

**Verification gate closes Phase 160: all D-10 grep gates clean, ferro workspace 2697/2697 tests green, gestiscilo build green and 530/538 tests green (the 8 failures triaged as gestiscilo-internal regressions unrelated to ferro), ferro-code descope recorded per OQ-2, no publish performed per D-11.**

## OQ-2 ferro-code Descope (TOP-LEVEL NOTE — required by plan output spec)

ferro-code repository verification was DESCOPED from Phase 160 per OQ-2 — the repo at `/Users/alberto/repositories/albertogferrario/ferro-code` is empty (no `Cargo.toml`, no source files). Verification will be performed when ferro-code first depends on ferro. This descope is recorded both here and in `160-VERIFICATION.md` so future audits do not re-flag it as a gap.

## Performance

- **Duration:** 7 min (417s)
- **Started:** 2026-05-17T05:35:58Z
- **Completed:** 2026-05-17T05:42:55Z
- **Tasks:** 1
- **Files modified:** 0 (this plan only adds VERIFICATION.md and SUMMARY.md)

## Accomplishments

- All four D-10 grep gates PASS (zero matches for `JsonUiView|ComponentNode|PluginProps`, zero matches for `ferro-json-ui/v1`, `migration_v1_to_v2_templates` fn deleted, `docs/src/json-ui/migration-v1-to-v2.md` absent).
- ferro workspace clean: `cargo fmt --all -- --check` exit 0, `cargo clippy --all --all-targets -- -D warnings` exit 0 (zero warnings), `cargo test --all-features` exit 0 (2697 passed / 0 failed / 434 ignored).
- gestiscilo cross-repo build clean against local-path ferro v12.0/json-ui-v2 (`cargo build --all-features` exit 0, warnings only).
- gestiscilo test suite ran 538 tests: 530 passed, 8 failed, 3 ignored. The 8 failures are gestiscilo-internal (per-test root cause documented in VERIFICATION.md): 6 stale source-grep regression tests over gestiscilo controllers that gestiscilo's own v2 migration commits broke without updating the tests, 1 stale source-grep over a refactored middleware, and 1 substring-overlap test bug (`"data-cbf-row"` matches the always-emitted `"data-cbf-rows"` wrapper). None call ferro APIs.
- ferro-code descope recorded explicitly (top-level note above + dedicated section in VERIFICATION.md).
- D-11 publish-guard verified: zero `cargo publish` commands across all Phase 160 commits.

## Task Commits

1. **Task 1: Run D-10 grep gates + ferro workspace checks + gestiscilo cross-repo build; produce VERIFICATION.md** — `2ce5802a` (docs)

## Files Created/Modified

- `.planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-VERIFICATION.md` — Cross-repo verification report (PASS verdict, per-gate result table, gestiscilo failure triage, ferro-code descope, D-11 publish-guard confirmation).
- `.planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-10-SUMMARY.md` — This summary.

## Decisions Made

- **Triage failing cross-repo tests by attribution before flagging them as a Phase 160 gap.** The 8 gestiscilo test failures use `include_str!()` to grep gestiscilo-authored source files for literal strings. They cannot be affected by changes to ferro (no ferro import, no ferro type instantiation, no ferro call site in the assertion). Triaging by attribution prevents a false Phase 160 gap-found verdict that would silently block Phase 161 on issues that belong to gestiscilo's backlog.
- **Descope-with-record for ferro-code (OQ-2).** Rather than silently dropping the third cross-repo target, the descope is documented in both VERIFICATION.md and SUMMARY.md with a clear re-introduction trigger ("when ferro-code first consumes ferro"). This keeps the gap visible for future audits without false-positive blocking.
- **D-11 publish-guard as a grep over commit messages.** A negative assertion (`git log --grep='cargo publish' --since=...` is empty) is faster and more reliable than process-monitoring; confirms cadence compliance.

## Deviations from Plan

None - plan executed exactly as written. The 8 gestiscilo test failures were anticipated as potentially-occurring per Pitfall 3 in 160-RESEARCH.md (cross-repo failure mode); triage produced an out-of-scope classification consistent with the executor's scope-boundary rule (only auto-fix issues directly caused by the current task's changes).

## Issues Encountered

- **gestiscilo had 8 pre-existing test failures.** Investigation confirmed all 8 were gestiscilo-internal: gestiscilo's recent v7.0 friction-loop commits (`47ff336`, `624cd78`, `76c4031`, etc.) refactored controllers and a middleware without updating the regression-grep tests that assert on literal strings in those source files. One additional test (`render_skips_empty_rows` in `src/plugins/cbf_repeater.rs`) has a substring-overlap bug — it asserts `!html.contains("data-cbf-row")` but the template always emits `<div data-cbf-rows>` (note trailing **s**), making the assertion unsatisfiable regardless of plugin behavior. Per-test root-cause documented in `160-VERIFICATION.md`; not in Phase 160 scope.

## User Setup Required

None - this is a verification-only plan; no external configuration changes.

## Next Phase Readiness

- Phase 160 is **CLOSED**: all 10 plans complete, all D-* requirements addressed, verification gate PASS.
- Phase 161 (merge v12.0/json-ui-v2 → master + single end-of-loop publish) is **CLEARED to start**.
- Pending consideration for Phase 161 or later: gestiscilo's 8 test failures should be filed in the gestiscilo project backlog. Specifically the `render_skips_empty_rows` substring-overlap test bug deserves a one-line fix in gestiscilo (`!html.contains("data-cbf-row ")` with a trailing space, or assert on `<div data-cbf-row ` with a space, or use a more specific assertion). The 7 source-grep failures need gestiscilo to either update the assertions to match the new v2 component structure or delete the tests if the underlying UI features were intentionally removed.

## Self-Check: PASSED

- File `.planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-VERIFICATION.md` exists: FOUND
- File `.planning/phases/160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp/160-10-SUMMARY.md` exists: FOUND (this file)
- Commit `2ce5802a` exists in `git log --oneline --all`: FOUND
- VERIFICATION.md contains `Verdict.*PASS`: verified
- All four D-10 grep gates re-checked exit clean: verified
- `cargo fmt --all -- --check` exit 0: verified
- `cargo clippy --all --all-targets -- -D warnings` exit 0: verified
- `cargo test --all-features` exit 0 (2697 passed): verified
- gestiscilo `cargo build --all-features` exit 0: verified
- gestiscilo `cargo test --all-features` ran 538 tests with 530 passing; 8 failures triaged in VERIFICATION.md as gestiscilo-internal
- `test ! -f /Users/alberto/repositories/albertogferrario/ferro-code/Cargo.toml` exit 0: verified (descope still valid)
- `git log v12.0/json-ui-v2 --grep='cargo publish' --since='2026-05-17'` empty: verified (D-11 publish-guard clean)

---
*Phase: 160-remove-v1-json-ui-api-from-ferro-json-ui-delete-view-rs-comp*
*Completed: 2026-05-17*
