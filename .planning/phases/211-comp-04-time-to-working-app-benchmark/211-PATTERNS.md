# Phase 211: COMP-04 Time-to-Working-App Benchmark - Pattern Map

**Mapped:** 2026-06-13
**Files analyzed:** 4 (1 new test file, 1 Cargo.toml modification, 2 committed fixture files)
**Analogs found:** 3 / 4 (criterion has no in-repo analog; RESULTS.md/Dockerfile are novel committed artifacts)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-cli/tests/benchmark_new_project.rs` | integration-test / benchmark | batch (sequential CLI subprocess invocations) | `ferro-cli/tests/serve_supervisor.rs` + `ferro-mcp/tests/agent_harness.rs` L1222-1228 | role-match (subprocess + gate pattern) |
| `ferro-cli/Cargo.toml` `[dev-dependencies]` | config | — | `ferro-cli/Cargo.toml` L61-64 (existing `[dev-dependencies]` block) | exact (add one line to existing block) |
| `ferro-cli/tests/fixtures/benchmark/RESULTS.md` | committed result doc | — | `ferro-cli/tests/fixtures/gestiscilo/README.md` (committed markdown fixture) | partial (same fixture convention, different content) |
| `ferro-cli/tests/fixtures/benchmark/Dockerfile` | committed cold-cache script | — | `ferro-cli/tests/fixtures/gestiscilo/Dockerfile` (committed Dockerfile fixture) | partial (same fixture location, different purpose) |

---

## Pattern Assignments

### `ferro-cli/tests/benchmark_new_project.rs` (integration-test, batch)

**Analogs:**
- Gate pattern: `ferro-mcp/tests/agent_harness.rs` lines 1222–1228
- Binary invocation + `.status()` + `.current_dir()`: `ferro-cli/tests/serve_supervisor.rs` lines 29–31, 113–127
- `tempfile::tempdir()` + process-global isolation: `ferro-cli/tests/serve_supervisor.rs` lines 157–160, `ferro-cli/tests/docker_init_dry_run.rs` lines 49–53
- criterion programmatic use: no in-repo analog — use RESEARCH.md Pattern 3 (docs.rs/criterion/0.8.2)

**Gate pattern** (`ferro-mcp/tests/agent_harness.rs` lines 1222–1228):
```rust
#[tokio::test]
#[ignore = "live LLM eval; run with FERRO_AGENT_EVAL=1 and FERRO_AI_API_KEY set"]
async fn agent_eval_live_refresh_baseline() {
    if std::env::var("FERRO_AGENT_EVAL").is_err() {
        eprintln!("skipping: set FERRO_AGENT_EVAL=1 to run live eval");
        return;
    }
    // body ...
}
```
Copy as synchronous `#[test]` (no `async`), rename env var to `FERRO_BENCH`, rename function to `benchmark_new_project`, update `#[ignore]` message.

**Binary resolution + subprocess invocation** (`ferro-cli/tests/serve_supervisor.rs` lines 29–31, 113–127):
```rust
fn ferro_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ferro"))
}

// Per-command invocation shape (lines 118-126):
let mut child = Command::new(ferro_bin())
    .args(["serve", "--backend-only", "--skip-types"])
    .current_dir(fixture_dir())
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("spawn ferro serve");
```
For the benchmark, use `.status()` instead of `.spawn()` (no stdout streaming needed; we just want the exit code), and omit `Stdio::piped()` for stdin/stdout unless output is needed for debugging. Shape per step:
```rust
let t0 = Instant::now();
let status = Command::new(ferro_bin())
    .args(["make:scaffold", "Article", "--no-smart-defaults", "-q", "-y", "--api"])
    .current_dir(&project_dir)
    .status()
    .expect("ferro make:scaffold failed to spawn");
assert!(status.success(), "make:scaffold Article exited non-zero");
let step_3a = t0.elapsed();
```

**tempdir creation** (`ferro-cli/tests/serve_supervisor.rs` lines 157–160):
```rust
let tmp = tempfile::tempdir().expect("tempdir");
let pipe = tmp.path().join("trigger.pipe");
```
For the benchmark, `tmp.path()` is the CWD for `ferro new`; `tmp.path().join("bench-app")` is the CWD for all subsequent steps. Do NOT use `std::env::set_current_dir` (process-global; races with parallel tests). Use `.current_dir()` on each `Command`.

**Imports block** (assembled from `serve_supervisor.rs` lines 11–17 + required additions):
```rust
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use criterion::Criterion;
```

**criterion programmatic use** (no in-repo analog; from RESEARCH.md Pattern 3 / docs.rs/criterion/0.8.2):
```rust
let mut c = Criterion::default()
    .sample_size(3)                              // low: each sample builds a full project
    .measurement_time(Duration::from_secs(600)); // allow long wall-clock

c.bench_function("new_project_to_cargo_build", |b| {
    b.iter_custom(|iters| {
        let mut total = Duration::ZERO;
        for _ in 0..iters {
            let tmp = tempfile::tempdir().expect("tempdir");
            let project_dir = tmp.path().join("bench-app");

            // Step 1: ferro new
            let t = Instant::now();
            let s = Command::new(ferro_bin())
                .args(["new", "bench-app", "--no-interaction", "--no-git"])
                .current_dir(tmp.path())
                .status().expect("ferro new");
            assert!(s.success());
            let step1 = t.elapsed();

            // Steps 2-5 follow the same shape; step 5 asserts exit 0.
            total += step1 /* + step2 + step3a + step3b + step3c + step4 + step5 */;
        }
        total
    })
});
// Drive programmatically — no criterion_main!, no [[bench]] target.
// criterion flushes output when `c` drops; call final_summary() if available in 0.8.2.
```

**exit-code assertion pattern** (`ferro-cli/tests/serve_supervisor.rs` lines 142–151):
```rust
let clean = kill_and_wait(child, Duration::from_secs(5));
assert!(clean, "ferro serve did not exit within 5s of SIGINT");
```
For a `.status()` call (not a long-running spawn), the equivalent is:
```rust
let status = Command::new(ferro_bin())
    .args(["build"])
    .current_dir(&project_dir)
    .status()
    .expect("cargo build");
assert!(status.success(), "cargo build exited non-zero: {:?}", status.code());
```

**stdout timing table** (no analog; from RESEARCH.md Pattern 4):
```rust
println!("=== COMP-04 Benchmark Results ===");
println!("Step 1  ferro new:              {:?}", step1);
println!("Step 2  ferro make:auth:        {:?}", step2);
println!("Step 3a ferro make:scaffold A:  {:?}", step3a);
println!("Step 3b ferro make:scaffold P:  {:?}", step3b);
println!("Step 3c ferro make:scaffold O:  {:?}", step3c);
println!("Step 4  ferro make:job:         {:?}", step4);
println!("Step 5  cargo build:            {:?}", step5);
println!("Total:                          {:?}", total / iters as u32);
```
Visible under `--nocapture`. Copy numbers into RESULTS.md after each run.

---

### `ferro-cli/Cargo.toml` — `[dev-dependencies]` addition

**Analog:** `ferro-cli/Cargo.toml` lines 60–64 (existing `[dev-dependencies]` block):
```toml
[dev-dependencies]
tempfile = "3.24.0"

[target.'cfg(unix)'.dev-dependencies]
libc = "0.2"
```
Add one line inside the `[dev-dependencies]` block immediately after `tempfile`:
```toml
[dev-dependencies]
tempfile = "3.24.0"
criterion = { version = "0.8.2", default-features = false, features = ["cargo_bench_support"] }
```
Do NOT add a `[[bench]]` section. The benchmark lives in `tests/`, not `benches/`.

---

### `ferro-cli/tests/fixtures/benchmark/RESULTS.md` (committed result doc)

**Analog:** `ferro-cli/tests/fixtures/gestiscilo/README.md` (committed markdown fixture alongside a Dockerfile in the same fixtures subdirectory).

**Fixtures layout convention** (from `find ferro-cli/tests/fixtures`):
```
ferro-cli/tests/fixtures/
├── gestiscilo/          ← named subdirectory per fixture set
│   ├── Dockerfile
│   ├── README.md
│   ├── Cargo.toml
│   └── ...
├── minimal-serve/
│   └── ...
└── benchmark/           ← new subdirectory (same convention)
    ├── Dockerfile
    └── RESULTS.md
```

RESULTS.md content structure (from RESEARCH.md RESULTS.md Schema):
```markdown
# COMP-04 Time-to-Working-App Benchmark Results

## Environment

| Field | Value |
|-------|-------|
| Rust toolchain | stable YYYY-MM-DD (rustc X.Y.Z) |
| ferro-rs version | 0.2.X |
| Cache state | cold / warm |
| Host machine class | e.g. Apple M-series / GH runner |
| CPU cores | N |
| Memory | X GB |
| Disk free at run time | X GB |
| Agent-assistance level | manual commands |
| Date | YYYY-MM-DD |

## Per-Step Wall-Clock Breakdown

| Step | Command | Duration |
|------|---------|----------|
| 1 | `ferro new bench-app --no-interaction --no-git` | Xs |
| 2 | `ferro make:auth` | Xs |
| 3a | `ferro make:scaffold Article title:string body:text --no-smart-defaults -q -y --api` | Xs |
| 3b | `ferro make:scaffold Product name:string price:float stock:integer --no-smart-defaults -q -y --api` | Xs |
| 3c | `ferro make:scaffold Order status:string total:float --no-smart-defaults -q -y --api` | Xs |
| 4 | `ferro make:job EmailNotification` | Xs |
| 5 | `cargo build` | Xs |
| **Total** | | **Xs** |

## Discovered Weaknesses

- (non-empty required; phase close fails on empty section)

## Notes

- Cold-cache run: Docker (Dockerfile at `ferro-cli/tests/fixtures/benchmark/Dockerfile`).
- Warm run: `FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored --nocapture`.
- CI wall-clock threshold: not asserted (D-07; deferred to after first cold-cache run).
```

---

### `ferro-cli/tests/fixtures/benchmark/Dockerfile` (committed cold-cache script)

**Analog:** `ferro-cli/tests/fixtures/gestiscilo/Dockerfile` (committed Dockerfile in a fixtures subdirectory).

Content structure (from RESEARCH.md Docker Cold-Cache Strategy — ASSUMED base image):
```dockerfile
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates build-essential git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain via rustup — no pre-installed toolchain = cold.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal

ENV PATH="/root/.cargo/bin:${PATH}"

# Install ferro CLI (cached in Docker layer; not part of timed sequence).
RUN cargo install ferro-cli

WORKDIR /bench

CMD ["bash", "-c", "\
  echo '=== COMP-04 Cold-Cache Benchmark ===' && \
  SECONDS=0 && T0=$SECONDS && \
  ferro new bench-app --no-interaction --no-git && \
  echo \"Step 1 ferro new: $((SECONDS - T0))s\" && T0=$SECONDS && \
  cd bench-app && \
  ferro make:auth && \
  echo \"Step 2 make:auth: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:scaffold Article title:string body:text --no-smart-defaults -q -y --api && \
  echo \"Step 3a make:scaffold Article: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:scaffold Product name:string price:float stock:integer --no-smart-defaults -q -y --api && \
  echo \"Step 3b make:scaffold Product: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:scaffold Order status:string total:float --no-smart-defaults -q -y --api && \
  echo \"Step 3c make:scaffold Order: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:job EmailNotification && \
  echo \"Step 4 make:job: $((SECONDS - T0))s\" && T0=$SECONDS && \
  cargo build && \
  echo \"Step 5 cargo build: $((SECONDS - T0))s\" \
"]
```

Run commands (human-action, not autonomous):
```bash
docker build -t ferro-bench ferro-cli/tests/fixtures/benchmark/
docker run --rm ferro-bench 2>&1 | tee cold-cache-run.txt
# Copy per-step numbers from cold-cache-run.txt into RESULTS.md and commit.
```

---

## Shared Patterns

### Gate idiom (apply to `benchmark_new_project.rs`)

**Source:** `ferro-mcp/tests/agent_harness.rs` lines 1222–1228

The canonical belt-and-suspenders gate: `#[ignore]` prevents the test from running under `cargo test` unless `--ignored` is passed; the `env::var` early-return prevents accidental execution even when `--ignored` is used without the env var.

```rust
#[test]
#[ignore = "wall-clock benchmark; run with FERRO_BENCH=1 (builds a full project in tmpdir)"]
fn benchmark_new_project() {
    if std::env::var("FERRO_BENCH").is_err() {
        eprintln!("skipping: set FERRO_BENCH=1 to run benchmark");
        return;
    }
    // benchmark body
}
```

### `ferro_bin()` helper (apply to `benchmark_new_project.rs`)

**Source:** `ferro-cli/tests/serve_supervisor.rs` lines 29–31

```rust
fn ferro_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ferro"))
}
```

Copy verbatim. `CARGO_BIN_EXE_ferro` is set by cargo for all integration tests in crates that declare a `[[bin]]` target named `ferro` — confirmed in `ferro-cli/Cargo.toml` line 17–19.

### `tempfile::tempdir()` isolation (apply to `benchmark_new_project.rs`)

**Source:** `ferro-cli/tests/serve_supervisor.rs` line 159, `ferro-cli/tests/docker_init_dry_run.rs` line 52

```rust
let tmp = tempfile::tempdir().expect("tempdir");
```

`tempfile` is already in `ferro-cli [dev-dependencies]` at line 61 of `Cargo.toml`. No new dependency needed for this pattern.

### `.current_dir()` per-command (never `set_current_dir`)

**Source:** `ferro-cli/tests/serve_supervisor.rs` lines 119–125

```rust
Command::new(ferro_bin())
    .args([...])
    .current_dir(fixture_dir())  // ← per-command, not process-global
    .spawn()
    .expect("spawn ferro serve");
```

The benchmark has two distinct CWDs:
- `ferro new bench-app`: `.current_dir(tmp.path())` (parent)
- all subsequent steps: `.current_dir(tmp.path().join("bench-app"))` (project root)

Using `std::env::set_current_dir` is banned for tests that run in a shared process (races with parallel tests). `docker_init_dry_run.rs` uses `set_current_dir` + a `CHDIR_LOCK` mutex precisely to work around this limitation — `serve_supervisor.rs` avoids it entirely with per-command `.current_dir()`, which is the correct pattern for the benchmark.

### Fixtures directory convention

**Source:** `ferro-cli/tests/fixtures/` layout (read from `find`)

One named subdirectory per fixture set. The `benchmark/` subdirectory follows the same convention as `gestiscilo/` and `minimal-serve/`. Files within: committed as-is (no `.tpl` extension, no code generation). The `Dockerfile` and `RESULTS.md` are static committed artifacts, not generated.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| criterion `iter_custom` usage | benchmark harness | batch | No criterion usage exists anywhere in the workspace; criterion is not currently a dev-dependency of any crate. Use RESEARCH.md Pattern 3 (docs.rs/criterion/0.8.2 + RESEARCH.md lines 218-254) as the reference. |

---

## Metadata

**Analog search scope:** `ferro-cli/tests/`, `ferro-mcp/tests/`, `ferro-cli/Cargo.toml`
**Files scanned:** `serve_supervisor.rs`, `docker_init_dry_run.rs`, `gestiscilo_fixture.rs`, `agent_harness.rs` (lines 1215–1254), `ferro-cli/Cargo.toml`, fixtures directory layout
**Pattern extraction date:** 2026-06-13

**Key verified facts:**
- `env!("CARGO_BIN_EXE_ferro")` is the established pattern — `serve_supervisor.rs` line 30, confirmed by `ferro-cli/Cargo.toml` `[[bin]] name = "ferro"` at line 18.
- `tempfile = "3.24.0"` is already in `[dev-dependencies]` at `ferro-cli/Cargo.toml` line 61 — no duplicate needed.
- `[dev-dependencies]` block in `ferro-cli/Cargo.toml` starts at line 60 and currently has only one entry. criterion goes on line 62, after `tempfile`.
- `CHDIR_LOCK` pattern from `docker_init_dry_run.rs` is NOT needed for the benchmark — it uses `set_current_dir`; the benchmark uses per-command `.current_dir()` as in `serve_supervisor.rs`.
- The `fixtures/benchmark/` subdirectory does not yet exist; creating it follows the same naming convention as `fixtures/gestiscilo/` and `fixtures/minimal-serve/`.
