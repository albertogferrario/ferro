---
phase: 229-framework-benchmark-harness-foundation-1a-build-the-reproduc
plan: 03
subsystem: infra
tags: [docker, oha, tokei, benchmark, load-testing, python]

# Dependency graph
requires:
  - phase: 229-02
    provides: parse_perf.parse_oha interface (D-07) + count_static + build_tables contracts
provides:
  - Pinned containerised toolbox image ferro-bench-toolbox (oha 1.9.0 + tokei 12.1.2 + python3/jq/curl)
  - run_perf.py: CLI driver that invokes oha with warmup + timed run, writes perf-<framework>.json
affects:
  - 229-04 (Ferro micro-app Dockerfile, compose — uses toolbox image)
  - 229-05 (results run — invokes run_perf.py and the toolbox)

# Tech tracking
tech-stack:
  added:
    - oha 1.9.0 (HTTP load generator; --output-format json; compiled from source inside toolbox)
    - tokei 12.1.2 (static line counter; --output json; compiled from source inside toolbox)
    - rust:1.88.0-slim-bookworm (multi-stage build base)
    - debian:bookworm-slim (runtime base)
  patterns:
    - Multi-stage Docker build: heavy compile (Rust toolchain + oha + tokei) in build stage; only binaries copied to slim runtime
    - Python subprocess invocation of oha using two-token form: ["--output-format", "json"] (not --json)
    - Warmup-then-timed pattern: discarded warmup run followed by JSON-output timed run per endpoint

key-files:
  created:
    - benchmark/harness/Dockerfile.toolbox
    - benchmark/harness/perf/run_perf.py
  modified: []

key-decisions:
  - "oha pin bumped from 1.4.7 to 1.9.0: 1.4.7 uses -j/--json (not --output-format); --output-format introduced in 1.9.0 (confirmed via binary search 1.5–1.10)"
  - "Added apt-get install make to build stage: oha 1.x depends on jemalloc-sys which calls make at build time; not present in rust:slim images"
  - "run_perf.py uses ['--output-format', 'json'] as two separate argv tokens — single --json flag does not exist in oha"

patterns-established:
  - "oha invocation: -z <dur>, -c <n>, --no-tui, --output-format json, <url> — in that order"
  - "Warmup run intentionally omits --output-format (output discarded); only timed run writes JSON"
  - "perf key = endpoint path without query string: ep.split('?')[0]"

requirements-completed: []

# Metrics
duration: 21min
completed: 2026-06-15
---

# Phase 229 Plan 03: Toolbox Image + Perf Runner Summary

**Pinned ferro-bench-toolbox Docker image (oha 1.9.0 + tokei 12.1.2) and run_perf.py oha driver writing perf-<framework>.json via the parse_oha interface**

## Performance

- **Duration:** 21 min
- **Started:** 2026-06-15T02:06:15Z
- **Completed:** 2026-06-15T02:27:49Z
- **Tasks:** 2 (Task 6a: Dockerfile.toolbox; Task 6b: run_perf.py)
- **Files modified:** 2

## Accomplishments

- Multi-stage Dockerfile.toolbox builds cleanly: Rust build stage compiles oha + tokei from source (pinned, --locked), runtime stage is debian:bookworm-slim + python3/jq/curl
- oha --help confirms `--output-format` present in image; tokei 12.1.2 has json serialization support
- run_perf.py drives per-endpoint oha runs (warmup discarded, timed run JSON-parsed via parse_oha) and writes perf-<framework>.json consumed by build_tables (Plan 02/05)

## Task Commits

Both files entered a single commit (run_perf.py was pre-staged from plan 02 setup alongside the Dockerfile):

1. **Tasks 6a + 6b: Dockerfile.toolbox + run_perf.py** - `8afdbfa9` (feat)

**Plan metadata:** pending final docs commit

## Files Created/Modified

- `benchmark/harness/Dockerfile.toolbox` — multi-stage image: rust:1.88.0-slim-bookworm build stage + debian:bookworm-slim runtime; pins oha 1.9.0 + tokei 12.1.2 + python3/jq/curl
- `benchmark/harness/perf/run_perf.py` — CLI: base_url + framework + out_dir → perf-<framework>.json; imports parse_oha from Plan 02; uses `--output-format json` (two tokens)

## Decisions Made

- oha pinned at 1.9.0 (not the D-05 target of 1.4.7): 1.4.7 uses `-j`/`--json` which does not produce the JSON schema `parse_oha` expects; `--output-format json` arrived in 1.9.0 (binary searched across 1.5–1.10). The meta.json tooling block in Plan 05 should record `oha_version: "1.9.0"`.
- `make` installed in build stage: oha 1.x pulls in jemalloc-sys which requires GNU make at compile time; not present in `rust:1.88.0-slim-bookworm`; added as a single `apt-get install make` layer before the cargo installs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] oha 1.4.7 build failed: jemalloc-sys requires make**
- **Found during:** Task 6a (first docker build attempt)
- **Issue:** `rust:1.88.0-slim-bookworm` does not include make; jemalloc-sys (pulled by oha 1.4.7) calls `make` at build time → `No such file or directory (os error 2)` panic in build.rs
- **Fix:** Added `apt-get install -y --no-install-recommends make` as a separate RUN layer in the build stage before the cargo installs; make is only in the build stage (not copied to runtime)
- **Files modified:** `benchmark/harness/Dockerfile.toolbox`
- **Verification:** Image built successfully; oha + tokei binaries present in runtime stage
- **Committed in:** `8afdbfa9`

**2. [Rule 1 - Bug] oha 1.4.7 lacks --output-format; bumped to 1.9.0**
- **Found during:** Task 6a acceptance check (`oha --help | grep -i output-format`)
- **Issue:** oha 1.4.7 exposes `-j`/`--json` (not `--output-format`); the RESEARCH doc flagged this as Assumption A3 with a verify-at-build-time gate; the gate fired
- **Fix:** Binary searched oha versions (1.5 → no, 1.7 → no, 1.8 → no, 1.9 → yes); bumped Dockerfile.toolbox pin from `1.4.7` to `1.9.0`; rebuilt image; reran all acceptance checks
- **Files modified:** `benchmark/harness/Dockerfile.toolbox`
- **Verification:** `docker run --rm ferro-bench-toolbox sh -c 'oha --help | grep -i output-format'` prints the flag; `oha --version` shows 1.9.0
- **Committed in:** `8afdbfa9`

---

**Total deviations:** 2 auto-fixed (1 blocking build failure, 1 version-flag mismatch)
**Impact on plan:** Both fixes necessary for correctness; oha version bump is the plan-mandated fallback path (plan text: "bump the oha pin to a known-good newer 1.x"). No scope creep.

## Issues Encountered

The RESEARCH doc correctly flagged oha 1.4.7's `--output-format` support as an unverified assumption (A3) and specified a build-time verification gate. That gate fired exactly as designed — the fix path (bump pin, record new version) was already written into the plan.

## User Setup Required

None — Docker is the only prerequisite and was already running.

## Next Phase Readiness

- `ferro-bench-toolbox` image is built locally and ready for use in Plan 04 (app Dockerfiles) and Plan 05 (results run)
- run_perf.py is ready to invoke once a target app is running on a local URL
- Plan 05 meta.json should record `oha_version: "1.9.0"` and `tokei_version: "12.1.2"`
- The Dockerfile.toolbox can be rebuilt deterministically on any machine with Docker + internet access

---
*Phase: 229-framework-benchmark-harness-foundation-1a*
*Completed: 2026-06-15*
