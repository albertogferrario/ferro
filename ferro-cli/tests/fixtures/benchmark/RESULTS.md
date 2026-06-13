# COMP-04 Time-to-Working-App Benchmark Results

## Environment

| Field | Value |
|-------|-------|
| Rust toolchain | TBD (filled by 211-02 cold run) |
| ferro-rs version | TBD (filled by 211-02 cold run) |
| Cache state | cold / warm (specify per run) |
| Host machine class | TBD (filled by 211-02 cold run) |
| CPU cores | TBD (filled by 211-02 cold run) |
| Memory | TBD (filled by 211-02 cold run) |
| Disk free at run time | TBD (filled by 211-02 cold run) |
| Agent-assistance level | manual commands |
| Date | TBD (filled by 211-02 cold run) |

## Per-Step Wall-Clock Breakdown

| Step | Command | Duration |
|------|---------|----------|
| 1 | `ferro new bench-app --no-interaction --no-git` | TBD |
| 2 | `ferro make:auth` | TBD |
| 3a | `ferro make:scaffold Article title:string body:text --no-smart-defaults -q -y --api` | TBD |
| 3b | `ferro make:scaffold Product name:string price:float --no-smart-defaults -q -y --api` | TBD |
| 3c | `ferro make:scaffold Order status:string total:float --no-smart-defaults -q -y --api` | TBD |
| 4 | `ferro make:job EmailNotification` | TBD |
| 5 | `cargo build` | TBD |
| **Total** | | **TBD** |

## Discovered Weaknesses

**Spec/implementation naming mismatch:** The ROADMAP success criteria (SC#2) and
CONTEXT.md specify `ferro make:model <X>` as the entity generation step, but no
`make:model` subcommand exists in the current codebase. The benchmark uses
`ferro make:scaffold <Name> --no-smart-defaults -q -y --api` (verified against the
`Commands` enum in `ferro-cli/src/main.rs`). The benchmark satisfies SC#2's intent
(three entity types with migration + controller per entity) despite the wording
discrepancy. The human-action plan 211-02 may surface additional weaknesses from the
real run — for example: `cargo build` dominating total time by an order of magnitude
over the CLI scaffolding steps, or a crates.io version-mismatch risk if the local
workspace is ahead of the published `ferro-rs` version.

## Notes

- Cold-cache run: Docker container defined at
  `ferro-cli/tests/fixtures/benchmark/Dockerfile`. Build and run commands:
  ```bash
  docker build -t ferro-bench ferro-cli/tests/fixtures/benchmark/
  docker run --rm ferro-bench 2>&1 | tee cold-cache-run.txt
  ```
  Copy per-step durations from `cold-cache-run.txt` into the table above and commit.

- Warm run: `FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored --nocapture`
  Run from the workspace root. Check `df -h` before starting; each run writes a full
  `target/` directory (~GBs) into a tmpdir.

- CI wall-clock threshold: not asserted (D-07; deferred to after first cold-cache run).
  The benchmark currently asserts only the `cargo build` exit code (SC#2).

- The `cargo install ferro-cli` step in the Dockerfile is cached in the Docker layer
  and is not part of the timed sequence. The benchmark measures project scaffolding
  time starting from an already-installed `ferro` binary.

- The scaffolded project references `ferro-rs` from crates.io (not a local path
  dependency). The cold Docker `cargo build` step requires network access to resolve
  ferro-rs and its transitive dependencies. This is expected behavior; the Dockerfile
  does not use `--network none`.
