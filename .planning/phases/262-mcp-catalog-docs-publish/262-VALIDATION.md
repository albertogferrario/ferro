---
phase: 262
slug: mcp-catalog-docs-publish
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-26
---

# Phase 262 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test (`#[test]`) + mdBook build + `cargo publish --dry-run` |
| **Config file** | none (workspace-level `cargo test`); `docs/book.toml` for docs |
| **Quick run command** | `cargo test -p ferro-mcp generation_context && cargo test -p ferro-json-ui catalog` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~30–90s quick; full `--all-features` several minutes (serialize; check disk) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-mcp -- tools::generation_context 2>&1 | tail -5`
- **After every plan wave:** `cargo test --all-features 2>&1 | grep "test result"`
- **Before `/gsd-verify-work`:** Full CI-exact gate green (fmt + clippy `--all-features` + test `--all-features` + `cargo doc -D warnings` + mdBook build)
- **Max feedback latency:** ~90 seconds (quick run)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| SC-1 catalog canonical | catalog/publish plan | 1 | LIVE-04 | — | N/A (introspection integrity) | unit | `cargo test -p ferro-json-ui -- catalog::tests::builtin_types_count_drift_guard --all-features` | ✅ catalog.rs:1294 | ⬜ pending |
| SC-1 mirror | catalog/publish plan | 1 | LIVE-04 | — | N/A | unit | `cargo test -p ferro-mcp -- tools::json_ui_catalog::tests::test_all_components_present --all-features` | ✅ json_ui_catalog.rs:411 | ⬜ pending |
| SC-2 drift guard | generation_context plan | 1 | LIVE-04 | — | Prose names must match authoritative registries (no drift) | unit | `cargo test -p ferro-mcp -- tools::generation_context::tests::live_projection_drift_guard --all-features` | ❌ W0 (create) | ⬜ pending |
| SC-2 sections present | generation_context plan | 1 | LIVE-04 | — | New `live_projection` guidance field non-empty | unit | `cargo test -p ferro-mcp -- tools::generation_context::tests::test_generation_context_has_all_sections --all-features` | ✅ (update to cover new field) | ⬜ pending |
| SC-3 mdBook build | docs plan | 1 | LIVE-04 | — | Docs build clean (`create-missing=false` — new SUMMARY entry needs a file) | smoke | `mdbook build docs/` | ✅ docs/book.toml | ⬜ pending |
| SC-4 publish dry-run | publish gate plan | 2 | LIVE-04 | — | Package builds + version > crates.io max | gate | `cargo publish -p ferro-rs --dry-run` | ✅ Cargo.toml | ⬜ pending |
| SC-4 full CI gate | publish gate plan | 2 | LIVE-04 | — | fmt/clippy/test/doc all green | gate | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp/src/tools/generation_context.rs` — add `LiveProjectionGuidance` struct + `live_projection` field + `execute()` assembly + `live_projection_drift_guard` test (asserts `"LiveFragment"` in `global_catalog()`; `"data-live-fragment"` + `"data-channel"` in `FERRO_RUNTIME_JS`; macro names present in their exports). Mirror the existing `RegisterCompositionGuidance` (generation_context.rs:99-130) + `register_composition_drift_guard` (generation_context.rs:559-638).

*All other test infrastructure exists — only the new drift-guard test needs creating; SC-1 guards are already green.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Operator-gated crates.io publish | LIVE-04 (SC-4) | Publishing is irreversible + operator-authorized; not run autonomously | Present pre-publish checklist (gate results, resolved version, what ships incl. ferro-payments rider if any); on operator go, push master via gh HTTPS helper, then verify ferro-rs (and ferro-payments if riding) on crates.io / gh API |
| mdBook binary availability | LIVE-04 (SC-3) | `mdbook` CLI presence not confirmed locally (research MEDIUM confidence) | Confirm `mdbook --version` resolves before relying on `mdbook build docs/`; install if absent |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (the one new drift-guard test — created in-phase by Plan 01 Task 1; `wave_0_complete: false` is correct, the test does not pre-exist execution)
- [x] No watch-mode flags
- [x] Feedback latency < 90s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-07-26
