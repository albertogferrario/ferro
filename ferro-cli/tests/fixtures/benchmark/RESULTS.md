# COMP-04 Time-to-Working-App Benchmark Results

## Environment

| Field | Value |
|-------|-------|
| Rust toolchain | rustc 1.96.0 (ac68faa20 2026-05-25), stable, profile minimal (in-container) |
| ferro-rs version | 0.2.55 (resolved from `ferro = { package = "ferro-rs", version = "0.2" }`) |
| ferro-cli version | 0.2.55 (installed via `cargo install ferro-cli`) |
| Cache state | cold (clean `debian:bookworm-slim`, no pre-warmed toolchain or Cargo registry) |
| Host machine class | Apple silicon laptop (MacBookPro18,3) |
| CPU cores | 8 (Apple M1 Pro) |
| Memory | 16 GB |
| Disk free at run time | ~15 GB |
| Agent-assistance level | manual commands |
| Date | 2026-06-13 |

## Per-Step Wall-Clock Breakdown

Cold run via the committed Dockerfile. Steps 1–4 are CLI scaffolding (sub-second each;
the container's `SECONDS` counter has whole-second resolution, so each reports `0s`).
Step 5 (`cargo build`) **failed to compile** — see Discovered Weaknesses W1.

| Step | Command | Duration |
|------|---------|----------|
| 1 | `ferro new bench-app --no-interaction --no-git` | <1s (0s) |
| 2 | `ferro make:auth` | <1s (0s) |
| 3a | `ferro make:scaffold --no-smart-defaults -q -y --api Article title:string body:text` | <1s (0s) |
| 3b | `ferro make:scaffold --no-smart-defaults -q -y --api Product name:string price:float` | <1s (0s) |
| 3c | `ferro make:scaffold --no-smart-defaults -q -y --api Order status:string total:float` | <1s (0s) |
| 4 | `ferro make:job EmailNotification` | <1s (0s) |
| 5 | `cargo build` | FAILED — 52 compile errors in generated code (working app not reached) |
| **Total (scaffolding, steps 1–4)** | | **<4s** |
| **Total (to working app)** | | **not achieved — `cargo build` does not compile (W1)** |

The headline result: on the published toolchain, the four CLI scaffolding steps complete
in under a second each, but the generated project **does not build**. "Time to working app"
is therefore unbounded on `ferro-cli` / `ferro-rs` 0.2.55 — the dominant cost is not
compile time but a scaffold↔library API mismatch that prevents compilation entirely.

## Discovered Weaknesses

See `.planning/phases/211-comp-04-time-to-working-app-benchmark/211-WEAKNESSES.md` for the
full phase finding. Summary of what the cold run surfaced:

**W1 — Generated app does not compile against published `ferro-rs` 0.2.55 (dominant
finding).** `cargo build` of the scaffolded project fails with 52 errors. The scaffold
templates reference APIs that the published `ferro` crate does not export, and emit code
with missing imports and an undeclared dependency:
- `ferro::error_response!` macro — not found in `ferro` (every generated API controller).
- `#[rule]` attribute macro — not in scope (validation on the generated request structs).
- `ferro::Queue` / `ferro::QueueConfig` — unresolved imports.
- `use ferro_queue::…` in the `make:job` output, but `ferro-queue` is **not** a dependency
  in the generated `Cargo.toml` (confirmed: the generated `[dependencies]` has no
  `ferro-queue`/`ferro_queue` entry).
- `ActiveValue` — used as `ActiveValue::Set(...)` in scaffold controllers but never imported.
- `crate::models::users` — unresolved in the `make:auth` output.
- `ferro::database::connection` — used as a function but is a module.

**W2 — Cold CLI install requires `libssl-dev` + `pkg-config`.** A clean
`debian:bookworm-slim` cannot `cargo install ferro-cli`: `openssl-sys` (pulled via
`native-tls`) aborts with "Could not find directory of OpenSSL installation". The
Dockerfile now installs `libssl-dev pkg-config`; this is an undocumented first-time
prerequisite for installing the CLI on a minimal Debian base.

**W3 — `make:scaffold` flag ordering.** `[FIELDS]...` is a greedy trailing positional, so
flags placed after the fields (the natural `make:scaffold Name field:type --flags` order)
are parsed as field names: `Error parsing fields: Invalid field name: '--no-smart-defaults'`.
Flags must precede the fields (`make:scaffold [OPTIONS] <NAME> [FIELDS]...`).

**W4 — Spec/implementation naming mismatch.** ROADMAP SC#2 and CONTEXT.md specify
`ferro make:model <X>`, but no `make:model` subcommand exists; the benchmark uses
`ferro make:scaffold`. The benchmark satisfies SC#2's intent (three entity types) despite
the wording.

## Notes

- Cold-cache run: Docker container defined at
  `ferro-cli/tests/fixtures/benchmark/Dockerfile`. Build and run commands:
  ```bash
  docker build -t ferro-bench ferro-cli/tests/fixtures/benchmark/
  docker run --rm ferro-bench 2>&1 | tee cold-cache-run.txt
  ```
  Copy per-step durations from `cold-cache-run.txt` into the table above and commit.
  (`cold-cache-run.txt` itself is not committed.)

- Warm run: `FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored --nocapture`
  Run from the workspace root. Check `df -h` before starting; each run writes a full
  `target/` directory (~GBs) into a tmpdir. Note: the warm benchmark's `cargo build` step
  resolves `ferro-rs` from crates.io (not a local path dependency), so it reproduces W1 —
  the generated app does not compile against published 0.2.55 regardless of which `ferro`
  binary scaffolds it.

- CI wall-clock threshold: not asserted (D-07; deferred). The benchmark asserts only the
  `cargo build` exit code (SC#2) — which is itself the signal that surfaced W1.

- The `cargo install ferro-cli` step in the Dockerfile is cached in the Docker layer and is
  not part of the timed sequence. The benchmark measures scaffolding time from an
  already-installed `ferro` binary.

- The scaffolded project references `ferro-rs` from crates.io (not a local path dependency).
  The cold Docker `cargo build` step requires network access to resolve ferro-rs and its
  transitive dependencies. The Dockerfile does not use `--network none`.
