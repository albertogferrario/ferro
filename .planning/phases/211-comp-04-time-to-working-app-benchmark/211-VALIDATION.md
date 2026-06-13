---
phase: 211
slug: comp-04-time-to-working-app-benchmark
status: ready
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-13
---

# Phase 211 — Validation Strategy

> Per-phase validation contract. Observable signals from RESEARCH.md §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust, `ferro-cli/tests/benchmark_new_project.rs`) |
| **Config file** | none — integration test target under `ferro-cli/tests/` |
| **Quick run command** | `cargo test -p ferro-cli --test benchmark_new_project` (gate OFF → skips; proves CI-safe) |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy -p ferro-cli --all-targets -- -D warnings && cargo test -p ferro-cli --test benchmark_new_project` |
| **Estimated runtime** | <5s with `FERRO_BENCH` unset (gated skip) |

**Gated benchmark (opt-in, heavy, NOT CI):** `FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored --nocapture` — builds a full scaffolded app in a tmpdir (warm/local per-step breakdown). **Cold-cache run is a human-action via Docker** (see RESULTS.md).

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-cli --test benchmark_new_project` (gate off → green, proves the gate)
- **After every plan wave:** full suite command above
- **Before `/gsd-verify-work`:** default `cargo test` green WITHOUT `FERRO_BENCH` (always-green/bounded-disk invariant)
- **Max feedback latency:** ~5s (gated-skip path)

---

## Per-Task Verification Map

> Populated by the planner. Autonomous tasks verify with the gate-OFF path; the cold-cache Docker run is human-action.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 211-01-01 | 01 | 1 | COMP-04 | — | criterion dev-dep test-only; no always-on dep | build | `cargo build -p ferro-cli --tests 2>&1 \| tail -5` | ❌ W0 | ⬜ pending |

---

## Wave 0 Requirements

- [ ] `ferro-cli/tests/benchmark_new_project.rs` — new gated benchmark target
- [ ] `ferro-cli/Cargo.toml` `[dev-dependencies]` — criterion 0.8.2 `default-features=false, features=["cargo_bench_support"]`
- [ ] `ferro-cli/tests/fixtures/benchmark/RESULTS.md` + `Dockerfile` — committed apparatus

---

## Observable Signals (from RESEARCH.md §Validation Architecture)

| Signal | How proven | SC |
|--------|-----------|----|
| Gate works (CI-safe) | `cargo test -p ferro-cli --test benchmark_new_project` green with `FERRO_BENCH` unset (skips) | SC#1 |
| 5 steps timed individually | benchmark records per-step `Instant` durations; build step asserts exit 0 | SC#2 |
| Cold-cache run committed | RESULTS.md contains a row labeled `cache: cold` from the Docker run | SC#3 |
| Env spec complete | RESULTS.md has all of: rust toolchain version, cache state, host class, agent-assistance level, per-step breakdown, total (grep-checkable) | SC#4 |
| Weakness named | phase VERIFICATION / WEAKNESSES names ≥1 real finding | SC#5 |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cold-cache number | COMP-04 SC#3 | Needs Docker + clean container (no toolchain/cache); not in autonomous env; heavy disk/time | Build the committed Dockerfile, run the benchmark inside, record per-step + total into RESULTS.md (`cache: cold`) |

---

## Validation Sign-Off

- [x] Autonomous tasks have gate-off `<automated>` verify; cold run is documented human-action
- [x] Wave 0 covers test target, criterion dev-dep, RESULTS.md + Dockerfile
- [x] No watch-mode flags
- [x] Feedback latency < 5s (gated-skip path)
- [x] `nyquist_compliant: true`
- [ ] `wave_0_complete: true` — executor flips once the target + fixtures exist

**Approval:** approved 2026-06-13 (planning-stage contract complete)
