---
phase: 249
slug: ferro-mcp-introspection-docs
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-08-15
---

# Phase 249 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: 249-RESEARCH.md §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` (synchronous helpers — no `#[tokio::test]`, so the suite-collapse gotcha does not apply) |
| **Config file** | `Cargo.toml` (no separate test config) |
| **Quick run command** | `cargo test -p ferro-mcp list_services` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~5s quick (ferro-mcp unit tests); full suite bounded by workspace build |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-mcp`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~5 seconds (quick), full suite before phase gate

---

## Per-Task Verification Map

Task IDs are assigned by the planner. The rows below bind each phase behavior to an
automated (or manual) check; the planner maps them onto concrete plan/task numbers.
Expected plan split: **Plan 01 = MCP introspection (deliverable A)**, **Plan 02 = docs (deliverable B)**.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 249-01-* | 01 | 1 | OFFLOAD-06 (MCP) | — | Offload-attr detection returns declared queue, default `"default"` when arg absent | unit | `cargo test -p ferro-mcp detect_offload` | ❌ W0 | ⬜ pending |
| 249-01-* | 01 | 1 | OFFLOAD-06 (MCP) | — | Method-param extraction is bracket-aware (handles `Vec<T>`, tuples) | unit | `cargo test -p ferro-mcp extract_method_params` | ❌ W0 | ⬜ pending |
| 249-01-* | 01 | 1 | OFFLOAD-06 (MCP) | — | Full static parse recovers both offload methods from a two-method fixture | unit | `cargo test -p ferro-mcp scan_offload_methods` | ❌ W0 | ⬜ pending |
| 249-01-* | 01 | 1 | OFFLOAD-06 (MCP) | — | Non-offload service output byte-for-byte unchanged (additive-only, D-02) | unit | `cargo test -p ferro-mcp plain_service_unchanged` | ❌ W0 | ⬜ pending |
| 249-02-* | 02 | 2 | OFFLOAD-06 (docs) | — | `offload.md` exists and is registered in `docs/src/SUMMARY.md` nav | manual | doc existence + `grep offload.md docs/src/SUMMARY.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-mcp/src/tools/list_services.rs` — add `#[cfg(test)] mod tests` block covering the four MCP unit behaviors above (matching the inline-`mod tests` pattern in `route_dependencies.rs`). No new test file — tests live inline with the module.
- [ ] Test fixtures: a sample source string with two `#[offload]` methods (one with `queue = "…"`, one without), interleaved non-offload methods, and varied param arities (0 / 1 / N, including a bracketed generic type).

*Existing infrastructure (`cargo test`) covers execution; the module currently has zero tests, so the inline test block is the only Wave 0 addition.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `offload.md` registered in nav and renders | OFFLOAD-06 (docs) | mdBook nav wiring + prose correctness is not unit-testable | Confirm `docs/src/features/offload.md` exists, appears in `docs/src/SUMMARY.md`, and `queues.md`/`deployments.md` cross-links resolve. Optionally `mdbook build docs` succeeds. |
| Scaling-model / limitations claims are code-accurate | OFFLOAD-06 (docs) | Four spec-sourced claims are tagged ASSUMED in RESEARCH.md | During authoring, grep-verify each ASSUMED claim (worker CLI shape, `serve --no-worker`, default queue name, no built-in metrics) against phases 244–248 code before committing prose. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
