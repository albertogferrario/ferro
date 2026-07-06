---
phase: 207-comp-02-synthetic-regression-catalog
verified: 2026-06-12T00:00:00Z
status: passed
score: 6/6
overrides_applied: 0
re_verification: null
---

# Phase 207: COMP-02 — Synthetic Regression Catalog Verification Report

**Phase Goal:** A permanent, machine-checkable baseline asserts that `derive_intents()` produces the correct primary intent for each canonical app class. The catalog is the regression foundation for every future change to `ferro-projections/src/derive.rs` and `intent.rs`, and it is the ground-truth source that Phase 210 (agent harness) consumes.

**Verified:** 2026-06-12
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Evidence Collected

All checks run directly against the codebase. No trust placed in SUMMARY claims — each was independently verified.

### Test Suite

```
cargo test -p ferro-projections --test catalog 2>&1 | tail -3
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

19 tests, 0 failed, 0 ignored. Test breakdown: 7 canonical + 4 adversarial + 7 snapshot + 1 proptest = 19.

### Clippy

```
cargo clippy -p ferro-projections --all-targets -- -D warnings
Finished `dev` profile [unoptimized] target(s) in 0.17s
```

Exit 0. No warnings.

### Formatting

```
cargo fmt --all -- --check
(no output)
```

Exit 0. All files formatted.

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cargo test -p ferro-projections --test catalog` exits 0 with all 7 canonical tests, 4 adversarial tests, 7 snapshot tests, and the proptest passing | VERIFIED | `19 passed; 0 failed; 0 ignored` — confirmed live |
| 2 | Each of the 7 structural intents has a named ServiceDef builder fn whose `derive_intents()[0].intent` equals the expected intent | VERIFIED | `fn canonical_{browse,focus,collect,process,summarize,analyze,track}` — each asserts primary intent identity + calibrated floor + margin. Confirmed in `catalog.rs` lines 239–642 |
| 3 | A future regression in `derive.rs` that flips any canonical primary intent produces a named, legible test failure (no `#[ignore]`, runs in the default CI gate) | VERIFIED | `grep -c '#[ignore]' catalog.rs` = 0; all 7 canonical tests plus 4 adversarial and 1 proptest in CI gate with no ignore annotations |
| 4 | `ferro-projections/src/intent.rs` and `ferro-projections/src/derive.rs` are byte-for-byte unchanged (read-only system under test) | VERIFIED | `git diff --stat 6c712636 -- ferro-projections/src/intent.rs ferro-projections/src/derive.rs` produced empty output |
| 5 | Structural-invariant assertions outnumber `insta` snapshot assertions in `catalog.rs` | VERIFIED | 59 structural/prop assertions (`assert!` / `assert_eq!` / `prop_assert`) vs 7 `assert_yaml_snapshot!` calls — ratio 8.4× |
| 6 | "Discovered weaknesses" note names >= 1 real, calibration-surfaced limitation | VERIFIED | Module doc block lines 8–19: names Analyze↔Summarize thin margin (0.04 normalized), cites the flat `datetime_numeric_cooccurrence` 0.35 signal that does not scale with DateTime count. Concrete, non-boilerplate |

**Score:** 6/6 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-projections/Cargo.toml` | `[dev-dependencies]` with `insta` (yaml) + `proptest` | VERIFIED | Lines 19–21: `insta = { version = "1", features = ["yaml"] }` and `proptest = "1"`; no `ferro-json-ui` entry |
| `ferro-projections/tests/catalog.rs` | 7 canonical fixtures + per-intent tests + 4 adversarial + 7 snapshot + proptest + discovered-weaknesses doc block; min 400 lines | VERIFIED | 969 lines; all required content present |
| `ferro-projections/tests/snapshots/catalog__canonical_browse.snap` | Committed insta snapshot containing "Browse" with no confidence floats | VERIFIED | Contains `intent: Browse` with signal strings only; `grep -RiE 'confidence|0\.[0-9]{2,}'` finds nothing in snapshots dir |
| `ferro-projections/tests/snapshots/catalog__canonical_focus.snap` | Same — "Focus" | VERIFIED | Present |
| `ferro-projections/tests/snapshots/catalog__canonical_collect.snap` | Same — "Collect" | VERIFIED | Present |
| `ferro-projections/tests/snapshots/catalog__canonical_process.snap` | Same — "Process" | VERIFIED | Present |
| `ferro-projections/tests/snapshots/catalog__canonical_summarize.snap` | Same — "Summarize" | VERIFIED | Present |
| `ferro-projections/tests/snapshots/catalog__canonical_analyze.snap` | Same — "Analyze" | VERIFIED | Present |
| `ferro-projections/tests/snapshots/catalog__canonical_track.snap` | Same — "Track" | VERIFIED | Present |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `ferro-projections/tests/catalog.rs` | `ferro_projections::derive_intents` | Direct function call on each fixture `ServiceDef` | VERIFIED | Pattern `derive_intents(&` appears in 7 canonical tests, 4 adversarial tests, 7 snapshot tests, and 1 proptest — 19 call sites total |
| `ferro-projections/Cargo.toml` | `insta` + `proptest` dev-deps | `[dev-dependencies]` section | VERIFIED | Section present at lines 19–21; `insta = { version = "1", features = ["yaml"] }` and `proptest = "1"` |

---

## Roadmap Success Criteria Coverage

| SC | Text (abbreviated) | Status | Evidence |
|----|-------------------|--------|---------|
| SC#1 | 7 canonical `ServiceDef` builder fns + `derive_intents(&service)[0].intent == ExpectedIntent` + confidence threshold | VERIFIED | 7 fixture fns in `mod fixtures { }` (lines 70–233); 7 canonical tests each assert identity + floor + margin with calibrated constants (`BROWSE_FLOOR=0.85`, etc.) |
| SC#2 | Structural-invariant assertions outnumber `insta` snapshot assertions; >=1 structural property per intent; no empty fixture | VERIFIED | 59 structural/prop vs 7 snapshot (8.4×); every canonical test has >=1 structural invariant on the `ServiceDef` shape (e.g., entity field count, writable ratio, state machine presence, guarded transition count); all fixtures are non-trivial (smallest is `collect_form` with 7 fields including 2 write-only) |
| SC#3 | >=1 non-trivial fixture per intent + >=1 explicitly adversarial competing-signal fixture documented in a comment | VERIFIED | 4 adversarial tests: `adversarial_browse_vs_summarize`, `adversarial_process_vs_track`, `adversarial_analyze_vs_summarize`, `adversarial_collect_vs_focus`; each has a `// competing:` inline rationale (4 such comments confirmed); each adversarial fixture has >=3 domain fields |
| SC#4 | All 7 catalog tests pass under `cargo test --all-features`; no `#[ignore]`; future `derive.rs` break causes named CI failure | VERIFIED | 19/19 pass; `#[ignore]` count = 0; all tests have named assertion messages that identify which intent failed |
| SC#5 | "Discovered weaknesses" note names >=1 real calibration-surfaced limitation | VERIFIED | Module doc block (lines 8–19) names: Analyze↔Summarize thin margin (ANALYZE_MARGIN=0.04), cites `datetime_numeric_cooccurrence` flat 0.35 signal that does not scale with DateTime field count; names the concrete scenario that flips the winner |

---

## Structural Checks (All Passing)

| Check | Command | Result |
|-------|---------|--------|
| Tests: 19 pass, 0 fail, 0 ignore | `cargo test -p ferro-projections --test catalog \| tail -3` | `ok. 19 passed; 0 failed; 0 ignored` |
| Clippy clean | `cargo clippy -p ferro-projections --all-targets -- -D warnings` | Exit 0 |
| Format clean | `cargo fmt --all -- --check` | Exit 0 |
| `intent.rs` / `derive.rs` read-only | `git diff --stat 6c712636 -- ferro-projections/src/intent.rs ferro-projections/src/derive.rs` | Empty (no changes) |
| No `#[ignore]` | `grep -c '#[ignore]' catalog.rs` | 0 |
| SC#2 assertion ratio | `grep -cE 'assert!|assert_eq!|prop_assert' catalog.rs` vs `grep -c 'assert_yaml_snapshot' catalog.rs` | 59 > 7 |
| 7 canonical fixtures | `grep -n 'fn canonical_' catalog.rs` | 7 lines (browse/focus/collect/process/summarize/analyze/track) |
| 4 adversarial tests | `grep -n 'fn adversarial_' catalog.rs` | 4 lines (browse_vs_summarize, process_vs_track, analyze_vs_summarize, collect_vs_focus) |
| proptest present + 256 cases | `grep -n 'cases: 256'` and `grep -n 'engine_never_panics'` | Lines 932 and 937 |
| Discovered weaknesses non-empty | `grep -A3 'Discovered weaknesses' catalog.rs` | Names Analyze↔Summarize thin margin with concrete numbers |
| No TODO placeholder | `grep -c 'TODO' catalog.rs` | 0 |
| `ferro-json-ui` not in dev-deps | `grep -E 'ferro-json-ui' ferro-projections/Cargo.toml` | No output |
| 7 snapshot files committed | `ls snapshots/catalog__canonical_*.snap \| wc -l` | 7 |
| Snapshots contain no confidence floats | `grep -RiE 'confidence|0\.[0-9]{2,}' snapshots/` | No output (CLEAN) |
| Calibration constants present | `grep -cE '_(FLOOR\|MARGIN): f64' catalog.rs` | 14 (floor + margin per intent) |
| No debug eprintln! | `grep -c 'eprintln!' catalog.rs` | 0 |
| 4 inline rationale comments | `grep -c '// competing:' catalog.rs` | 4 |
| 3 phase commits exist | `git log --oneline cd1f8c74 52b2fe67 b57578b4` | All 3 present |

---

## Requirements Coverage

| Requirement | Phase | Description | Status | Evidence |
|-------------|-------|-------------|--------|----------|
| COMP-02 | 207 | Synthetic catalog with regression harness covering 7 structural intents, structural invariants, competing-signal adversarial fixtures | SATISFIED | All 5 ROADMAP SCs verified above; `cargo test` 19/19 passing; catalog is the ground-truth baseline Phase 210 will consume |

---

## Anti-Patterns Found

None. Catalog.rs scanned for: TODO/FIXME, placeholder comments, empty handlers, hardcoded empty data, `#[ignore]`, debug `eprintln!`. All clear.

---

## Data-Flow Trace

Not applicable. This phase delivers test code only — no production components, no API routes, no dynamic data rendering. The system under test (`derive_intents()`) is a pure synchronous function; the catalog calls it directly. No Level 4 data-flow trace required.

---

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 19 catalog tests pass with 0 failures | `cargo test -p ferro-projections --test catalog \| tail -3` | `ok. 19 passed; 0 failed; 0 ignored` | PASS |
| `analyze_timeseries` primary is Intent::Analyze | `canonical_analyze` test in above run | Passed (included in 19) | PASS |
| `collect_form` primary is Intent::Collect | `canonical_collect` test in above run | Passed (included in 19) | PASS |
| Proptest 256-case robustness invariant | `engine_never_panics_returns_valid_scores` in above run | Passed (included in 19) | PASS |

---

## Human Verification Required

None. This phase is test-only Rust code with no UI, no visual output, and no external services. All success criteria are machine-verifiable.

---

## Gaps Summary

No gaps. All 6 must-have truths verified, all 5 ROADMAP success criteria met, all artifacts present and substantive, all key links wired, no anti-patterns, 0 `#[ignore]`, 0 debug residue.

---

_Verified: 2026-06-12_
_Verifier: Claude (gsd-verifier)_
