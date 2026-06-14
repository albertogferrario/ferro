---
phase: 228
slug: readme-and-scaffold-doc-sweep
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-15
---

# Phase 228 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Docs/scripts phase: the only Rust-tested artifact is the scaffold README template render. All
> other verification is grep-based (brew-first ordering, absence of phantom commands / stale pins)
> and cross-reference against the install-doc oracle.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` via cargo (template render only); grep assertions for docs/scripts |
| **Config file** | none — workspace cargo defaults |
| **Quick run command** | `cargo test -p ferro-cli -- test_readme_substitution` |
| **Full suite command** | `cargo test -p ferro-cli` (scoped — NOT the full workspace suite) |
| **Estimated runtime** | ~30–60 seconds (ferro-cli only) |

---

## Sampling Rate

- **After the README.md.tpl edit:** Run `cargo test -p ferro-cli -- test_readme_substitution`
- **After all edits:** Run `cargo test -p ferro-cli`
- **Before sign-off:** template test green + all grep verifications pass
- **Max feedback latency:** ~60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 228-scaffold-readme | tpl | 1 | — | — | N/A (docs) | unit | `cargo test -p ferro-cli -- test_readme_substitution` | ✅ `ferro-cli/src/templates/mod.rs:582` | ⬜ pending |
| 228-root-readme | — | 1 | — | — | N/A (docs) | grep | `grep -n "v0.2.0\|v12.0 spec-driven" README.md` returns 0 | ✅ | ⬜ pending |
| 228-scripts | — | 1 | — | — | N/A (docs) | grep | `grep -rn "ferro migrate\|cargo run -- migrate" scripts/` returns 0 | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — `test_readme_substitution` already exists
(`ferro-cli/src/templates/mod.rs`). No new test scaffolding needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tap-repo README draft accuracy | — | Targets a separate, non-local repo; cannot be auto-tested here | Human reviews the draft before pasting into `albertogferrario/homebrew-ferro` |

---

## Validation Sign-Off

- [ ] Scaffold template edit verified by `test_readme_substitution`
- [ ] Root README + scripts verified by grep (no phantom commands, no stale pins, brew-first)
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s

**Approval:** pending
