# Stack Research — v13.0 Compressive Validation

**Project:** ferro / v13.0 Compressive Validation (COMP-02, COMP-03, COMP-04)
**Researched:** 2026-06-12
**Scope:** Additive — what the three validation harnesses need beyond the existing workspace.
**Confidence:** HIGH (insta 1.48.0 and criterion 0.8.2 verified via docs.rs; rmcp transport verified via docs.rs 0.12.0 features page; proptest 1.11.0 verified via docs.rs; existing workspace patterns confirmed by direct source read)

---

## Summary

Three validation harnesses are being built:

| COMP | Goal | Key tooling need |
|------|------|-----------------|
| COMP-02 | Synthetic catalog of canonical app classes covering all 7 intents, regression-tested on every projection/intent change | Snapshot/golden testing for `Vec<IntentScore>` and rendered `Spec` output |
| COMP-03 | Agent-success-rate measurement — can an agent reading ferro-mcp produce a working projection from a natural-language description? | In-process MCP client that calls tools/call against a live FerroMcpService |
| COMP-04 | Time-to-working-app benchmark — `cargo new` → running service with auth + 3 entities + 1 job | Wall-clock timing with statistical confidence; multi-step setup/teardown |

The workspace already has the infrastructure for most of this. The additions are minimal: one new dev-dependency for snapshot testing (insta), one for benchmarking (criterion), and one for property testing already present (proptest). MCP client testing uses existing rmcp `transport-async-rw` (a default rmcp 0.12 feature) via `tokio::io::duplex` — no new crate required.

---

## Recommended Stack

### COMP-02: Synthetic Catalog Regression Tests

#### Core: `insta` — Snapshot / Golden Testing

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `insta` | `1.48` | Snapshot-test `Vec<IntentScore>` output and rendered `Spec` JSON for each canonical app class | The `assert_json_snapshot!` macro serializes the full `IntentScore` vector (including `confidence`, `matching_signals`) to a committed `.snap` file. On every projection/intent change, the diff surfaces immediately; `cargo insta review` presents an interactive accept/reject UI. No external build tooling — pure Rust dev-dependency, zero runtime impact. |

**Recommended Cargo.toml entry** (in `ferro-projections/Cargo.toml` dev-dependencies):

```toml
[dev-dependencies]
insta = { version = "1.48", features = ["json", "redactions"] }

# Optimize insta for faster test runs (insta's own recommendation)
[profile.dev.package.insta]
opt-level = 3
[profile.dev.package.similar]
opt-level = 3
```

`features = ["json"]` enables `assert_json_snapshot!` for `IntentScore` (already `Serialize`).
`features = ["redactions"]` allows redacting `confidence` floats that may drift slightly across
analyzer weight changes, keeping snapshots stable while still asserting intent ordering.

**Why not hand-written `assert_eq!` + expected structs**: The seven-intent catalog will have
30–40 canonical `ServiceDef` fixtures. Maintaining expected `Vec<IntentScore>` literals inline
is brittle on any weight recalibration. Snapshot files are committed, diff on change, and updated
in one `cargo insta test --accept` pass — they encode intent without the maintenance burden.

**Why not `assert_yaml_snapshot!`**: JSON is already the `ServiceDef` wire format and `serde_json`
is already a direct dependency of `ferro-projections`. No extra `serde_yaml` dependency needed.

#### Fixture Organization

Canonical app-class fixtures live inside `ferro-projections` as a new test module, not a separate
crate. The fixtures are pure `ServiceDef` builder expressions — no database, no HTTP, no runtime.
Each fixture maps to one canonical app class and is structured as:

```
ferro-projections/tests/
  catalog/
    mod.rs           -- loads all fixtures, runs derive_intents + snapshot
    ecommerce.rs     -- Browse (product list), Focus (product detail), Collect (checkout)
    saas.rs          -- Summarize (dashboard), Process (subscription workflow)
    crm.rs           -- Browse (contact list), Track (activity log)
    scheduling.rs    -- Process (booking state machine), Browse (calendar view)
    content.rs       -- Focus (article), Analyze (analytics dashboard)
    finance.rs        -- Summarize (balance sheet), Analyze (time-series)
    logistics.rs     -- Process (shipment workflow), Track (status trail)
```

Each file contains one or more `ServiceDef` builder expressions and one `#[test]` per fixture
that calls `derive_intents()` + `assert_json_snapshot!`. A single top-level regression test
(`catalog/mod.rs`) iterates all fixtures and asserts: (1) top intent matches the expected intent
for this app class; (2) confidence >= 0.5; (3) snapshot matches.

Snapshot files are stored at `ferro-projections/tests/snapshots/` (insta default) and committed.
They appear in PRs as diffs when any signal weight or analyzer changes.

**Where the `Spec` render snapshot goes**: The `ferro-json-ui` crate already has `Spec::from_service_def()`.
A second test layer in `ferro-json-ui/tests/catalog_render.rs` uses the same fixtures (imported
from `ferro-projections`) to snapshot the rendered `Spec` JSON. This asserts that a `Browse`
catalog fixture still produces a `DataTable` root, a `Collect` fixture produces a `Form` root,
etc., on every commit.

```toml
# ferro-json-ui/Cargo.toml dev-dependencies
insta = { version = "1.48", features = ["json"] }
```

---

### COMP-03: Agent-Success-Rate Measurement Harness

#### Core: `rmcp` with `transport-async-rw` feature — In-Process MCP Client

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `rmcp` | `0.12` (already in workspace) | Drive `FerroMcpService` in-process over a `tokio::io::duplex` pipe | `transport-async-rw` is a **default feature of rmcp 0.12** — it is already compiled into every crate that depends on `ferro-mcp`. No new crate, no version bump. The `tokio::io::duplex()` function creates an in-memory bidirectional pipe; the server runs in a tokio task; the test calls `peer().call_tool(...)` exactly as a real MCP client would. |

**Pattern already proven in the workspace**: `ferro-api-mcp/tests/e2e.rs` uses
`rmcp = { version = "0.12", features = ["client", "transport-child-process"] }` in dev-dependencies
and calls `mcp.peer().call_tool(CallToolRequestParam { ... })` for end-to-end validation.
COMP-03 adapts this to use an in-process transport (no subprocess overhead, no port conflicts).

**Recommended Cargo.toml entry** (in a new `ferro-projections-harness` test binary or in
`ferro-mcp/Cargo.toml` dev-dependencies):

```toml
[dev-dependencies]
rmcp = { version = "0.12", features = ["client", "transport-async-rw"] }
tokio = { version = "1", features = ["full"] }
```

`transport-async-rw` is already a default feature on the server side; the `client` feature adds
`serve_client()` and the `RoleClient` API needed on the test side.

**COMP-03 test structure** (`ferro-mcp/tests/agent_success_rate.rs`):

Each test case encodes a natural-language scenario string (the "description" an agent would pass
to `generate_projection`) plus an expected `ServiceDef` shape. The test:

1. Boots `FerroMcpService` in-process via `tokio::io::duplex` transport.
2. Calls the `generate_projection` MCP tool with the scenario string as input.
3. Parses the returned JSON back into a `ServiceDef`.
4. Asserts: (a) parse succeeds; (b) primary intent matches the scenario's expected intent;
   (c) minimum required fields are present; (d) no validation errors from `validate_projection`.

Success rate is reported as `N_passed / N_total` over the test corpus. Tests are `#[ignore]`
by default and run explicitly as `cargo test -p ferro-mcp --test agent_success_rate -- --include-ignored`
so they do not block the standard CI gate (they may require a running ferro application with
`DATABASE_URL` set).

**What "success" means**: A projection is successful when it round-trips `ServiceDef` from
the tool output and passes `validate_projection`. This tests the MCP tool pipeline, not the LLM.
The natural-language → `ServiceDef` AI path (from v12.1 `ferro ai:make`) is a separate concern
and depends on an LLM API key; COMP-03 focuses on whether the MCP introspection layer provides
sufficient context for a projection to be generated correctly.

**Alternative considered — `transport-child-process`**: The existing e2e tests in `ferro-api-mcp`
spawn a subprocess binary. For COMP-03, in-process is preferred because: (1) it does not require
a compiled binary artifact to be staged before the test; (2) `FerroMcpService` can be instantiated
directly with a `PathBuf` pointing to the test fixture directory; (3) no port allocation or
timing loops needed.

---

### COMP-04: Time-to-Working-App Benchmark

#### Core: `criterion` — Wall-Clock Timing with Statistical Confidence

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `criterion` | `0.8.2` | Measure multi-step scaffold workflow: `cargo new` → `ferro-cli` scaffold → build → server start | Criterion provides warm-up runs, statistical noise thresholds, HTML reports, and baseline comparison (`cargo bench -- --save-baseline before` / `--load-baseline before`). Stable-only (no nightly required). `iter_custom` supports externally-timed operations where Criterion does not control the loop — required here because each benchmark step is a subprocess invocation. |

**Recommended Cargo.toml entry** (new `benches/` in `ferro-cli` or a dedicated `ferro-validation` crate):

```toml
[dev-dependencies]
criterion = { version = "0.8", default-features = false, features = ["rayon", "cargo_bench_support"] }

[[bench]]
name = "time_to_working_app"
harness = false
```

`default-features = false` excludes the `plotters` feature which pulls in a large dependency
tree. `rayon` is excluded by default in 0.8 — add only if parallelizing benchmark setup steps.
`cargo_bench_support` is required for `criterion_group!` / `criterion_main!` macros.

**What COMP-04 actually measures**: The benchmark does not run the full `cargo build` on every
iteration (that would be minutes per sample). Instead it measures the scaffold layer — the
time from `cargo new <name>` through `ferro new` template application through the point where
a buildable `Cargo.toml` exists with the correct dependency graph. The _first_ full build is
measured once separately as a warm-start artifact, recorded as a named annotation in the
criterion baseline file.

Full workflow breakdown:

| Step | Measured how | Notes |
|------|-------------|-------|
| `cargo new ferro-test-app` | Subprocess in `iter_custom` wall-clock | Fast (< 100 ms) |
| `ferro` CLI scaffold (auth + 3 entities + 1 job) | Subprocess in `iter_custom` | The substantive measurement — tests CLI generation speed |
| First `cargo build` | Measured once, recorded as annotation | Too slow for criterion iteration; gates "buildable" |
| Server starts, serves `/health` | Recorded once in a smoke test, not criterion | Gate: service reaches readiness |

**Baseline usage**: Before a phase that touches `ferro-cli` generation, run:

```bash
cargo bench -p ferro-cli --bench time_to_working_app -- --save-baseline before
```

After the phase:

```bash
cargo bench -p ferro-cli --bench time_to_working_app -- --load-baseline before
```

Criterion reports percentage regressions at the configured noise threshold (2 % default).

**Why not `hyperfine`**: hyperfine is a CLI-level benchmarking tool with no Rust API. It would
require an external binary on CI, violating the no-external-build-tooling constraint. Criterion
runs as a Rust test binary — `cargo bench` is sufficient. hyperfine would be appropriate for
one-off developer analysis (e.g. comparing two branch builds at the command line), but as a CI
gate it adds a non-cargo dependency.

---

### Supporting: `proptest` — Property Testing (Already Present)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `proptest` | `1` (already in workspace) | Property-based tests for `derive_intents`: assert invariants over generated `ServiceDef` inputs | Already used in `ferro-reservation` and `ferro-projection`. For COMP-02, add proptest strategies to `ferro-projections` to verify: (1) `derive_intents` always returns at least one result; (2) all returned confidences are in [0.0, 1.0]; (3) a `Primary` hint always produces confidence 1.0 at position 0. These complement the snapshot tests — snapshots catch regressions on named fixtures; proptest catches invariant violations across the full input space. |

**Recommended Cargo.toml entry** (in `ferro-projections/Cargo.toml`):

```toml
[dev-dependencies]
proptest = "1"
```

Version `1.11.0` is current. Cargo semver `"1"` resolves to the latest 1.x automatically.
No change needed if this is a new dependency; add it directly in `ferro-projections`.

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `hyperfine` | External binary, violates no-external-build-tooling rule | `criterion` with `iter_custom` for process timing |
| `rmcp-in-process-transport` crate | Requires `rmcp >= 1.5`; workspace pins `0.12`; would force an rmcp major bump across all three crates that use it | Use `tokio::io::duplex` + rmcp 0.12's built-in `transport-async-rw` (default feature) |
| `divan` | Newer benchmark crate, requires nightly for some measurement backends | `criterion` 0.8 is stable-only and already well-understood |
| `assert_cmd` / `trycmd` | Useful for CLI snapshot testing but adds a dependency for a problem already handled by the combination of insta + criterion `iter_custom` subprocess | Keep as a potential addition if CLI command output regression testing becomes its own harness phase |
| Golden file via hand-rolled JSON file comparison | Readable at first, becomes a maintenance burden as the catalog grows; no interactive review tooling | `insta` with `cargo insta review` |
| Separate `ferro-validation` crate | Adds workspace member overhead for tests that belong in existing crates | Tests live in `ferro-projections`, `ferro-json-ui`, and `ferro-mcp`; benchmarks live in `ferro-cli` |
| `criterion` `plotters` feature | Pulls in a large transitive dependency tree for HTML charts that are not needed in CI | Use `default-features = false`; charts are optional and can be enabled locally |

---

## Integration with Existing cargo test Infrastructure

The validation harnesses integrate with the existing `cargo test --all-features` gate:

| Harness | Where | Gate behavior |
|---------|-------|--------------|
| COMP-02 catalog snapshot tests | `ferro-projections/tests/catalog/` and `ferro-json-ui/tests/catalog_render.rs` | Run as part of `cargo test --all-features`; snapshot mismatch = CI fail |
| COMP-02 proptest invariants | `ferro-projections/tests/catalog/proptest.rs` | Part of `cargo test --all-features` |
| COMP-03 agent success rate | `ferro-mcp/tests/agent_success_rate.rs` (all tests `#[ignore]`) | Excluded from normal gate; run explicitly with `--include-ignored` |
| COMP-04 scaffold benchmark | `ferro-cli/benches/time_to_working_app.rs` | `cargo bench` only; not `cargo test`; separate CI step |

The `generate_schemas.rs` test in `ferro-projections/tests/` regenerates
`docs/protocol/schemas/*.json` and dirties the tree. COMP-02 catalog tests must not regenerate
anything — they only read `ferro_projections` types, so there is no tree-dirtying concern.

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|----------------|-------|
| `insta 1.48` | `serde_json 1` (already in workspace) | JSON feature uses `serde_json`; no new transitive dep |
| `insta 1.48` | `schemars 1` | No interaction; insta is dev-only |
| `criterion 0.8.2` | Rust 1.88+ (workspace MSRV is 1.88.0) | criterion 0.8 targets the last three stable minors; workspace MSRV matches |
| `proptest 1.11` | Rust 2021 edition | Compatible; no nightly features |
| `rmcp 0.12` + `transport-async-rw` | `tokio 1` (already in workspace) | `transport-async-rw` default feature uses `tokio::io::{AsyncRead, AsyncWrite}` |

---

## Cargo.toml Changes Summary

**`ferro-projections/Cargo.toml`** — add to `[dev-dependencies]`:

```toml
insta = { version = "1.48", features = ["json", "redactions"] }
proptest = "1"

[profile.dev.package.insta]
opt-level = 3
[profile.dev.package.similar]
opt-level = 3
```

**`ferro-json-ui/Cargo.toml`** — add to `[dev-dependencies]`:

```toml
insta = { version = "1.48", features = ["json"] }
```

**`ferro-mcp/Cargo.toml`** — add to `[dev-dependencies]`:

```toml
rmcp = { version = "0.12", features = ["client", "transport-async-rw"] }
```

**`ferro-cli/Cargo.toml`** — add bench target and dependency:

```toml
[[bench]]
name = "time_to_working_app"
harness = false

[dev-dependencies]
criterion = { version = "0.8", default-features = false, features = ["cargo_bench_support"] }
```

The `profile.dev.package.insta` / `similar` optimization entries belong in the root
`Cargo.toml` (workspace-level profile overrides), not in the crate-level `Cargo.toml`.

---

## Sources

- `docs.rs/insta/latest/insta/` — version 1.48.0 confirmed; feature list verified (HIGH)
- `docs.rs/criterion/latest/criterion/` — version 0.8.2 confirmed (HIGH)
- `docs.rs/proptest/latest/proptest/` — version 1.11.0 confirmed (HIGH)
- `docs.rs/rmcp/0.12.0/features` — `transport-async-rw` confirmed as default feature in 0.12 (HIGH)
- `ferro-api-mcp/tests/e2e.rs` — existing `rmcp` client test pattern with `call_tool` verified by direct read (HIGH)
- `ferro-projections/Cargo.toml` — no existing snapshot or benchmark deps confirmed by direct read (HIGH)
- `ferro-mcp/Cargo.toml` — rmcp 0.12 with `server` + `transport-io` confirmed (HIGH)
- Context7 `/mitsuhiko/insta` — `assert_json_snapshot!`, `cargo insta test`, workspace integration (HIGH)
- Context7 `/bheisler/criterion.rs` — `iter_custom`, benchmark groups, `Criterion::default()` (HIGH)
- [insta.rs](https://insta.rs/) — canonical docs (HIGH)
- [criterion crates.io](https://crates.io/crates/criterion) — 0.8.2 latest confirmed (HIGH)
- [rmcp transport docs](https://docs.rs/rmcp/latest/rmcp/transport/index.html) — transport-async-rw confirmed (HIGH)

---
*Stack research for: ferro v13.0 Compressive Validation (COMP-02, COMP-03, COMP-04)*
*Researched: 2026-06-12*
