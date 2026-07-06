---
phase: 211-comp-04-time-to-working-app-benchmark
reviewed: 2026-06-13T00:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - ferro-cli/tests/benchmark_new_project.rs
  - ferro-cli/Cargo.toml
  - ferro-cli/tests/fixtures/benchmark/Dockerfile
  - ferro-cli/tests/fixtures/benchmark/RESULTS.md
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 211: Code Review Report

**Reviewed:** 2026-06-13T00:00:00Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Reviewed the COMP-04 time-to-working-app benchmark artifacts: a gated `#[test]`
wall-clock harness (`benchmark_new_project.rs`), the `criterion` dev-dependency
addition (`Cargo.toml`), a cold-cache `Dockerfile`, and the `RESULTS.md` finding
doc. The harness is well-structured and its intentional design choices are
documented inline (FERRO_BENCH gate, `#[ignore]`, `#[test]`-not-`[[bench]]`, the
Step 5 build assertion that surfaces W1, the make:scaffold flag-ordering
workaround). No security issues and no correctness bugs that would produce a wrong
measurement on the realistic path.

The findings below are reproducibility and robustness concerns. The two warnings
are the unpinned `cargo install ferro-cli` in the Dockerfile (which contradicts the
exact-version reproducibility claim in RESULTS.md) and an unverified gap between
the warm harness's `cargo build` cache state and the cold-cache claim. The rest are
informational hardening suggestions.

## Warnings

### WR-01: Dockerfile installs unpinned `ferro-cli`, contradicting RESULTS.md version claim

**File:** `ferro-cli/tests/fixtures/benchmark/Dockerfile:36`
**Issue:** `RUN cargo install ferro-cli` installs whatever is the latest published
version at image-build time. RESULTS.md (lines 7-9) records a specific
environment — `ferro-cli version 0.2.55` / `ferro-rs 0.2.55` — as the baseline.
Because the image is not pinned to that version, rebuilding the container after a
new crates.io release silently produces a different baseline than the one the
results document attests to. The Dockerfile comment (lines 32-35) even states the
installed CLI version "should match the ferro-rs version the scaffolded project
will resolve" — but nothing enforces that match. This is the same reproducibility
class as the digest-pinning note already acknowledged for the base image.
**Fix:** Pin the install to the documented baseline so the image and RESULTS.md
agree:
```dockerfile
# Keep in sync with RESULTS.md "ferro-cli version".
RUN cargo install ferro-cli --version 0.2.55 --locked
```
`--locked` additionally makes the CLI's own dependency resolution reproducible.

### WR-02: Warm harness `cargo build` cache state differs from the cold-cache claim — verify before publishing the baseline

**File:** `ferro-cli/tests/fixtures/benchmark/RESULTS.md:87-92`
**Issue:** RESULTS.md documents two run paths (cold Docker, warm `FERRO_BENCH=1`
cargo test). The warm path's Step 5 (`cargo build` in a tmpdir,
`benchmark_new_project.rs:142-149`) resolves `ferro-rs` from crates.io but runs
against the **host's** pre-warmed Cargo registry and a host-shared `~/.cargo`
download cache. Its wall-clock for Step 5 is therefore not comparable to the cold
container's Step 5, yet both feed the single "time to working app" narrative. The
note correctly states the warm path reproduces W1's compile failure, but does not
caution that the warm Step 5 *duration* must not be quoted as a cold-cache number.
Since W1 makes Step 5 fail today the timing is moot, but once W1 is fixed this
becomes a live foot-gun for whoever fills in the duration table.
**Fix:** Add an explicit caveat in the Notes section that the warm harness's Step 5
duration is registry-warm and only the Docker run yields a publishable cold-cache
`cargo build` time; the warm path is for correctness (build exit code), not for the
headline timing.

## Info

### IN-01: `iters as u32` truncates a `u64` loop count

**File:** `ferro-cli/tests/benchmark_new_project.rs:167`
**Issue:** `total / iters as u32` casts the `u64` `iters` (criterion's
`iter_custom` closure parameter) to `u32`. With `sample_size(10)` this is never a
problem in practice, but a lossy `u64 -> u32` cast in arithmetic is a latent
foot-gun if the sample size or measurement strategy ever changes, and it is the
kind of cast clippy's pedantic lints flag.
**Fix:** Use `Duration::div_f64` or cast the dividend path through `u64`/`f64`:
```rust
println!(
    "Total (avg over {iters} iters):  {:?}",
    total.div_f64(iters as f64)
);
```

### IN-02: `code` is bound on every step solely to interpolate into the assert message

**File:** `ferro-cli/tests/benchmark_new_project.rs:38-39` (and 49-52, 76-79, 99-103, 123-126, 137, 147-148)
**Issue:** Each step does `let code = status.code();` then immediately
`assert!(status.success(), "... {code:?}")`. The intermediate binding is repeated
seven times and adds no value over inlining. Minor duplication / noise.
**Fix:** Inline the call in the assert message:
```rust
assert!(status.success(), "ferro new exited non-zero: {:?}", status.code());
```
This removes seven `let code` lines without changing behavior.

### IN-03: Per-step spawn + assert is copy-pasted seven times

**File:** `ferro-cli/tests/benchmark_new_project.rs:31-149`
**Issue:** Steps 1-5 (with 3a/3b/3c) follow an identical shape: record `Instant`,
spawn a `Command`, assert success, capture `elapsed()`. Three of those (the
`make:scaffold` calls) differ only in entity name and field list. The repetition
makes the file longer and means a fix to the timing/assert pattern must be applied
in seven places.
**Fix (optional, readability only):** Extract a helper that times a labeled command,
e.g. `fn timed(label: &str, dir: &Path, args: &[&str]) -> Duration`, and drive the
three scaffolds from a small table:
```rust
for (name, f1, f2) in [
    ("Article", "title:string", "body:text"),
    ("Product", "name:string", "price:float"),
    ("Order",   "status:string", "total:float"),
] { /* timed("make:scaffold", &project_dir, &["make:scaffold","--no-smart-defaults","-q","-y","--api",name,f1,f2]) */ }
```
Leaving the explicit unrolled form is also defensible for a benchmark where each
step's timing line must stay individually visible — flagged as info only.

### IN-04: Docker `CMD` shell sequence relies on `&&` chaining without `set -euo pipefail`

**File:** `ferro-cli/tests/fixtures/benchmark/Dockerfile:44-62`
**Issue:** The benchmark sequence is a single `bash -c` string chained with `&&`,
so a non-zero step does short-circuit the rest (acceptable). However there is no
`set -euo pipefail`; an unset-variable typo in a future edit (e.g. a mistyped
`$SECONDS`/`$T0`) would expand to empty and silently produce a wrong arithmetic
result rather than failing. For a measurement container, fail-fast on the shell
level is cheap insurance.
**Fix:** Prefix the command body with `set -euo pipefail &&` (or start the `bash -c`
string with `set -euo pipefail;`). This keeps the existing `&&` semantics while
catching unset-variable and pipe-failure mistakes in later edits.

---

_Reviewed: 2026-06-13T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
