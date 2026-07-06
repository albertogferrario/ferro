---
phase: 208
slug: comp-05-cross-modality-vocabulary-sketch
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-12
---

# Phase 208 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`#[cfg(test)]` unit tests) |
| **Config file** | none — workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-projections --lib render::sketch` |
| **Full suite command** | `cargo test -p ferro-projections` |
| **Estimated runtime** | ~15 seconds (quick), ~45 seconds (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-projections --lib render::sketch`
- **After every plan wave:** Run `cargo test -p ferro-projections`
- **Before `/gsd-verify-work`:** Full suite green + `cargo fmt --all -- --check` + `cargo clippy --all --all-targets -- -D warnings` clean
- **Max feedback latency:** 45 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 208-01-01 | 01 | 1 | COMP-05 SC#1 | — | N/A (pure function, no I/O) | unit (smoke) | `cargo test -p ferro-projections --lib render::sketch::cli` | ❌ W0 | ⬜ pending |
| 208-01-02 | 01 | 1 | COMP-05 SC#1 | — | N/A | unit (smoke) | `cargo test -p ferro-projections --lib render::sketch::voice` | ❌ W0 | ⬜ pending |
| 208-01-03 | 01 | 1 | COMP-05 SC#1 | — | N/A | unit (smoke) | `cargo test -p ferro-projections --lib render::sketch::mobile` | ❌ W0 | ⬜ pending |
| 208-02-01 | 02 | 2 | COMP-05 SC#3,#4,#5 | — | N/A (Markdown doc) | doc review | manual + `test -f docs/research/comp-05-cross-modality-vocabulary-sketch.md` | ❌ W0 | ⬜ pending |
| 208-02-02 | 02 | 2 | COMP-05 SC#2 | — | N/A | invariant check | `git diff --exit-code ferro-projections/src/intent.rs ferro-projections/src/derive.rs` | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-projections/src/render/sketch/mod.rs` — `pub(crate)` module entry point, registered in `render/mod.rs`
- [ ] `ferro-projections/src/render/sketch/cli.rs` — `CliSummaryRenderer` + smoke test
- [ ] `ferro-projections/src/render/sketch/voice.rs` — `VoiceRenderer` + smoke test
- [ ] `ferro-projections/src/render/sketch/mobile.rs` — `MobileCardRenderer` + smoke test
- [ ] `docs/research/comp-05-cross-modality-vocabulary-sketch.md` — analysis document

*Rust built-in harness needs no install; the source files above are the Wave 0 stubs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Document covers all 7 intents × 3 modalities | COMP-05 SC#3 | Prose coverage — not machine-checkable | Read the doc; confirm a 7×3 matrix and ≥1 named vocabulary tension |
| "v14.0 implications" section lists open questions | COMP-05 SC#4 | Editorial judgment | Confirm a section with concrete Channel Projection open questions |
| "discovered weaknesses" non-empty with a real tension | COMP-05 SC#5 | Editorial judgment | Confirm the section names a workaround/awkward output the sketch forced |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 45s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
