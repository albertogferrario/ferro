---
phase: 183
slug: ferro-bundle-capability-new-crate
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-06-06
---

# Phase 183 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in test harness) |
| **Config file** | None — convention-driven via `#[cfg(test)]` blocks and `tests/*.rs` integration binaries |
| **Quick run command** | `cargo test -p ferro-bundle` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~3–8s for the crate's tests; ~2–4 min for full workspace suite |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-bundle`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets --all-features -- -D warnings && cargo test --all-features`
- **Phase gate (before D-12 publish bootstrap):** Full workspace suite green AND `cargo publish -p ferro-bundle --dry-run` exits 0 from local terminal
- **Max feedback latency:** ~10 seconds for the quick command

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 183-01-* | 01 (scaffold) | 1 | BUNDLE-05, BUNDLE-06 | — | N/A (metadata only) | manual / grep | `grep -F 'ferro-bundle' Cargo.toml`; `grep -F 'ferro-bundle' .github/workflows/publish.yml`; `grep -F 'do not fold' ferro-bundle/README.md` | ❌ W0 | ⬜ pending |
| 183-02-* | 02 (core impl) | 1 | BUNDLE-01, BUNDLE-04 | T-183-V5, T-183-V14 | Path-traversal safe (exact-match registry); octet-stream default | unit (lib.rs `#[cfg(test)]`) | `cargo test -p ferro-bundle hash_is_deterministic default_content_type_is_octet_stream duplicate_name_panics` | ❌ W0 | ⬜ pending |
| 183-03-* | 03 (integration tests) | 2 | BUNDLE-02 cold, BUNDLE-02 304, BUNDLE-03 | — | Strong ETag + immutable cache | integration | `cargo test -p ferro-bundle --test serve_cold --test serve_304 --test alias_redirect` | ❌ W0 | ⬜ pending |
| 183-04-* | 04 (bootstrap doc) | 3 | BUNDLE-06 (manual `cargo publish -p ferro-bundle` first run) | — | N/A | manual (planner SUMMARY records execution) | (no automated test — first-publish friction documented in SUMMARY.md) | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Note on task IDs:** Above is guideline mapping per RESEARCH.md. Planner finalizes exact task IDs in step 8 of plan-phase.

---

## Wave 0 Requirements

All test files are new — Phase 183 builds the test suite from scratch.

- [ ] `ferro-bundle/src/lib.rs` — `#[cfg(test)] mod tests` block (hash determinism, default content-type, duplicate-name `#[should_panic]`).
- [ ] `ferro-bundle/tests/serve_cold.rs` (synthetic Request + asserts 200 + cache headers).
- [ ] `ferro-bundle/tests/serve_304.rs` (synthetic Request with `If-None-Match` + asserts 304).
- [ ] `ferro-bundle/tests/alias_redirect.rs` (synthetic Request to alias path + asserts 301 + Location header).

*Existing `cargo test` infrastructure covers all phase requirements. No framework install needed.*

**Risk:** RESEARCH §Open Question #3 — synthetic `Request` construction may be too heavy for unit tests. Planner may fold integration tests into `lib.rs` via a private helper `serve_inner(path: &str, if_none_match: Option<&str>)` that bypasses `Request` entirely. Both shapes are acceptable.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| First publish of `ferro-bundle` to crates.io | BUNDLE-06 (D-12 manual bootstrap) | CI publish token has publish-update only, not publish-new (project memory `project_ferro_publish_token_scoping.md`) | After Phase 183 merges to master: from local terminal, run `cargo publish -p ferro-bundle`. Future versions ship via Wave-N CI automatically. |
| Publish wave entry correctness | BUNDLE-06 | Wave structure must respect dep ordering (`ferro-rs` in Wave 2 → `ferro-bundle` cannot be Wave 1B per RESEARCH §critical correction) | After planner picks the wave: `grep -B 2 -A 5 'ferro-bundle' .github/workflows/publish.yml` shows correct ordering. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (Plan 04 bootstrap is manual; planner ensures Plans 01–03 are fully automated)
- [ ] Wave 0 covers all MISSING references (4 new test files added in Plans 01–03)
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s for quick command
- [ ] `nyquist_compliant: true` set in frontmatter after planner finalizes task IDs

**Approval:** pending

---

## Notes on Phase 183's validation shape

Phase 183 is a pure-Rust scaffold-+-implementation phase: a new crate, new tests, no UI, no external browser verification. Validation is fully cargo-test-driven plus a manual `cargo publish` bootstrap step.

The integration tests (`serve_cold`, `serve_304`, `alias_redirect`) depend on the consumer-side approach to synthetic `Request` construction — see RESEARCH.md Open Question #3. Planner picks: (a) synthetic Request via `hyper::Body::from(...)` shim, or (b) bypass via a private `serve_inner(path, if_none_match)` helper. Both honor BUNDLE-02/03 success criteria.

Manual bootstrap (D-12) is the only non-automated verification step. The planner documents the exact command in Plan 04's SUMMARY so the friction-loop pattern (manual bootstrap → CI handles ongoing releases) is preserved for future new-crate phases.
