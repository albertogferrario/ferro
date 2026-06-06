---
phase: 182
slug: ferro-json-ui-data-lazy-hero-runtime-primitive
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-06
---

# Phase 182 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `cargo test` |
| **Config file** | `ferro-json-ui/Cargo.toml` (workspace inherited dev-deps; no separate test runner config) |
| **Quick run command** | `cargo test -p ferro-json-ui --lib runtime::tests` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~5–15s for the runtime module's tests; ~2–4 min for full workspace suite |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui --lib runtime::tests`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~15 seconds for the quick command

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 182-01-01 | 01 | 1 | LAZYHERO-02, LAZYHERO-03 | — | N/A (DOM-attribute primitive; no security surface) | unit (string-presence) | `cargo test -p ferro-json-ui --lib runtime_contains_lazy_hero_setup` | ❌ W0 | ⬜ pending |
| 182-01-02 | 01 | 1 | LAZYHERO-02, LAZYHERO-03 | — | N/A | unit (string-presence) | `cargo test -p ferro-json-ui --lib bundle_contains_all_setup_functions` | ✅ (extended) | ⬜ pending |
| 182-01-03 | 01 | 1 | LAZYHERO-02, LAZYHERO-03 | — | N/A | unit (string-presence) | `cargo test -p ferro-json-ui --lib dispatcher_invokes_every_setup` | ✅ (extended) | ⬜ pending |
| 182-02-01 | 02 | 2 | LAZYHERO-01 | — | N/A (deferred to consumer UAT) | manual-only | (Chrome DevTools Network panel + scroll; verified in gestiscilo Phase 186) | N/A | ⬜ pending |
| 182-03-01 | 03 | 2 | LAZYHERO-05 | — | N/A | integration (CI workflow) | Post-merge: GH Actions Wave1A run + `cargo search ferro-json-ui` showing 0.2.42 | N/A — CI artifact | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Note on task IDs:** The above is a guideline mapping based on RESEARCH.md. The planner finalizes the exact task IDs and plan/wave assignments in step 8. Update this table once PLAN.md files are generated.

---

## Wave 0 Requirements

- [ ] `ferro-json-ui/src/runtime/hero_lazy.rs` — does not exist; created as the implementation deliverable (NOT scaffolding — this IS the feature). Plan 01 creates it.
- [ ] New `#[test] fn runtime_contains_lazy_hero_setup` in `ferro-json-ui/src/runtime/mod.rs` — added in the same commit/plan that adds `hero_lazy.rs`.
- [ ] Updates to `bundle_contains_all_setup_functions` and `dispatcher_invokes_every_setup` assertion arrays in `ferro-json-ui/src/runtime/mod.rs` — same commit/plan.

*Existing `cargo test` infrastructure covers all phase requirements. No framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Below-the-fold video flips `preload` from `"none"` to `"auto"` exactly when rootMargin boundary crosses viewport | LAZYHERO-01 | Requires real browser IntersectionObserver + scroll + Network-panel inspection. No automated path exercises this in ferro's test suite — D-07 confirms "no headless browser test." | 1. Open jetskiadriatic landing (post gestiscilo Phase 186) with `<video preload="none" data-lazy-hero>` below fold. 2. Chrome DevTools → Network panel → filter `.mp4`/`.webm`/`media`. 3. Hard reload — observe: no video bytes requested before scroll. 4. Slow-scroll toward lazy hero — observe: video bytes start arriving ≈200px before viewport entry (or ≈400px for `data-lazy-hero-margin="400px 0px"` override). 5. Inspect element: `preload="auto"` + `data-lazy-hero-promoted="1"` after promote. 6. Re-scroll: no additional fetches (idempotency verified). |
| Crates.io publish via Wave1A succeeds | LAZYHERO-05 | Post-merge CI artifact. No pre-merge test path. | Verify GH Actions Wave1A run completes; `cargo search ferro-json-ui` returns `0.2.42`. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (1 new test added in Plan 01)
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s for quick command
- [ ] `nyquist_compliant: true` set in frontmatter (after planner finalizes task IDs)

**Approval:** pending

---

## Notes on Phase 182's validation shape

Phase 182's deliverable is a JS source string compiled into `FERRO_RUNTIME_JS` at process start. The "behavior" of the primitive is observable only in a real browser running real `<video>` elements scrolled into view. This makes the validation strategy intentionally string-presence-heavy on the ferro side and manual-UAT-heavy on the consumer side. Both shapes are documented in CONTEXT.md D-07 and RESEARCH.md §Validation Architecture.

This is consistent with how every other runtime primitive in `ferro-json-ui/src/runtime/` is validated. Future behavioral testing (e.g., a Playwright suite running tenant pages against the assembled bundle) would be a deferred infrastructure phase, not a Phase 182 deliverable.
