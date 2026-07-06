---
phase: 145
slug: ferro-serve-manual-reload-key-and-watch-supervisor
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-22
---

# Phase 145 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `tempfile` 3.24 (already a dev-dependency of ferro-cli) |
| **Config file** | none — `cargo test` driven by `ferro-cli/Cargo.toml` + workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p ferro-cli` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60 seconds (quick) / ~5–8 minutes (full workspace clippy + all-features) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-cli`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds (quick) / 480 seconds (full)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD-filled-by-planner | 01 | 1 | D-XX | — | — | unit/integration | `cargo test -p ferro-cli --test {name}` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Planner fills this table with one row per task after plans are generated. Source of truth for Nyquist enforcement — each decision D-01..D-38 from CONTEXT.md must map to either an automated row here OR a Manual-Only Verifications row below.*

---

## Wave 0 Requirements

- [ ] `ferro-cli/tests/fixtures/minimal-serve/` — minimal Ferro project fixture (Cargo.toml + src/main.rs + trivial binary that `cargo run` completes in under a second)
- [ ] `ferro-cli/tests/serve_supervisor.rs` — integration test module for `serve` command behaviors (D-36 items)
- [ ] Inline unit test module in `ferro-cli/src/commands/serve.rs` (`#[cfg(test)] mod tests { ... }`) — banner rendering, key classification, debouncer coalescing (D-35 items)

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `r` cancels an in-flight build and restarts | D-09 | Requires observing that `cargo run` is actually killed mid-compile, then a fresh compile starts — timing and process-tree state is hard to assert reliably from Rust. | (1) Introduce a slow compilation (touch a file that forces full rebuild). (2) `ferro serve`. (3) Press `r` while `[backend]` is still printing `Compiling…`. (4) Verify: first cargo process disappears from `ps`, a new one starts, banner never prints twice. |
| Backend non-zero exit does NOT trigger auto-respawn | D-12 | Requires inducing a compile error and observing that the supervisor stays idle until the next trigger. | (1) Introduce a syntax error. (2) `ferro serve`. (3) Observe `cargo` fails, supervisor logs exit code, no respawn. (4) Fix error, press `r`, observe backend comes back. |
| Types regen is uninterruptible | D-18 | Requires observing that a trigger arriving during types regen is picked up on the next cycle (not mid-regen). | (1) Make type regen artificially slow (add large fixture module). (2) `ferro serve`. (3) Press `r`, then `r` again within <1s. (4) Observe: first regen completes, one additional cycle runs. |
| Raw-mode restored on panic / Ctrl+C | D-25 | Requires inducing a panic in a spawned thread and verifying terminal state afterward via `stty -a` diff. Unstable under CI (terminal state varies by runner). | (1) `stty -a > /tmp/before`. (2) `ferro serve`. (3) Ctrl+C (or send SIGTERM). (4) `stty -a > /tmp/after`. (5) `diff /tmp/before /tmp/after` — expect zero-line diff. |
| `enable_raw_mode()` failure fallback | D-26 | Requires artificially failing `enable_raw_mode()` at runtime — no clean injection point without modifying production code. | Manual only on machines where raw mode is known to fail (e.g. CI sandboxes); expect warning log + serve continues without `r` key. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (fixture project, integration test file, inline unit module)
- [ ] No watch-mode flags (e.g. `cargo watch`, `cargo-nextest --watch`)
- [ ] Feedback latency < 60s (quick) / < 480s (full)
- [ ] `nyquist_compliant: true` set in frontmatter after planner fills per-task table

**Approval:** pending
