---
phase: 259
slug: request-scoped-memoization
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-21
---

# Phase 259 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` (async) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p framework memoize && cargo test -p ferro-macros` |
| **Full suite command** | `cargo test --all-features` (CI-exact) |
| **Estimated runtime** | quick ~10–40s; full suite several minutes (disk-full-prone — check `df`, clean `target/` first) |

---

## Sampling Rate

- **After every task commit:** Run the quick command scoped to the touched crate.
- **After every plan wave:** Run `cargo test --all-features` for the affected crates.
- **Before `/gsd-verify-work`:** Full CI-exact gate green — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
- **Max feedback latency:** ~40 seconds (quick), full suite as a wave gate.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| (Wave 0 refines) | — | — | LIVE-01 | — | N/A (request-scoped optimization, no auth/data-exposure surface) | — | — | ❌ W0 | ⬜ pending |

**Invariants the map MUST cover (Success Criteria → measurable tests):**

| Invariant | Success Criterion | Test type | Assertion mechanism |
|-----------|-------------------|-----------|---------------------|
| Run-once per `(callsite, args)` | SC-1 | unit table test | `Arc<AtomicUsize>` call counter == 1 after N calls with same args, inside one `MEMO_STORE.scope(...)` |
| Distinct args recompute | SC-1 | unit table test | counter increments per distinct arg hash (miss path) |
| Concurrent callers coalesce | SC-2 | `#[tokio::test]` | two `tokio::join!`ed awaits of the same `(callsite,args)` in one scope → body runs once (counter == 1) |
| Store dropped with request | SC-2 | unit test | store `Weak`/drop observation, or fresh scope shows miss (no cross-request leak) |
| Out-of-scope graceful no-op | D-02 | unit test | call outside any `MEMO_STORE.scope(...)` runs body, no panic; counter increments each call |
| `Err` cached for request | D-04 | `#[tokio::test]` | a `Result`-returning memoized fn returning `Err` returns the same `Err` to a second caller; body runs once |
| N intents / one key = one fetch | SC-3 | integration test | constructed harness: a `#[memoize]` loader invoked by a multi-intent render path; call counter == 1 across Browse+Summarize over one key |

---

## Wave 0 Requirements

- [ ] Unit test module for `MemoStore` hit/miss/coalesce (in `framework` — new `framework/src/memoize/` or `tests/`).
- [ ] `trybuild` (or documented compile-fail) case for a non-`Hash` argument, if the macro emits a hard compile error (per D-03).
- [ ] Render-path integration-test harness proving SC-3 honestly (schema-only renderer → the memoized fetch is a loader called before/around render, NOT discovered inside the renderer — see RESEARCH.md critical finding).

*Rust built-in test harness already exists workspace-wide; no framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| (none) | LIVE-01 | All invariants above are automatable with call counters + tokio async tests | — |

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 40s (quick command)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
