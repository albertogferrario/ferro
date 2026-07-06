---
phase: 251
slug: component-variant-discipline-interactive-state-pass
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-03
---

# Phase 251 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness via cargo (workspace) |
| **Config file** | Cargo.toml (workspace); no separate test config |
| **Quick run command** | `cargo test -p ferro-json-ui` |
| **Full suite command** | `cargo test --all-features` (CI-exact; plus `cargo fmt --all -- --check` and `cargo clippy --all --all-targets --all-features -- -D warnings`) |
| **Estimated runtime** | ~60s crate-scoped; several minutes full suite |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-json-ui` (+ `cargo fmt --all -- --check`)
- **After every plan wave:** Run `cargo clippy --all --all-targets --all-features -- -D warnings` + `cargo test --all-features` (check `df` disk headroom first — known ENOSPC risk)
- **Before `/gsd-verify-work`:** Full CI-exact triple green + ferro-base.css regenerated + Chrome MCP visual pass
- **Max feedback latency:** ~120 seconds (crate-scoped runs); serialize CPU-intensive runs — never parallelize cargo invocations

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 251-01-01 | 01 | 1 | DS-03 | — | old enum values rejected at spec-parse | build + grep | `cargo build -p ferro-json-ui` + retired-identifier grep | ✅ | ⬜ pending |
| 251-01-02 | 01 | 1 | DS-03 | — | N/A | build | `cargo build --workspace` | ✅ | ⬜ pending |
| 251-01-03 | 01 | 1 | DS-03 | — | N/A | unit | `cargo test -p ferro-json-ui component::` / `variant_enums_strum` | ✅ | ⬜ pending |
| 251-02-01 | 02 | 2 | DS-04 | — | N/A | build + grep | `cargo build -p ferro-json-ui` + ring greps | ✅ | ⬜ pending |
| 251-02-02 | 02 | 2 | DS-04 | — | N/A | build + grep | build + retired-class grep | ✅ | ⬜ pending |
| 251-02-03 | 02 | 2 | DS-04 | — | N/A | unit | `cargo test -p ferro-json-ui` | ✅ | ⬜ pending |
| 251-03-01 | 03 | 3 | DS-03 | — | N/A | unit (new drift guard) | `cargo test -p ferro-json-ui variant_tone_size_enum_sets_drift_guard` | ❌ authored in-task | ⬜ pending |
| 251-03-02 | 03 | 3 | DS-03 | — | N/A | unit + grep | `cargo test -p ferro-json-ui catalog::` + prose grep | ✅ | ⬜ pending |
| 251-03-03 | 03 | 3 | DS-03 | — | N/A | unit + grep | `cargo test -p ferro-mcp json_ui` + grep | ✅ | ⬜ pending |
| 251-04-01 | 04 | 4 | DS-03 | — | N/A | grep sweep | docs old-vocabulary grep sweep → 0 hits | ✅ | ⬜ pending |
| 251-04-02 | 04 | 4 | DS-04 | — | N/A | smoke | `scripts/gen-ferro-base-css.sh` + `grep -c "ring-ring\|duration-fast" ferro-json-ui/assets/ferro-base.css` ≥ 1 | ✅ | ⬜ pending |
| 251-04-03 | 04 | 4 | DS-03/04 | — | N/A | checkpoint:human-verify | Chrome MCP light+dark before/after screenshots | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] D-19 schema-walking drift-guard test — authored and executed within 251-03 Task 1 itself (no separate Wave 0 stub required; no `MISSING` verify references exist in any plan)
- [x] OQ-1 scope decision (action-level `variant` normalization) — RESOLVED: normalized to shared `Tone`; recorded in 251-01/251-03 `<decisions_adopted>`

*No framework install needed; all other coverage exists and is updated in place.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual parity + intended deltas, light + dark | DS-03/DS-04 | Rendered visual quality (hover/focus/disabled/motion feel) is not assertable as strings | Chrome MCP screenshots of the sample `app/` before/after, light + dark, per Phase 250 practice (251-04 Task 3 checkpoint) |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (vacuously — no MISSING markers; D-19 guard authored in 251-03-01)
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-03 (plan-checker Dimension 8 PASS, checks 8a–8e)
