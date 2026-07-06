---
phase: 215
slug: non-visual-rendering-context-basecontext-intent-extensions
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-13
---

# Phase 215 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust, inline `#[cfg(test)] mod tests`) |
| **Config file** | none — workspace Cargo.toml; tests live in-crate |
| **Quick run command** | `cargo test -p ferro-projections` |
| **Full suite command** | `cargo test -p ferro-projections -p ferro-json-ui -p ferro-mcp -p ferro-mcp-server` |
| **Estimated runtime** | ~60–120 seconds (incremental; cold build longer) |

> Per project convention, the full pre-commit gate is
> `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.
> Run CPU-heavy gates one at a time (no parallel cargo runs).

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p <crate-touched>` for the crate modified by that task
- **After every plan wave:** Run the full suite command above
- **Before `/gsd-verify-work`:** `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and the full suite must all be green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 215-01-xx | 01 | 1 | CHAN-01 | — | `BaseContext::default()` yields empty guards + `Verbosity::Full` (preserves current visual behavior) | unit | `cargo test -p ferro-projections base_context` | ✅ | ⬜ pending |
| 215-01-xx | 01 | 1 | CHAN-01 | — | Action with guard mapped `false` is filterable; absent guard renders (semantics) | unit | `cargo test -p ferro-projections evaluated_guards` | ❌ W0 | ⬜ pending |
| 215-02-xx | 02 | 1 | CHAN-02 | — | `Intent::label()` returns stable snake_case string; `Custom(s)` → `s` | unit | `cargo test -p ferro-projections intent_label` | ❌ W0 | ⬜ pending |
| 215-02-xx | 02 | 1 | CHAN-02 | — | `Error::NoIntents` exists and renders a typed message (not `"unknown"`) | unit | `cargo test -p ferro-projections no_intents` | ❌ W0 | ⬜ pending |
| 215-03-xx | 03 | 2 | CHAN-02 | — | No renderer/tool uses `format!("{:?}", *.intent)` for a label (4 sites migrated) | grep | `! grep -rn 'format!("{:?}", *[a-z_.]*intent' ferro-mcp/src ferro-json-ui/src` | ✅ | ⬜ pending |
| 215-03-xx | 03 | 2 | CHAN-01 | — | `VisualContext` embeds `base: BaseContext`; ferro-json-ui + ferro-mcp + ferro-mcp-server build and tests pass unchanged | build+test | `cargo test -p ferro-json-ui -p ferro-mcp -p ferro-mcp-server` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-projections/src/render/mod.rs` `#[cfg(test)] mod tests` — new cases for `evaluated_guards` default + `Verbosity::Full` default
- [ ] `ferro-projections/src/intent.rs` `#[cfg(test)] mod tests` — new cases for `Intent::label()` (all 7 known variants + `Custom`)
- [ ] `ferro-projections/src/error.rs` `#[cfg(test)] mod tests` — new case asserting `Error::NoIntents` message

*Existing inline-test infrastructure covers all phase requirements — no framework install needed; tests are added to existing `mod tests` blocks.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| — | — | — | — |

*All phase behaviors have automated verification (unit tests + grep assertions + cargo build/test).*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
