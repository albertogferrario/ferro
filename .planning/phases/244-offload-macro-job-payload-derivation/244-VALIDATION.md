---
phase: 244
slug: offload-macro-job-payload-derivation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-08-13
---

# Phase 244 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source: `244-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `trybuild 1.x` (macro compilation UI tests) + `tokio::test` (round-trip) |
| **Config file** | `ferro-macros/Cargo.toml` dev-deps (trybuild already present); `ferro-queue/Cargo.toml` (tokio `full` already in dev-deps) |
| **Quick run command** | `cargo test -p ferro-macros --test offload_macro && cargo test -p ferro-queue` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~30 seconds (quick) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-macros --test offload_macro && cargo test -p ferro-queue`
- **After every plan wave:** Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

> Requirement-level map from research. Task IDs are assigned by the planner; every behavior below
> must be claimed by at least one plan task with an `<automated>` verify.

| Behavior (Req sub-ID) | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|-----------------------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| Valid `#[offload]` method emits `<Trait><Method>Job` struct (OFFLOAD-01-a) | — | OFFLOAD-01 | — | N/A | trybuild pass | `cargo test -p ferro-macros --test offload_macro` | ❌ W0 | ⬜ pending |
| `&str` param maps to owned `String` field (OFFLOAD-01-b) | — | OFFLOAD-01 | — | N/A | trybuild pass | `cargo test -p ferro-macros --test offload_macro` | ❌ W0 | ⬜ pending |
| `&mut T` param emits `compile_error!` (OFFLOAD-01-c) | — | OFFLOAD-01 | — | N/A | trybuild fail + `.stderr` | `cargo test -p ferro-macros --test offload_macro` | ❌ W0 | ⬜ pending |
| Derived Job dispatched via `dispatch(..).await` runs `handle()` (OFFLOAD-01-d) | — | OFFLOAD-01 | — | N/A | unit (`tokio::test` + serial) | `cargo test -p ferro-queue --test offload_round_trip` | ❌ W0 | ⬜ pending |
| `Result<T,E>` method: `Err(e)` → `handle()` returns job failure (OFFLOAD-01-e) | — | OFFLOAD-01 | — | N/A | unit | `cargo test -p ferro-queue --test offload_round_trip` | ❌ W0 | ⬜ pending |
| Derived Job auto-registers; `WorkerLoop::from_registry` includes it (OFFLOAD-01-f) | — | OFFLOAD-01 | — | N/A | unit | `cargo test -p ferro-queue` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-macros/tests/offload_macro.rs` — trybuild harness for `#[offload]`
- [ ] `ferro-macros/tests/ui/offload/pass/basic.rs` — minimal valid `#[offload]` fixture
- [ ] `ferro-macros/tests/ui/offload/pass/result_method.rs` — `Result<T, E>` return fixture
- [ ] `ferro-macros/tests/ui/offload/pass/ref_str_param.rs` — `&str` param fixture
- [ ] `ferro-macros/tests/ui/offload/fail/mut_ref_param.rs` + `.stderr` — `&mut T` compile-error fixture
- [ ] `ferro-queue/tests/offload_round_trip.rs` — sync-mode dispatch round-trip test

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

The one non-code concern noted in research — sensitive data serialized into a job payload when a
developer offloads a method carrying secrets/PII — is a documentation matter deferred to Phase 249,
not a control Phase 244 enforces. No manual verification required this phase.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
