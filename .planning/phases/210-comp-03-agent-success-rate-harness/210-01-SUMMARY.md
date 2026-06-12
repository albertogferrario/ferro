---
phase: 210-comp-03-agent-success-rate-harness
plan: "01"
subsystem: ferro-mcp
tags: [agent-harness, comp-03, corpus, contamination-guard, dev-deps]
dependency_graph:
  requires: []
  provides:
    - ferro-mcp/tests/agent_harness.rs (COMP-03 test target skeleton + contamination guard)
    - ferro-mcp/tests/fixtures/agent_harness/corpus.json (14-task contamination-guarded corpus)
    - ferro-mcp/Cargo.toml [dev-dependencies] delta (rmcp client+transport-async-rw, serde derive)
  affects:
    - ferro-mcp test surface (adds agent_harness integration test target)
tech_stack:
  added:
    - rmcp 0.12 client+transport-async-rw (dev-dep only)
    - serde 1 derive (dev-dep — serde already a prod dep, this adds derive for test structs)
  patterns:
    - include_str! compile-time fixture loading (corpus guard is a standing CI invariant)
    - Contamination denylist pattern (nouns from catalog.rs + gestiscilo + Italian state labels)
    - Inlined format args in assert! macros (clippy uninlined_format_args compliance)
key_files:
  created:
    - ferro-mcp/tests/agent_harness.rs
    - ferro-mcp/tests/fixtures/agent_harness/corpus.json
  modified:
    - ferro-mcp/Cargo.toml
decisions:
  - "Corpus contamination scan ran twice: python smoke-check caught 'approval' in two descriptions; rephrase removed it before the Rust guard ran. Final denylist includes 'approval' and other catalog.rs fixture nouns beyond the initial plan list."
  - "Tasks 1, 2, and 3 committed atomically: include_str! in Task 3 requires the corpus fixture from Task 2 to exist at compile time, making them a single compilable unit."
metrics:
  duration_seconds: 332
  completed_date: "2026-06-12"
  tasks_completed: 3
  files_modified: 3
---

# Phase 210 Plan 01: COMP-03 Agent Harness Foundation Summary

14-task contamination-guarded NL corpus (2 per intent × 7 intents) with a standing CI contamination-guard test; rmcp client+transport-async-rw and serde derive added as dev-only dependencies.

## What Was Built

**Dev-dependency delta (`ferro-mcp/Cargo.toml`):** Added `rmcp = { version = "0.12", features = ["client", "server", "transport-async-rw"] }` and `serde = { version = "1", features = ["derive"] }` to `[dev-dependencies]` only. The library `[dependencies]` rmcp entry (`features = ["server", "transport-io"]`) is unchanged. This enables the in-process rmcp client that Wave 2/3 will use to drive `FerroMcpService` over a `tokio::io::duplex` pipe.

**Corpus (`ferro-mcp/tests/fixtures/agent_harness/corpus.json`):** 14 tasks, 2 per intent across Browse, Focus, Collect, Process, Summarize, Analyze, and Track. Each task has `id`, `target_intent`, `description` (NL only, no ferro intent vocabulary), `expected_actions`, and `expected_guards`. Domains are deliberately exotic:

| Intent | Task IDs |
|--------|----------|
| Browse | `browse-mineral-specimens`, `browse-aviary-band-records` |
| Focus | `focus-glacier-core-sample`, `focus-meteorite-custody-entry` |
| Collect | `collect-loom-warp-tension-log`, `collect-kelp-transect-survey-entry` |
| Process | `process-telescope-allocation-slots`, `process-kiln-firing-batch` |
| Summarize | `summarize-apiary-hive-yield`, `summarize-reef-transect-metrics` |
| Analyze | `analyze-aurora-event-log`, `analyze-seismograph-burst-log` |
| Track | `track-seed-specimen-custody`, `track-expedition-parcel-dispatch` |

Analyze tasks carry an intentional one-numeric-aggregate constraint (exactly one Money/Percentage/Quantity field + a DateTime field) to stay above the Summarize tie-break threshold — the thin Analyze↔Summarize margin documented in catalog.rs lines 12–19.

**Test target (`ferro-mcp/tests/agent_harness.rs`):** Module header documents the hybrid execution model (replay path / live path), all four tier definitions (T1–T4), and the four-wave build structure. Contains `corpus_contamination_guard()` — a plain `#[test]` (not `#[ignore]`, not async, no network) that:
1. Loads corpus via `include_str!` at compile time.
2. Asserts `len() == 14`.
3. Asserts the multiset of `target_intent` values is exactly `{Browse:2, Focus:2, Collect:2, Process:2, Summarize:2, Analyze:2, Track:2}` in Rust (not just python).
4. Runs the contamination denylist check: 31 nouns sourced from `catalog.rs` + project memory, checked as substrings in each task's lowercased description and id.

## Verification Results

```
cargo test -p ferro-mcp --test agent_harness corpus_contamination_guard
test corpus_contamination_guard ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```
cargo clippy -p ferro-mcp --all-targets -- -D warnings
Finished `dev` profile — no warnings, no errors
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Contamination scan found 'approval' in two task descriptions**
- **Found during:** Task 2 corpus authoring
- **Issue:** `process-telescope-allocation-slots` used "reviewer approval check" and "approval gate"; `track-seed-specimen-custody` used "approval gate" — `approval` is in the denylist (from catalog.rs `approval_workflow` fixture).
- **Fix:** Replaced "approval check" with "reviewer sign-off check", "approval gate" with "reviewer sign-off gate" / "sign-off gate". Added "approval" explicitly to the Rust denylist.
- **Files modified:** `ferro-mcp/tests/fixtures/agent_harness/corpus.json`
- **Commit:** 54f1537b

**2. [Rule 1 - Bug] Clippy uninlined_format_args in three assert! call sites**
- **Found during:** Task 3 clippy gate
- **Issue:** Three `assert!` / `assert_eq!` macros used old-style `"text {}", var` format strings instead of Rust 2021 inlined `"text {var}"` form. Clippy `-D warnings` rejects these.
- **Fix:** Rewrote all three assert messages using inlined format args.
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** 54f1537b (same commit — fixed before commit)

**3. [Rule 2 - Missing functionality] Extended denylist beyond initial plan list**
- **Found during:** Task 2/3 review of catalog.rs fixtures
- **Issue:** The plan's denylist enumerated the core nouns but missed several catalog.rs fixture entity names: `article`, `approval`, `revenue`, `sales`, `variant`, `publication` (from `article_detail`, `approval_workflow`, `revenue_summary`, `sales_timeseries` fixture names in catalog.rs).
- **Fix:** Added these six nouns to the Rust `DENYLIST` constant and the python smoke-check list. The contamination guard now covers all catalog.rs domain nouns.
- **Files modified:** `ferro-mcp/tests/agent_harness.rs`
- **Commit:** 54f1537b

### Atomic Commit (Tasks 1+2+3 together)

Tasks 1, 2, and 3 were committed atomically rather than separately. Reason: `include_str!("fixtures/agent_harness/corpus.json")` in `agent_harness.rs` (Task 3) is resolved at compile time — `cargo build -p ferro-mcp --tests` (Task 1's verification) fails if the corpus file does not exist. The three tasks form a single compilable unit.

## Known Stubs

None. The corpus is fully authored with real NL descriptions. The test target skeleton is minimal by design — later waves add the scorer and live loop. No placeholder data flows to any UI.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: information_disclosure | ferro-mcp/tests/fixtures/agent_harness/corpus.json | Corpus is a committed public artifact; contamination guard (T-210-01) enforces tenant-neutral domain nouns — no app identity, no gestiscilo-specific nouns in any description or id. Verified passing. |

## Self-Check

Files created/modified:
- `ferro-mcp/tests/agent_harness.rs` — FOUND
- `ferro-mcp/tests/fixtures/agent_harness/corpus.json` — FOUND
- `ferro-mcp/Cargo.toml` — FOUND (modified)

Commit: `54f1537b` — FOUND in git log.

## Self-Check: PASSED
