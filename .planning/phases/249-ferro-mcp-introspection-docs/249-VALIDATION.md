---
phase: 249
slug: ferro-mcp-introspection-docs
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-08-15
audited: 2026-08-15
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

Verified via `cargo test -p ferro-mcp list_services` on 2026-08-15: **7 passed, 0 failed**
(the six planned inline tests plus a bonus `extract_service_impl_name_positional_and_named`).
Tests live inline in `ferro-mcp/src/tools/list_services.rs` (`#[cfg(test)] mod tests`, L594).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Test Fn(s) | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|------------|-------------------|-------------|--------|
| 249-01-1/2 | 01 | 1 | OFFLOAD-06 (MCP) | T-249-02 | Offload-attr detection returns declared queue, default `"default"` when arg absent | unit | `detect_offload_attr_bare_returns_default`, `detect_offload_attr_reads_declared_queue` | `cargo test -p ferro-mcp detect_offload` | ✅ inline | ✅ green |
| 249-01-1/2 | 01 | 1 | OFFLOAD-06 (MCP) | — | Method-param extraction is bracket-aware + owned-type substitution (`Vec<T>`, `&str`→`String`) | unit | `extract_method_params_bracket_aware`, `extract_method_params_owned_substitution` | `cargo test -p ferro-mcp extract_method_params` | ✅ inline | ✅ green |
| 249-01-1/2 | 01 | 1 | OFFLOAD-06 (MCP) | T-249-03 | Full static parse recovers both offload methods from a two-method fixture | unit | `scan_offload_methods` | `cargo test -p ferro-mcp scan_offload_methods` | ✅ inline | ✅ green |
| 249-01-1/2 | 01 | 1 | OFFLOAD-06 (MCP) | — | Non-offload service output byte-for-byte unchanged (additive-only, D-02) | unit | `plain_service_unchanged` | `cargo test -p ferro-mcp plain_service_unchanged` | ✅ inline | ✅ green |
| 249-02-1/2 | 02 | 2 | OFFLOAD-06 (docs) | T-249-04 | `offload.md` exists and is registered in `docs/src/SUMMARY.md` nav | manual | — | doc existence + `grep offload.md docs/src/SUMMARY.md` | ✅ present | ✅ verified |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `ferro-mcp/src/tools/list_services.rs` — `#[cfg(test)] mod tests` block (L594) covering the four MCP unit behaviors above (inline-`mod tests` pattern from `route_dependencies.rs`). No new test file — tests live inline with the module.
- [x] Test fixtures: `scan_offload_methods` builds a temp-dir source tree with two `#[offload]` methods (one with `queue = "reports"`, one bare) plus a non-offload method, and asserts `methods.len() == 2`.

*Existing infrastructure (`cargo test`) covers execution; the module currently has zero tests, so the inline test block is the only Wave 0 addition.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `offload.md` registered in nav and renders | OFFLOAD-06 (docs) | mdBook nav wiring + prose correctness is not unit-testable | Confirm `docs/src/features/offload.md` exists, appears in `docs/src/SUMMARY.md`, and `queues.md`/`deployments.md` cross-links resolve. Optionally `mdbook build docs` succeeds. |
| Scaling-model / limitations claims are code-accurate | OFFLOAD-06 (docs) | Four spec-sourced claims are tagged ASSUMED in RESEARCH.md | During authoring, grep-verify each ASSUMED claim (worker CLI shape, `serve --no-worker`, default queue name, no built-in metrics) against phases 244–248 code before committing prose. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 10s (list_services suite finished in 0.01s)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** verified 2026-08-15

---

## Validation Audit 2026-08-15

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

State A audit: the four automated MCP behaviors were already covered by inline
tests, confirmed green empirically (`cargo test -p ferro-mcp list_services` →
7 passed, 0 failed). The single docs behavior remains manual-only (mdBook nav +
prose correctness are not unit-testable) with its artifact and nav entry present.
No gaps to fill — no auditor spawn required. `nyquist_compliant: true`.
