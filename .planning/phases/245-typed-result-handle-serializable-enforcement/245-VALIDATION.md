---
phase: 245
slug: typed-result-handle-serializable-enforcement
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-08-13
---

# Phase 245 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `245-RESEARCH.md` §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `trybuild 1` (compile-fail UI tests, already a `ferro-macros` dev-dependency) + `cargo test` (unit tests) |
| **Config file** | `ferro-macros/Cargo.toml` (`trybuild = "1"`); harness `ferro-macros/tests/offload_macro.rs` (glob `tests/ui/offload/fail/*.rs`) |
| **Quick run command** | `cargo test -p ferro-macros --test offload_macro && cargo test -p ferro-queue` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds quick (macro + queue crates); full suite several minutes |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p ferro-macros --test offload_macro && cargo test -p ferro-queue`
- **After every plan wave:** Run the full suite (`cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds (quick run)

> `.stderr` snapshots are rustc-version-sensitive. Regenerate deliberately with `TRYBUILD=overwrite cargo test -p ferro-macros --test offload_macro`, then inspect the diff — never blind-overwrite.

---

## Per-Task Verification Map

Task IDs are assigned by the planner; the rows below bind each phase requirement to its
automated command and Wave 0 fixture. The planner/executor MUST map each task to one of these
requirement rows (Threat Ref column set once the threat model is authored in PLAN.md).

| Req ID | Requirement / Behavior | Threat Ref | Test Type | Automated Command | File Exists | Status |
|--------|------------------------|------------|-----------|-------------------|-------------|--------|
| OFFLOAD-02a | `.offload()` returns `OffloadHandle<T>` typed on the method's success type (not the bare value) | — | unit / compile-pass | `cargo test -p ferro-macros --test offload_macro` (pass fixture) | ❌ W0 (new pass fixture) | ⬜ pending |
| OFFLOAD-02b | Non-`Serialize`/`DeserializeOwned` **parameter** fails to compile with the branded, type-naming diagnostic | — | compile-fail (trybuild) | `cargo test -p ferro-macros --test offload_macro` | ❌ W0 (`non_serializable_param.rs` + `.stderr`) | ⬜ pending |
| OFFLOAD-02c | Non-`Serialize`/`DeserializeOwned` **return** type fails to compile with the branded, type-naming diagnostic | — | compile-fail (trybuild) | `cargo test -p ferro-macros --test offload_macro` | ❌ W0 (`non_serializable_return.rs` + `.stderr`) | ⬜ pending |
| OFFLOAD-02d | `OffloadHandle<T>::key()` / `.id()` returns the minted UUID v4 string | — | unit | `cargo test -p ferro-queue` | ❌ W0 (unit test on `HandleKey`) | ⬜ pending |
| OFFLOAD-02e | `OffloadHandle<T>` serde round-trips regardless of `T: !Serialize` (phantom is `#[serde(skip)]`) | — | unit | `cargo test -p ferro-queue` | ❌ W0 (serde round-trip test) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `ferro-queue/src/offload.rs` — new module (`OffloadHandle`, `HandleKey`, `Offloadable`, `OffloadSerializable`) with unit tests: `HandleKey::new()` yields a v4 UUID; `OffloadHandle<T>` serde round-trip with a non-`Serialize` `T`
- [ ] `ferro-macros/tests/ui/offload/fail/non_serializable_param.rs` + `.stderr`
- [ ] `ferro-macros/tests/ui/offload/fail/non_serializable_return.rs` + `.stderr`
- [ ] `ferro-macros/tests/ui/offload/pass/` — one new pass fixture proving `.offload()` returns `OffloadHandle<Output>`

*Existing infrastructure (trybuild harness `offload_macro.rs`, cargo test) covers the run mechanics; only the fixtures/tests above are new.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Branded diagnostic reads as intended (message + note wording, `{Self}` interpolation, isolation-boundary framing) | OFFLOAD-02b/c | `.stderr` equality proves *content* but not *readability*; the human-facing quality of the message is a judgement call | After generating `.stderr` via `TRYBUILD=overwrite`, read both fail-fixture `.stderr` files and confirm the primary error line is the branded `OffloadSerializable` message naming the offending type — not serde's default `the trait bound ... is not satisfied` |

---

## Validation Sign-Off

- [ ] All tasks map to an `<automated>` verify command or a Wave 0 fixture dependency
- [ ] Sampling continuity: no 3 consecutive tasks without an automated verify
- [ ] Wave 0 covers all MISSING references (the four fixture/module gaps above)
- [ ] No watch-mode flags in any command
- [ ] Feedback latency < 120s (quick run)
- [ ] `nyquist_compliant: true` set in frontmatter once the above hold

**Approval:** pending
