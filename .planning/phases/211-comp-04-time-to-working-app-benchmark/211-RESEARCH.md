# Phase 211: COMP-04 — Time-to-Working-App Benchmark - Research

**Researched:** 2026-06-13
**Domain:** Rust CLI integration testing, criterion 0.8.2, Docker cold-cache benchmarking
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Hybrid gate: criterion `iter_custom` warm/local run behind `FERRO_BENCH=1`; cold-cache number from a separate Docker run. Default `cargo test`/CI runs neither.
- **D-02:** Cold-cache Docker run is a human-action step (`autonomous: false`). Autonomous executor builds + verifies everything else; human executes Docker and commits the number.
- **D-03:** Five steps: `ferro new <tmpdir>` → `ferro make:auth` → `ferro make:model <X>` × 3 → `ferro make:job <Y>` → `cargo build` in tmpdir; each step's wall-clock individually recorded; `cargo build` asserts exit 0. The exact `make:model` subcommand name is a RESEARCH item.
- **D-04:** criterion 0.8.2 added to `ferro-cli` `[dev-dependencies]` with `default-features = false, features = ["cargo_bench_support"]`.
- **D-05:** Committed Markdown result doc with Rust toolchain version, cache state, host machine class, agent-assistance level, per-step breakdown, total. Location: `ferro-cli/tests/fixtures/benchmark/RESULTS.md`.
- **D-06:** Agent-assistance level for committed baseline = manual commands.
- **D-07:** No CI wall-clock threshold asserted now. Benchmark asserts only the build step's exit code.

### Claude's Discretion

- Exact criterion `iter_custom` structure; the 3 entity names + 1 job name for the measured app; the Dockerfile base image tag; RESULTS.md table layout; how per-step timings are surfaced inside `iter_custom`.

### Deferred Ideas (OUT OF SCOPE)

- CI wall-clock threshold (set after first cold-cache run).
- Agent-driven scaffolding variant.
- Asserting the gate in CI.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-04 | Time-to-working-app benchmark: cold-cache run, committed apparatus doc, `FERRO_BENCH=1` gate. | All five research questions answered; concrete invocations, criterion structure, Docker strategy, and RESULTS.md schema documented below. |
</phase_requirements>

---

## Summary

Phase 211 builds a wall-clock benchmark at `ferro-cli/tests/benchmark_new_project.rs` that measures five CLI steps from a fresh `ferro new` scaffold to a compilable project with auth, three entity types, and one background job. The benchmark is gated behind `FERRO_BENCH=1` + `#[ignore]` so default CI never runs it. A separate committed Dockerfile drives the cold-cache number (human-action step).

The primary technical findings are: (1) there is no `make:model` subcommand — the correct command is `make:scaffold <Name> --no-smart-defaults -q -y --api`; (2) `ferro new` scaffolds a `ferro-rs` crates.io dependency, so `cargo build` in a cold Docker container needs network to crates.io, not the local workspace; (3) criterion `iter_custom` from a `tests/` file is viable but requires a `[[bench]]`-free approach — drive `Criterion::default()` programmatically with one `bench_function` call, use explicit `Instant` per step for the five-step breakdown, and return aggregate `Duration` to satisfy the `iter_custom` contract.

**Primary recommendation:** Use `make:scaffold <Name> --no-smart-defaults -q -y --api` for the three entity steps; drive the `ferro` binary via `env!("CARGO_BIN_EXE_ferro")` exactly as `serve_supervisor.rs` does; capture per-step wall-clock with `std::time::Instant` outside the criterion measurement layer; run criterion purely for the harness/statistics layer over the total five-step sequence.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CLI invocation (warm path) | Test binary (ferro-cli integration test) | ferro binary (the SUT) | Benchmark drives the ferro binary as a subprocess; `CARGO_BIN_EXE_ferro` resolves the built binary |
| Per-step timing | Test binary (Instant capture) | criterion (aggregate stats) | `iter_custom` returns one Duration; per-step breakdown requires explicit Instant wrapping inside the closure |
| Cold-cache run | Docker container (human-action) | — | No Docker daemon in autonomous executor; container must be clean (no toolchain, no Cargo cache) |
| Result document | Committed file (`RESULTS.md`) | — | Carried in `ferro-cli/tests/fixtures/benchmark/RESULTS.md` |
| Gate enforcement | `#[ignore]` + env-var early return | CI config (no change) | Belt-and-suspenders mirrors Phase 210 COMP-03 pattern |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| criterion | 0.8.2 [VERIFIED: cargo search] | Benchmark harness + statistics | Mandated by SC#1; only standard Rust microbench library with `iter_custom` + no external tooling requirement |
| tempfile | 3.24.0 | Isolated temp directory for scaffolded project | Already in `ferro-cli` `[dev-dependencies]`; established pattern in `docker_init_dry_run.rs` and Phase 210 harness |
| std::process::Command | stdlib | Invoke `ferro` binary subprocesses for each step | Established in `serve_supervisor.rs` |
| std::time::Instant | stdlib | Per-step wall-clock capture | Required: `iter_custom` returns one aggregate Duration; per-step breakdown needs explicit Instant inside the closure |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::env::var | stdlib | `FERRO_BENCH=1` gate check | Entry of the `#[test]` function |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `make:scaffold --api` for entity steps | `make:scaffold` (full-stack) | Full-stack scaffold generates Inertia pages; `--api` keeps it lean and avoids Node dependency during `cargo build`; use `--no-smart-defaults -q -y --api` |
| criterion `iter_custom` | `hyperfine` | `hyperfine` violates the no-external-build-tooling constraint (REQUIREMENTS.md); banned explicitly |
| `iter_custom` + explicit Instant | criterion `iter_batch` / `iter` | `iter` re-runs the same measurement many times; the 5-step scaffold is side-effect-heavy (writes files), cannot be meaningfully iterated inside `iter`; `iter_custom` with a fresh tempdir per iteration is correct |

**Installation (Cargo.toml addition):**
```toml
[dev-dependencies]
criterion = { version = "0.8.2", default-features = false, features = ["cargo_bench_support"] }
```

**No `[[bench]]` target is needed or wanted.** The benchmark lives in `ferro-cli/tests/benchmark_new_project.rs` as a `#[test]` function, not a `[[bench]]` target. The `cargo_bench_support` feature enables `Criterion::default()` to function outside the bench harness runner. Criterion is not invoked via `cargo bench` — it is driven programmatically from within the `#[test]` function.

---

## Architecture Patterns

### System Architecture Diagram

```
cargo test -p ferro-cli                     (no FERRO_BENCH=1 → test skips immediately)
cargo test -p ferro-cli -- --ignored FERRO_BENCH=1=1

  benchmark_new_project test fn
  │
  ├─ env::var("FERRO_BENCH") check ──→ skip if absent (early return)
  │
  ├─ tempfile::tempdir()  →  /tmp/ferro-bench-XXXX/
  │
  ├─ Step 1: ferro new bench-app --no-interaction --no-git
  │   CWD: parent of tmpdir (new.rs creates a subdir named "bench-app")
  │   Instant before/after → step_1_dur
  │
  ├─ Step 2: ferro make:auth
  │   CWD: bench-app dir (make_auth.rs checks src/controllers/, src/migrations/)
  │   Instant before/after → step_2_dur
  │
  ├─ Step 3a: ferro make:scaffold Article --no-smart-defaults -q -y --api
  │   CWD: bench-app dir
  │   Instant before/after → step_3a_dur
  │
  ├─ Step 3b: ferro make:scaffold Product --no-smart-defaults -q -y --api
  │   CWD: bench-app dir
  │   Instant before/after → step_3b_dur
  │
  ├─ Step 3c: ferro make:scaffold Order --no-smart-defaults -q -y --api
  │   CWD: bench-app dir
  │   Instant before/after → step_3c_dur
  │
  ├─ Step 4: ferro make:job EmailNotification
  │   CWD: bench-app dir
  │   Instant before/after → step_4_dur
  │
  ├─ Step 5: cargo build
  │   CWD: bench-app dir
  │   assert exit code 0 (SC#2)
  │   Instant before/after → step_5_dur
  │
  ├─ total = step_1..5 sum
  │
  └─ criterion b.iter_custom(|iters| { ... total * iters ... })
       (iters = 1 for a long-running scaffold; criterion records sample statistics)

Cold-cache path (human-action):
  docker build -t ferro-bench .   (Dockerfile at ferro-cli/tests/fixtures/benchmark/)
  docker run --rm ferro-bench     (prints per-step + total to stdout)
  → human copies numbers into RESULTS.md, commits
```

### Recommended Project Structure

```
ferro-cli/
├── Cargo.toml                          (add criterion dev-dep)
├── tests/
│   ├── benchmark_new_project.rs        (new file — the benchmark test)
│   └── fixtures/
│       └── benchmark/
│           ├── Dockerfile              (cold-cache Docker build, committed)
│           └── RESULTS.md             (committed result doc with env spec)
```

### Pattern 1: Gate Idiom (mirror of Phase 210 COMP-03)

**What:** Belt-and-suspenders gate that prevents the heavy benchmark from running in default `cargo test` or CI.

**When to use:** Any test that spawns `cargo build` or long-running processes.

```rust
// Source: ferro-mcp/tests/agent_harness.rs L1223-1226 (VERIFIED: read directly)
#[test]
#[ignore = "wall-clock benchmark; run with FERRO_BENCH=1 (builds a full project in tmpdir)"]
fn benchmark_new_project() {
    if std::env::var("FERRO_BENCH").is_err() {
        eprintln!("skipping: set FERRO_BENCH=1 to run benchmark");
        return;
    }
    // ... benchmark body
}
```

Run command:
```bash
FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored --nocapture
```

### Pattern 2: Binary Invocation (established pattern from serve_supervisor.rs)

**What:** Resolve the built `ferro` binary and spawn it as a subprocess.

```rust
// Source: ferro-cli/tests/serve_supervisor.rs L29-31 (VERIFIED: read directly)
fn ferro_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ferro"))
}
```

Each step follows the shape:
```rust
let t0 = Instant::now();
let status = Command::new(ferro_bin())
    .args(&["make:scaffold", "Article", "--no-smart-defaults", "-q", "-y", "--api"])
    .current_dir(&project_dir)
    .status()
    .expect("ferro make:scaffold failed to spawn");
assert!(status.success(), "make:scaffold Article exited non-zero");
let step_3a = t0.elapsed();
```

### Pattern 3: criterion iter_custom from a tests/ file

**What:** Drive criterion programmatically (no `[[bench]]` target, no `cargo bench`).

**Key insight:** `iter_custom` takes `FnMut(u64) -> Duration` where the closure runs `iters` iterations and returns total elapsed. For a one-shot multi-step scaffold (side effects make repetition meaningless), set `sample_size(10)` and run the full five-step sequence once per iteration in a fresh tempdir. The per-step breakdown uses explicit `Instant` captured inside the closure.

```rust
// Source: criterion docs (CITED: docs.rs/criterion/0.8.2/criterion/struct.Bencher.html)
// + pattern adapted from ferro-mcp/tests/agent_harness.rs structure
use criterion::Criterion;
use std::time::{Duration, Instant};

let mut c = Criterion::default()
    .sample_size(10)          // small: each sample builds a full project (~minutes warm)
    .measurement_time(Duration::from_secs(300));  // allow long wall-clock

c.bench_function("new_project_to_cargo_build", |b| {
    b.iter_custom(|iters| {
        let mut total = Duration::ZERO;
        for _ in 0..iters {
            let tmp = tempfile::tempdir().expect("tempdir");
            let project_dir = tmp.path().join("bench-app");
            // --- Step 1: ferro new ---
            let t = Instant::now();
            let s = Command::new(ferro_bin())
                .args(["new", "bench-app", "--no-interaction", "--no-git"])
                .current_dir(tmp.path())
                .status().unwrap();
            assert!(s.success());
            let step1 = t.elapsed();
            // ... steps 2-5 same shape ...
            // step 5: cargo build asserts exit 0
            total += step1 + step2 + step3a + step3b + step3c + step4 + step5;
            // drop(tmp) — tempdir cleaned on drop (careful: may add latency)
        }
        total
    })
});
// Must call final_summary() to flush output when not using criterion_main!
c.final_summary();
```

**Critical detail — CWD management:** `ferro new bench-app` must run with CWD = `tmp.path()` (parent), because `new.rs` creates a subdirectory named `bench-app`. All subsequent `make:*` commands must run with CWD = `tmp.path().join("bench-app")`. The benchmark does NOT use `std::env::set_current_dir` (that is process-global and races); it uses `.current_dir()` on each `Command`.

**Critical detail — make:scaffold flags:** `make:scaffold` with smart defaults active will prompt for confirmation on a non-TTY stdin. Use `--no-smart-defaults -q -y --api` to suppress all interactive prompts.

**Critical detail — make:auth CWD dependency:** `make_auth.rs` checks for `src/controllers/` and `src/migrations/` relative to CWD. These directories exist after `ferro new`. The benchmark must set `.current_dir(&project_dir)` on the `make:auth` invocation.

### Pattern 4: Per-Step Timing Surfaced in Output

Since criterion's output does not break down individual steps, print a timing table to stdout (visible under `--nocapture`) and write it into `RESULTS.md`:

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

### Anti-Patterns to Avoid

- **Using `std::env::set_current_dir` in the benchmark:** It is process-global. Use `.current_dir()` on each `Command` instead. If `set_current_dir` must be used, acquire `CWD_TEST_LOCK` from `ferro-cli::commands::CWD_TEST_LOCK` (already defined in `commands/mod.rs`).
- **Using `make:scaffold` without `--no-smart-defaults -q -y`:** Smart defaults detection prompts on non-TTY stdin; the benchmark subprocess will hang.
- **Using `make:model` as the subcommand:** This subcommand does not exist. The correct subcommand is `make:scaffold`.
- **Putting the benchmark in `[[bench]]` target:** No `[[bench]]` section should be added to `ferro-cli/Cargo.toml`. The benchmark lives in `tests/` as a gated `#[test]`.
- **Running `criterion_main!` macro in a `tests/` file:** `criterion_main!` generates a `main()` function for a `[[bench]]` binary. Instead, drive `Criterion::default()` directly and call `.final_summary()` at the end.
- **Calling `drop(tmp)` inside the iter_custom loop before measuring:** `tempfile::TempDir` drops (and deletes) on drop. Only drop after timing is recorded. Actually, better: let `tmp` drop at end of loop iteration scope.
- **Running criterion with default sample_size(100):** Each sample builds a full Rust project. Even warm, this takes 30–120 seconds per sample. Use `sample_size(10)` or lower.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Binary path resolution | Computing path to ferro executable | `env!("CARGO_BIN_EXE_ferro")` | cargo sets this automatically for integration tests of `[[bin]]` targets; established in `serve_supervisor.rs` |
| Temp directory lifecycle | Manual `fs::create_dir_all` + cleanup | `tempfile::tempdir()` | Already in dev-deps; cleanup on drop is correct behavior here |
| Benchmark statistics | Manual mean/stddev | `criterion` | criterion handles sampling, outlier detection, regression comparison; that is its purpose |
| Subprocess timeouts | `thread::sleep` + kill loop | `Command::status()` with OS-level timeout or just `status()` | For a benchmark (not a test), we want real wall-clock; no artificial timeout needed (SC only asserts exit 0) |

---

## Critical Finding: No `make:model` Subcommand

**The CONTEXT.md and ROADMAP both reference `ferro make:model <X>` as step 3.** This subcommand does not exist in the current codebase. [VERIFIED: read `ferro-cli/src/main.rs` — no `MakeModel` variant in `Commands` enum]

The correct command for generating an entity + migration is:

```bash
ferro make:scaffold <Name> --no-smart-defaults -q -y --api
```

This generates: migration file, model file, controller, and (with `--api`) JSON responses instead of Inertia pages.

**Implication for planning:** The PLAN must use `make:scaffold`, not `make:model`. The ROADMAP wording appears to be a shorthand that predates or misremembers the actual subcommand name. Recommend surfacing this in RESULTS.md's "discovered weaknesses" section (the benchmark name mismatch between spec and implementation is itself a finding).

**Alternative (lighter):** `make:migration <name>` + `make:module <name> --no-views` — but `make:scaffold --api` is closer to the stated intent (entity + migration + controller) and is a single invocation.

---

## Critical Finding: Scaffolded Project Uses crates.io Dependency

`ferro new` writes this `Cargo.toml` to the generated project: [VERIFIED: read `ferro-cli/src/templates/files/backend/Cargo.toml.tpl`]

```toml
[dependencies]
ferro = { package = "ferro-rs", version = "0.2" }
```

This is a **crates.io dependency**, not a path dependency to the local workspace. Implication:

1. **Warm/local `cargo build`:** Works only if the currently published `ferro-rs` version is compatible with the generated scaffold. If the local workspace is ahead of the published version, the generated scaffold may not compile with crates.io `ferro-rs 0.2`.
2. **Cold Docker run:** The container needs **network access to crates.io** during `cargo build`. A fully offline container will fail. The Dockerfile must NOT `--network none`.
3. **No local workspace mounting needed:** The container does not need the ferro workspace mounted. Standard `docker run` with network is sufficient.

The Cargo.toml template even documents this: `# Local ferro dev: append an uncommitted [patch.crates-io] block at the bottom of this file.`

**Risk for the warm benchmark:** If `ferro-rs 0.2.x` on crates.io is older than the locally generated scaffold templates, `cargo build` in the benchmark's tmpdir may fail. The plan should verify the published version matches the scaffold at benchmark run time, and document this as a discovered weakness.

---

## Common Pitfalls

### Pitfall 1: make:scaffold Hangs on Non-TTY Stdin

**What goes wrong:** `make:scaffold` detects smart defaults (e.g., existing factories in CWD) and prompts "Proceed with generation?" on a non-interactive stdin. The subprocess blocks indefinitely.

**Why it happens:** `make_scaffold.rs` calls `dialoguer::Confirm::interact()` when `smart_defaults.has_any()` and `!yes`.

**How to avoid:** Pass `--no-smart-defaults -q -y` (or `--yes -q --no-smart-defaults`). The `-y` flag maps to the `yes` parameter; `--no-smart-defaults` disables the smart defaults scan entirely; `-q` suppresses the summary output.

**Warning signs:** Benchmark subprocess never exits; `status()` blocks. Add a timeout in the subprocess invocation during development.

### Pitfall 2: ferro new Creates Subdirectory, Not In-Place

**What goes wrong:** Running `ferro new bench-app` with `.current_dir(some_tmp_path)` creates `some_tmp_path/bench-app/`. A subsequent `make:auth` with `.current_dir(some_tmp_path)` fails because `src/controllers/` does not exist there.

**How to avoid:** Use `tmp.path()` as CWD for the `new` step; use `tmp.path().join("bench-app")` as CWD for all subsequent steps.

**Evidence:** `new.rs` line 147 — `let project_path = Path::new(project_name)` and creates directories under it.

### Pitfall 3: Disk Exhaustion During Benchmark Run

**What goes wrong:** A single `cargo build` of a ferro project downloads and compiles ~200+ crates into a `target/` directory under the tmpdir. With ~12 GB free on this host, one run is fine; multiple criterion iterations may overflow.

**How to avoid:** Set `sample_size(3)` or even `sample_size(1)` for initial runs. Delete `target/` from the tmpdir between iterations (or use `CARGO_TARGET_DIR` pointing to a shared location that is cleaned between runs). Document disk requirements in RESULTS.md.

**Recovery:** Run `df -h` before invoking `FERRO_BENCH=1`. Clean `target/` under the bench tmpdir after each run.

### Pitfall 4: `criterion_main!` Cannot Be Used in tests/

**What goes wrong:** Adding `criterion_main!(benches)` in a `tests/` file causes a compilation error (duplicate `main` or unreachable macro expansion, since test files are compiled with their own harness that provides `main`).

**How to avoid:** Use `Criterion::default()` directly, call `.bench_function(...)` on it, then call `.final_summary()`. No macros.

### Pitfall 5: Warm Build Fails Due to crates.io Version Mismatch

**What goes wrong:** The generated project references `ferro-rs 0.2` on crates.io. If the local workspace is at `0.2.54` but the last published version is `0.2.55`, the scaffold templates may reference symbols or proc-macros not yet on crates.io.

**How to avoid:** Check `cargo search ferro-rs` before running the warm benchmark. If mismatch, publish first or apply a `[patch.crates-io]` block to the generated project before step 5.

### Pitfall 6: Docker `--network none` Blocks cargo build

**What goes wrong:** A "cold-cache" Docker container with `--network none` cannot reach crates.io. `cargo build` fails with network errors.

**How to avoid:** The Dockerfile does NOT use `--network none`. "Cold cache" means no Cargo registry cache pre-warmed, no toolchain pre-installed — not network isolation. Allow default network during `docker run`.

---

## Exact CLI Invocations (Verified)

All subcommands verified against `ferro-cli/src/main.rs` clap `Commands` enum. [VERIFIED: read directly]

| Step | Subcommand | Flags Required |
|------|-----------|---------------|
| 1 | `ferro new bench-app` | `--no-interaction --no-git` | Creates `bench-app/` in CWD; `--no-interaction` skips dialoguer prompts; `--no-git` skips `git init` (saves ~50ms) |
| 2 | `ferro make:auth` | none (or `--force` if re-running) | Requires CWD = project root; checks `src/controllers/` + `src/migrations/` |
| 3a | `ferro make:scaffold Article` | `--no-smart-defaults -q -y --api` | Entity #1; `--api` skips Inertia page generation |
| 3b | `ferro make:scaffold Product` | `--no-smart-defaults -q -y --api` | Entity #2 |
| 3c | `ferro make:scaffold Order` | `--no-smart-defaults -q -y --api` | Entity #3 |
| 4 | `ferro make:job EmailNotification` | none | Creates `src/jobs/email_notification_job.rs`; appends "Job" suffix automatically |
| 5 | `cargo build` | none (or `--release` optional) | CWD = project root; asserts exit 0 |

**make:scaffold field arguments (optional but recommended for a richer benchmark):**

```bash
ferro make:scaffold Article title:string body:text published:bool --no-smart-defaults -q -y --api
ferro make:scaffold Product name:string price:float stock:integer --no-smart-defaults -q -y --api
ferro make:scaffold Order status:string total:float --no-smart-defaults -q -y --api
```

These fields produce a realistic migration and model, making the benchmark more representative.

---

## Docker Cold-Cache Strategy

### Recommended Dockerfile

Location: `ferro-cli/tests/fixtures/benchmark/Dockerfile` [ASSUMED — base image choice is Claude's discretion per D-02/D-05]

```dockerfile
FROM debian:bookworm-slim

# Install dependencies for rustup and cargo
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates build-essential git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain via rustup (no pre-installed toolchain = cold)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal

ENV PATH="/root/.cargo/bin:${PATH}"

# No cargo registry cache pre-warmed (cold = fresh ~/.cargo on first cargo build)

# Install ferro CLI from crates.io
RUN cargo install ferro-cli

WORKDIR /bench

# Run the five-step sequence, printing timing for each step
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
  ferro make:scaffold Product name:string price:float --no-smart-defaults -q -y --api && \
  echo \"Step 3b make:scaffold Product: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:scaffold Order status:string total:float --no-smart-defaults -q -y --api && \
  echo \"Step 3c make:scaffold Order: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:job EmailNotification && \
  echo \"Step 4 make:job: $((SECONDS - T0))s\" && T0=$SECONDS && \
  cargo build && \
  echo \"Step 5 cargo build: $((SECONDS - T0))s\" \
"]
```

**Why `debian:bookworm-slim` + rustup, not `rust:slim`:** The `rust:slim` base image ships with a pre-installed Rust toolchain and a pre-warmed Cargo registry index — this is a warm cache, not a cold one. `debian:bookworm-slim` + rustup-install is truly cold: no toolchain, no registry index, no compiled deps. [ASSUMED — Docker image selection is Claude's discretion; the key requirement is "no pre-installed Rust toolchain and no Cargo cache" per SC#3]

**Run command:**
```bash
docker build -t ferro-bench ferro-cli/tests/fixtures/benchmark/
docker run --rm ferro-bench 2>&1 | tee cold-cache-run.txt
# Human copies per-step numbers from cold-cache-run.txt into RESULTS.md and commits
```

**Note on `cargo install ferro-cli`:** This installs the `ferro` CLI binary from crates.io. This itself is a warm step (it downloads and compiles ferro-cli). Two options: (a) install ferro-cli in the `RUN` layer (cached in Docker layer cache after first build) and time only the five benchmark steps; (b) include `cargo install ferro-cli` in the timed sequence. Option (a) is recommended: the benchmark measures project scaffolding time, not CLI installation time. The RESULTS.md should document which approach was used.

---

## RESULTS.md Schema

Location: `ferro-cli/tests/fixtures/benchmark/RESULTS.md` [VERIFIED: D-05]

```markdown
# COMP-04 Time-to-Working-App Benchmark Results

## Environment

| Field | Value |
|-------|-------|
| Rust toolchain | stable YYYY-MM-DD (rustc X.Y.Z) |
| ferro-rs version | 0.2.X |
| Cache state | cold / warm |
| Host machine class | e.g. Apple M-series / GH runner / EC2 type |
| CPU cores | N |
| Memory | X GB |
| Disk free at run time | X GB |
| Agent-assistance level | manual commands (D-06) |
| Date | YYYY-MM-DD |

## Per-Step Wall-Clock Breakdown

| Step | Command | Duration |
|------|---------|----------|
| 1 | `ferro new bench-app --no-interaction --no-git` | Xs |
| 2 | `ferro make:auth` | Xs |
| 3a | `ferro make:scaffold Article ...` | Xs |
| 3b | `ferro make:scaffold Product ...` | Xs |
| 3c | `ferro make:scaffold Order ...` | Xs |
| 4 | `ferro make:job EmailNotification` | Xs |
| 5 | `cargo build` | Xs |
| **Total** | | **Xs** |

## Discovered Weaknesses

- (required — empty section fails phase close per SC#5)
- Suggested candidates: `make:scaffold` does not match the `make:model` name in ROADMAP/CONTEXT (naming mismatch); `cargo build` dominates total time by an order of magnitude over CLI steps; no unhappy-path measurements (scaffold into existing directory, duplicate entity name, missing auth before scaffold); no measurement of `ferro db:migrate` after scaffolding.

## Notes

- Cold-cache run was executed in Docker (Dockerfile at `ferro-cli/tests/fixtures/benchmark/Dockerfile`).
- Warm run via `FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored --nocapture`.
- CI wall-clock threshold: not asserted (D-07; deferred to after first cold-cache run).
```

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `debian:bookworm-slim` + rustup is the correct cold-cache base image (no toolchain, no Cargo cache) | Docker Cold-Cache Strategy | `rust:slim` would be warm, not cold; the headline number would be misleading |
| A2 | `make:scaffold --api` with fields is more representative than a no-field scaffold | Exact CLI Invocations | Without fields, the generated migration and model are trivial; benchmark may undercount real-world template generation time |
| A3 | `cargo install ferro-cli` is run in the Dockerfile `RUN` layer (warm), not in the timed CMD | Docker Cold-Cache Strategy | If included in timing, Step 0 (CLI install) would dominate and the number would not be comparable to local warm runs |
| A4 | `sample_size(10)` is low enough to not overflow disk in warm mode | Architecture Patterns | If 10 full `cargo build` runs fill the 12 GB free, the benchmark aborts; may need `sample_size(3)` or even `sample_size(1)` |

---

## Open Questions

1. **make:scaffold vs make:model naming**
   - What we know: ROADMAP and CONTEXT say `ferro make:model <X>` but no such subcommand exists; `make:scaffold --api` is the closest match.
   - What's unclear: Whether the planner should use `make:scaffold` exactly as-is or whether a thin `make:model` alias should be added to the CLI.
   - Recommendation: Use `make:scaffold --api` in the benchmark as-is. Document the naming discrepancy in "discovered weaknesses" — it is the kind of real finding COMP-04 should surface.

2. **Published ferro-rs version vs local workspace version**
   - What we know: The scaffolded project pulls `ferro-rs 0.2` from crates.io. The workspace is at `0.2.54`.
   - What's unclear: Whether `0.2.54` (or the next publish) is the current crates.io HEAD. If the bench runs before the next publish, the scaffold may pull a slightly older version; `cargo build` may still succeed if no breaking changes.
   - Recommendation: Document the exact published version in RESULTS.md. Run `cargo search ferro-rs` before the warm benchmark and add a note if version mismatch is detected.

3. **criterion `final_summary()` availability**
   - What we know: Criterion has a `final_summary()` method for programmatic use.
   - What's unclear: Whether `final_summary()` is part of the public API in 0.8.2 or was added later.
   - Recommendation: If `final_summary()` is not available, the benchmark can simply end after the `bench_function` call; criterion will flush output when `c` drops. Verify during Wave 0 compilation.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo (Rust toolchain) | Step 5 `cargo build` | ✓ | host stable | — |
| `CARGO_BIN_EXE_ferro` | Binary invocation | ✓ | set by cargo during test | — |
| tempfile crate | tmpdir creation | ✓ | 3.24.0 (already in dev-deps) | — |
| Docker daemon | Cold-cache run (D-02) | not checked (human-action) | — | None needed; D-02 is human-action |
| network to crates.io | `cargo build` in cold Docker | required | — | `[patch.crates-io]` block in generated Cargo.toml |

**Missing dependencies with no fallback:**
- None blocking autonomous work. Docker is human-action (D-02).

**Missing dependencies with fallback:**
- crates.io network (Docker run): fallback is mounting local workspace + `[patch.crates-io]`, but this changes the cold-cache definition. Document in RESULTS.md which approach was used.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) + criterion 0.8.2 |
| Config file | None — criterion driven programmatically, not via `benches/` target |
| Quick run command | `cargo test -p ferro-cli --test benchmark_new_project` (skips due to gate) |
| Full/gated run command | `FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored --nocapture` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-04 SC#1 | Benchmark file compiles; criterion dep added; gate enforced | compilation + unit | `cargo test -p ferro-cli` (verifies gate: file compiles, test is skipped) | ❌ Wave 0 |
| COMP-04 SC#2 | `cargo build` in scaffolded project exits 0 | integration (gated) | `FERRO_BENCH=1 cargo test -p ferro-cli --test benchmark_new_project -- --ignored` | ❌ Wave 0 |
| COMP-04 SC#3 | Cold-cache Docker number committed | manual (D-02) | human action; grep-check `RESULTS.md` for "cold" label | ❌ Wave 0 |
| COMP-04 SC#4 | RESULTS.md has all required env-spec fields | file content check | `grep -E "Rust toolchain|Cache state|Host machine|Agent-assistance|per-step" ferro-cli/tests/fixtures/benchmark/RESULTS.md` | ❌ Wave 0 |
| COMP-04 SC#5 | "Discovered weaknesses" section is non-empty | file content check | `grep -A3 "Discovered Weaknesses" RESULTS.md \| grep -v "^--" \| wc -l` (> 1) | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-cli` — verifies compilation gate works (benchmark is compiled but skipped)
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-cli`
- **Phase gate:** Full suite green + `FERRO_BENCH=1` warm run produces output + RESULTS.md committed with all env-spec fields

### Wave 0 Gaps

- [ ] `ferro-cli/tests/benchmark_new_project.rs` — covers COMP-04 SC#1 + SC#2
- [ ] `ferro-cli/tests/fixtures/benchmark/RESULTS.md` — template with env-spec fields, to be filled by human after cold run (SC#3 + SC#4 + SC#5)
- [ ] `ferro-cli/tests/fixtures/benchmark/Dockerfile` — committed cold-cache Docker build definition (D-02)
- [ ] `ferro-cli/Cargo.toml` — add `criterion = { version = "0.8.2", default-features = false, features = ["cargo_bench_support"] }` to `[dev-dependencies]`

---

## Security Domain

> This phase adds only a test file and a Dockerfile. No new network endpoints, no authentication surfaces, no user input handling, no cryptography. ASVS categories V2-V6 do not apply. The Dockerfile does `rustup install` from the official rustup endpoint — verify the TLS pinning is standard (`--proto '=https' --tlsv1.2` in the curl command).

---

## Sources

### Primary (HIGH confidence)
- `ferro-cli/src/main.rs` — verified exact `Commands` enum; confirmed `make:scaffold`, `make:auth`, `make:job` subcommand names; confirmed no `make:model` exists [VERIFIED: read directly]
- `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` — confirmed scaffolded project uses `ferro-rs` crates.io dep, not path dep [VERIFIED: read directly]
- `ferro-cli/src/commands/new.rs` — confirmed `ferro new <name>` creates subdirectory named `<name>` in CWD [VERIFIED: read directly]
- `ferro-cli/src/commands/make_scaffold.rs` — confirmed `--no-smart-defaults -q -y --api` flags suppress interactive prompts [VERIFIED: read directly]
- `ferro-cli/src/commands/make_auth.rs` — confirmed CWD-relative path checks for `src/controllers/` [VERIFIED: read directly]
- `ferro-cli/src/commands/make_job.rs` — confirmed CWD-relative path check for `src/` [VERIFIED: read directly]
- `ferro-cli/tests/serve_supervisor.rs` L29-31 — `env!("CARGO_BIN_EXE_ferro")` pattern [VERIFIED: read directly]
- `ferro-mcp/tests/agent_harness.rs` L1223-1226 — `FERRO_AGENT_EVAL=1` + `#[ignore]` gate pattern to mirror [VERIFIED: read directly]
- `ferro-cli/Cargo.toml` — confirmed `tempfile = "3.24.0"` already in dev-deps [VERIFIED: read directly]
- `/bheisler/criterion.rs` Context7 — `iter_custom` signature `FnMut(u64) -> Duration`; `Criterion::default()` constructor [CITED: context7.com/bheisler/criterion.rs]

### Secondary (MEDIUM confidence)
- `cargo search criterion` output — confirmed criterion 0.8.2 is current [VERIFIED: ran command]
- docs.rs/criterion/0.8.2 — `iter_custom` signature and `Criterion::default()` configuration [CITED: docs.rs/criterion/0.8.2/criterion/struct.Bencher.html]
- github.com/bheisler/criterion.rs Cargo.toml — `cargo_bench_support` feature description [CITED: GitHub]

### Tertiary (LOW confidence)
- Dockerfile base image recommendation (`debian:bookworm-slim`) — [ASSUMED] based on "cold cache = no pre-installed toolchain" requirement

---

## Metadata

**Confidence breakdown:**
- CLI subcommands: HIGH — read directly from source
- criterion API: HIGH — verified via Context7 + docs.rs
- Docker cold-cache strategy: MEDIUM — requirement is clear; specific Dockerfile is ASSUMED
- make:model absence: HIGH — verified no such variant in clap Commands enum

**Research date:** 2026-06-13
**Valid until:** 2026-07-13 (stable — criterion API is stable; CLI commands unlikely to change in this window)

---

## RESEARCH COMPLETE

**Phase:** 211 - COMP-04 Time-to-Working-App Benchmark
**Confidence:** HIGH

### Key Findings

- **`make:model` does not exist.** The correct entity generator is `make:scaffold <Name> --no-smart-defaults -q -y --api`. This is the most important correction to the ROADMAP wording.
- **All four CLI steps are verified.** `ferro new`, `make:auth`, `make:scaffold`, `make:job` subcommands confirmed in `main.rs`. Exact flags for non-interactive execution documented.
- **`CARGO_BIN_EXE_ferro` + `.current_dir()` on each Command** is the established and correct pattern (from `serve_supervisor.rs`). Do NOT use `std::env::set_current_dir`.
- **criterion `iter_custom` from a `tests/` file works without `[[bench]]`** — drive `Criterion::default()` programmatically with `sample_size(3..10)` (low, since each sample builds a full project); per-step breakdown via explicit `Instant` inside the closure.
- **Scaffolded project uses crates.io `ferro-rs`**, not a local path dep. Cold Docker run needs network. This must be documented in RESULTS.md.

### File Created

`.planning/phases/211-comp-04-time-to-working-app-benchmark/211-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| CLI subcommands + flags | HIGH | Read directly from source; no inference |
| criterion iter_custom API | HIGH | Context7 + docs.rs cross-verified |
| Gate pattern | HIGH | Phase 210 `agent_harness.rs` read directly |
| Docker cold-cache strategy | MEDIUM | Requirement clear; specific Dockerfile is ASSUMED |
| make:model absence | HIGH | No `MakeModel` variant in clap Commands enum |

### Open Questions

- Whether `criterion::Criterion::final_summary()` is public API in 0.8.2 (verify during Wave 0 compilation).
- Whether the published `ferro-rs` version on crates.io matches the local workspace scaffold templates (check before running warm benchmark).

### Ready for Planning

Research complete. Planner can now create PLAN.md files.
