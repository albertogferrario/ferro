---
phase: 229-framework-benchmark-harness-foundation-1a-build-the-reproduc
plan: "02"
subsystem: benchmark/harness
tags: [benchmark, python, tdd, harness, perf-parser, static-counter, report-builder]
dependency_graph:
  requires: []
  provides:
    - benchmark/harness/perf/parse_perf.py (D-07 interface)
    - benchmark/harness/static/count_static.py (D-08 interface)
    - benchmark/harness/report/build_tables.py (D-09 interface)
  affects:
    - Plan 03 (run_perf.py imports parse_oha)
    - Plan 05 (results run imports count_static.run, build_tables.render_markdown + load_results)
tech_stack:
  added:
    - pytest 9.1.0 (brew)
  patterns:
    - TDD RED→GREEN per module (no REFACTOR pass needed — code shipped exactly as specified)
key_files:
  created:
    - benchmark/harness/perf/parse_perf.py
    - benchmark/harness/perf/test_parse_perf.py
    - benchmark/harness/static/count_static.py
    - benchmark/harness/static/test_count_static.py
    - benchmark/harness/report/build_tables.py
    - benchmark/harness/report/test_build_tables.py
  modified: []
decisions:
  - pytest installed via brew (externally-managed Python environment blocked pip install)
metrics:
  duration: ~10 min
  completed: 2026-06-15
  tasks: 3
  files: 6
---

# Phase 229 Plan 02: Benchmark Harness Python Units Summary

Three TDD-tested Python harness units define the measurement pipeline interface contracts (D-07/D-08/D-09) consumed by the perf runner (Plan 03) and results run (Plan 05).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 3 | Perf parser TDD — parse_oha D-07 | c462854f | benchmark/harness/perf/parse_perf.py, test_parse_perf.py |
| 4 | Static counter TDD — count_static D-08 | eb39cfcd | benchmark/harness/static/count_static.py, test_count_static.py |
| 5 | Report builder TDD — build_tables D-09 | 9f72a67f | benchmark/harness/report/build_tables.py, test_build_tables.py |

## Interfaces Delivered

**D-07 — `parse_perf.parse_oha(raw: str) -> dict`**
Returns `{rps, p50_ms, p90_ms, p99_ms, success_rate}`. Latency values converted from seconds to ms (`× 1000, round(3)`). Raises `ValueError` when `success_rate < 0.99` — prevents recording results from a saturated or failing run. Docstring references `oha --output-format json` (the correct flag).

**D-08 — `count_static` module**
- `summarize_tokei(raw: str) -> {code_lines, files}` — totals `code` across languages, counts `reports` entries per language; skips the `"Total"` key.
- `count_tokens(paths: list[str]) -> int` — whitespace-split token count across files.
- `run(app_dir: str) -> {code_lines, files, source_tokens}` — shells `tokei --output json` (excluding `vendor`/`target`), invokes `summarize_tokei`, then adds `source_tokens` from `count_tokens`. Runnable as `python3 count_static.py <app_dir>`.

**D-09 — `build_tables` module**
- `render_markdown(data: dict) -> str` — emits a Raw Performance table (rps per endpoint per framework + ratio column) and a Static Compression table (code_lines/files/source_tokens). Ratio is computed as hi/lo; 200000 / 9000 = 22.22x (passes `22.2x` assertion).
- `load_results(date_dir: str) -> dict` — globs `*.json`, parses `perf-<fw>.json` and `static-<fw>.json` filenames, returns nested dict keyed by framework then kind.

## Verification

All three pytest suites pass (6 tests total, 2 per module):

```
benchmark/harness/perf:   2 passed
benchmark/harness/static: 2 passed
benchmark/harness/report: 2 passed
```

TDD gate compliance:
- Each module observed RED (`ModuleNotFoundError`) before GREEN.
- No REFACTOR pass was needed; implementations match the plan specification exactly.

## Deviations from Plan

**1. [Rule 3 - Blocking] pytest not available via python3 -m pytest**

- **Found during:** Pre-task setup
- **Issue:** `python3 -m pytest` failed with `No module named pytest`. `pip install` blocked by externally-managed environment (PEP 668, Homebrew Python 3.14).
- **Fix:** Installed `pytest` via `brew install pytest`. The `pytest` CLI then resolved to `/opt/homebrew/bin/pytest` (version 9.1.0) and all test invocations used it directly.
- **Impact:** No interface changes; all plan-specified test commands work correctly with `pytest` in PATH.

## Known Stubs

None. All three modules implement their full interface contracts; no placeholder values or deferred data paths.

## Threat Flags

None. Pure local Python units: no network calls, no secrets, no untrusted external input. The `parse_oha` success-rate gate (T-229-03 mitigation) is implemented and tested.

## Self-Check: PASSED

Files verified:
- benchmark/harness/perf/parse_perf.py — FOUND
- benchmark/harness/perf/test_parse_perf.py — FOUND
- benchmark/harness/static/count_static.py — FOUND
- benchmark/harness/static/test_count_static.py — FOUND
- benchmark/harness/report/build_tables.py — FOUND
- benchmark/harness/report/test_build_tables.py — FOUND

Commits verified: c462854f, eb39cfcd, 9f72a67f — all present in git log.
