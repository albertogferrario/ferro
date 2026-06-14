# Phase 225: Release Workflow rustls Migration and E2E CLI-from-Release Test - Pattern Map

**Mapped:** 2026-06-14
**Files analyzed:** 18 Cargo.toml changes across 15 files + 2 workflow files
**Analogs found:** 17 / 17 (all files have a direct analog in the codebase)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `ferro-cli/Cargo.toml` | config | build | `ferro-storage/Cargo.toml` (reqwest rustls) | exact |
| `framework/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-queue/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-mcp/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-orm/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-audit/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-migration/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-projection/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-deployments/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-reservation/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-mcp-oauth/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-mcp-server/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `app/Cargo.toml` | config | build | `ferro-cli/Cargo.toml` sea-orm lines | exact |
| `ferro-notifications/Cargo.toml` | config | build | `ferro-storage/Cargo.toml` (rustls pattern) | role-match |
| `ferro-mcp/Cargo.toml` (reqwest) | config | build | `ferro-storage/Cargo.toml` | exact |
| `.github/workflows/release.yml` | config | CI/CD | `release.yml` existing build job + `ci.yml` scaffold-smoke job | exact |
| `.github/workflows/ci.yml` (ref only) | config | CI/CD | `ci.yml` scaffold-smoke job | exact |

---

## Pattern Assignments

### Class A: reqwest rustls-tls migration

**Analog:** `ferro-storage/Cargo.toml` line 22

This is the ONLY file in the workspace that already uses the correct form. All other reqwest users must copy this.

**Correct form** (`ferro-storage/Cargo.toml`, line 22):
```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

**ferro-cli/Cargo.toml** — add `blocking` to match current features, add `default-features = false`, add `rustls-tls`:

Current (line 48):
```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
```
After:
```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```

**Coherence crates** (ferro-mcp line 35, ferro-whatsapp line 14, ferro-api-mcp line 20+33, ferro-notifications line 28 [reqwest], ferro-ai line 16):

Current form (same across all):
```toml
reqwest = { version = "0.12", features = ["json"] }
```
After:
```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

ferro-ai is optional behind a feature flag but same fix applies:
```toml
# Current (ferro-ai/Cargo.toml line 16)
reqwest = { version = "0.12", features = ["json", "stream"], optional = true }
# After
reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls"], optional = true }
```

ferro-notifications has reqwest in both `[dependencies]` (line 28) and `[dev-dependencies]` (line 35):
```toml
# dev-dependencies — same fix
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

---

### Class B: sea-orm runtime-tokio-native-tls → runtime-tokio-rustls

**Rule:** Only the TLS runtime feature token changes. All other features (`sqlx-sqlite`, `sqlx-postgres`, `macros`, `with-uuid`, `with-chrono`, etc.) are preserved verbatim.

Each entry below shows the exact current line (from live file) and the after form.

#### `ferro-cli/Cargo.toml` (lines 35–36)

Current:
```toml
sea-orm-migration = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "with-uuid", "with-chrono"] }
```
After:
```toml
sea-orm-migration = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "with-uuid", "with-chrono"] }
```

#### `framework/Cargo.toml` (line 52)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-postgres", "sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-postgres", "sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-queue/Cargo.toml` (line 20)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-mcp/Cargo.toml` (line 27)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"] }
```

#### `ferro-orm/Cargo.toml` (line 19)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-audit/Cargo.toml` (line 25)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-migration/Cargo.toml` (line 20)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-projection/Cargo.toml` (line 30)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-deployments/Cargo.toml` (line 19)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros", "with-chrono"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros", "with-chrono"] }
```

#### `ferro-reservation/Cargo.toml` (line 39)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-mcp-oauth/Cargo.toml` (lines 16 and 30 — two occurrences)

Current (line 16, `[dependencies]`):
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
```
Current (line 30, `[dev-dependencies]`):
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
# [dependencies]
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"] }
# [dev-dependencies]
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
```

#### `ferro-mcp-server/Cargo.toml` (line 30)

Current:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
```
After:
```toml
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"] }
```

#### `app/Cargo.toml` (lines 15–16)

Current:
```toml
sea-orm-migration = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
```
After:
```toml
sea-orm-migration = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"] }
```

---

### Class C: lettre tokio1-native-tls → tokio1-rustls-tls

**Analog:** `ferro-storage/Cargo.toml` rustls pattern applied to lettre.

**`ferro-notifications/Cargo.toml`** (line 27):

Current:
```toml
lettre = { version = "0.11", features = ["tokio1-native-tls", "builder", "smtp-transport"] }
```
After:
```toml
lettre = { version = "0.11", features = ["tokio1-rustls-tls", "builder", "smtp-transport"] }
```

---

### Class D: `.github/workflows/release.yml` — drop `cross`, add aarch64 native cross-linker

**Analog:** Existing `release.yml` build job (lines 1–90, read above) for native-build steps; `ci.yml` scaffold-smoke job (lines 93–103) for e2e job structure.

#### D-1: Remove `cross: true` from the matrix and the two `cross` steps

Current aarch64 matrix entry (lines 24–27):
```yaml
- target: aarch64-unknown-linux-gnu
  os: ubuntu-latest
  archive: tar.gz
  cross: true
```
After (remove `cross: true`):
```yaml
- target: aarch64-unknown-linux-gnu
  os: ubuntu-latest
  archive: tar.gz
```

Current cross-only steps (lines 57–63) to REMOVE entirely:
```yaml
- name: Install cross
  if: matrix.cross
  run: cargo install cross --git https://github.com/cross-rs/cross

- name: Build (cross)
  if: matrix.cross
  run: cross build --release --target ${{ matrix.target }} -p ferro-cli
```

#### D-2: Add apt cross-linker install step BEFORE the native build step

Insert after "Add Rust target" step (after line 55):
```yaml
- name: Install cross-compilation toolchain (aarch64)
  if: matrix.target == 'aarch64-unknown-linux-gnu'
  run: |
    sudo apt-get update -q
    sudo apt-get install -y --no-install-recommends gcc-aarch64-linux-gnu
```

#### D-3: Update the native build step to pass CC + linker env for aarch64

Current native build step (lines 65–67):
```yaml
- name: Build (native)
  if: '!matrix.cross'
  run: cargo build --release --target ${{ matrix.target }} -p ferro-cli
```
After (remove the `if` guard since cross is gone; add env block):
```yaml
- name: Build (native)
  run: cargo build --release --target ${{ matrix.target }} -p ferro-cli
  env:
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: aarch64-linux-gnu-gcc
    CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc
```

Note: env vars are set for all matrix rows but only matter on the aarch64 row; harmless on others.

#### D-4: Keep "Add Rust target" step as-is (already covers aarch64 without `cross` guard)

Current (lines 53–55) — keep unchanged:
```yaml
- name: Add Rust target
  if: '!matrix.cross'
  run: rustup target add ${{ matrix.target }}
```
After removing `cross: true` from the matrix, update the `if` guard:
```yaml
- name: Add Rust target
  run: rustup target add ${{ matrix.target }}
```

---

### Class E: `.github/workflows/release.yml` — new `e2e-from-release` job

**Analog:** `ci.yml` scaffold-smoke job (lines 93–103) for job structure; `release.yml` `release` job (lines 91–114) for `needs: build` pattern; Dockerfile CMD (lines 50–69) for the step sequence.

The e2e has two trigger modes (tag-push uses the built artifact; schedule/dispatch installs from crates.io). Based on RESEARCH pitfall 4, implement as **two separate jobs** to avoid the `needs:` skip problem:

**Job 1: `e2e-tag`** — runs only on tag push, needs build artifact

```yaml
e2e-tag:
  name: E2E from release artifact (tag)
  needs: build
  if: github.event_name == 'push'
  runs-on: ubuntu-latest
  continue-on-error: true   # TODO: flip to false after template-alignment phase ships
  steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: "1.88.0"

    - name: Download linux x86_64 artifact
      uses: actions/download-artifact@v4
      with:
        name: ferro-x86_64-unknown-linux-gnu
        path: ./dist

    - name: Extract and install ferro binary
      run: |
        tar -xzf ./dist/ferro-${{ github.ref_name }}-x86_64-unknown-linux-gnu.tar.gz -C ./dist
        chmod +x ./dist/ferro
        echo "$PWD/dist" >> $GITHUB_PATH

    - name: Run COMP-04 scaffold sequence
      run: |
        set -euo pipefail
        TMPDIR=$(mktemp -d)
        ferro new bench-app --no-interaction --no-git -C "$TMPDIR"
        cd "$TMPDIR/bench-app"
        ferro make:auth
        ferro make:scaffold --no-smart-defaults -q -y --api Article title:string body:text
        ferro make:scaffold --no-smart-defaults -q -y --api Product name:string price:float
        ferro make:scaffold --no-smart-defaults -q -y --api Order status:string total:float
        ferro make:scaffold --no-smart-defaults -q -y Post title:string body:text
        ferro make:job EmailNotification
        # DO NOT add [patch.crates-io] — must build against published ferro-rs
        RUSTFLAGS="" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo build
```

**Job 2: `e2e-drift`** — runs on schedule/dispatch; acquires binary from crates.io

```yaml
e2e-drift:
  name: E2E published artifact drift check (scheduled)
  if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
  runs-on: ubuntu-latest
  continue-on-error: true   # TODO: flip to false after template-alignment phase ships
  steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: "1.88.0"

    - name: Install ferro-cli from crates.io
      run: cargo install ferro-cli

    - name: Run COMP-04 scaffold sequence
      run: |
        set -euo pipefail
        TMPDIR=$(mktemp -d)
        ferro new bench-app --no-interaction --no-git -C "$TMPDIR"
        cd "$TMPDIR/bench-app"
        ferro make:auth
        ferro make:scaffold --no-smart-defaults -q -y --api Article title:string body:text
        ferro make:scaffold --no-smart-defaults -q -y --api Product name:string price:float
        ferro make:scaffold --no-smart-defaults -q -y --api Order status:string total:float
        ferro make:scaffold --no-smart-defaults -q -y Post title:string body:text
        ferro make:job EmailNotification
        RUSTFLAGS="" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo build
```

**Cron cadence (discretion):** weekly (`cron: '0 6 * * 1'`) is sufficient — drift is introduced at publish time, not continuously. Add to the `on:` block of `release.yml`:

```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
  schedule:
    - cron: '0 6 * * 1'   # Weekly Monday 06:00 UTC
```

---

## Shared Patterns

### Pattern: `RUSTFLAGS=""` in scaffold build steps
**Source:** `ferro-cli/tests/benchmark_new_project.rs` line 167 (`.env_remove("RUSTFLAGS")`)
**Apply to:** All e2e job `cargo build` steps
**Rationale:** CI sets `RUSTFLAGS: -Dwarnings` globally; a freshly scaffolded starter app has expected unused-import/dead-code warnings. The build-compiles test, not warning-clean test.

In Rust test code:
```rust
Command::new("cargo")
    .args(["build"])
    .current_dir(&project_dir)
    .env_remove("RUSTFLAGS")   // line 167 — clears global CI -Dwarnings
    .status()
```

In CI YAML shell step:
```bash
RUSTFLAGS="" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo build
```

### Pattern: scaffold sequence command arguments
**Source:** `ferro-cli/tests/benchmark_new_project.rs` lines 21–174 (the `scaffold_builds_against_workspace_ferro` function)
**Apply to:** Both e2e CI jobs

The exact command sequence proven to exercise the COMP-04 surface:
```
ferro new bench-app --no-interaction --no-git      # CWD = parent dir; creates bench-app/
ferro make:auth                                     # CWD = bench-app/
ferro make:scaffold --no-smart-defaults -q -y --api Article title:string body:text
ferro make:scaffold --no-smart-defaults -q -y --api Product name:string price:float
ferro make:scaffold --no-smart-defaults -q -y --api Order status:string total:float
ferro make:scaffold --no-smart-defaults -q -y Post title:string body:text     # full-stack, no --api
ferro make:job EmailNotification
```

Key: flags (`--no-smart-defaults -q -y --api`) MUST precede the positional `NAME` argument. Placing them after the fields causes clap to parse them as field names. See comment in `benchmark_new_project.rs` line 43–44.

### Pattern: NO `[patch.crates-io]` in from-release e2e
**Source:** Contrast with `ferro-cli/tests/benchmark_new_project.rs` lines 141–156 (workspace-smoke deliberately adds the patch)
**Apply to:** Both e2e CI jobs

The workspace-smoke test adds:
```rust
// lines 147-148 — workspace-smoke ONLY, NOT for from-release e2e
"\n[patch.crates-io]\nferro-rs = {{ path = \"{}\" }}\n",
framework_path.display()
```
The from-release e2e MUST NOT add this block. The generated Cargo.toml resolves `ferro-rs` from crates.io — that is exactly what the test is validating.

### Pattern: `actions/upload-artifact@v4` / `actions/download-artifact@v4` pairing
**Source:** `release.yml` lines 85–89 (upload) and lines 101–104 (download in release job)

Upload (in build matrix job, lines 85–89):
```yaml
- name: Upload artifact
  uses: actions/upload-artifact@v4
  with:
    name: ferro-${{ matrix.target }}
    path: ferro-${{ github.ref_name }}-${{ matrix.target }}.${{ matrix.archive }}
```

Download (in downstream job, lines 101–104):
```yaml
- name: Download all artifacts
  uses: actions/download-artifact@v4
  with:
    path: artifacts
```

For the e2e-tag job, download only the x86_64-linux artifact by name:
```yaml
uses: actions/download-artifact@v4
with:
  name: ferro-x86_64-unknown-linux-gnu
  path: ./dist
```

### Pattern: `dtolnay/rust-toolchain@master` with pinned toolchain
**Source:** `ci.yml` lines 19–22 and 99–101
```yaml
- uses: dtolnay/rust-toolchain@master
  with:
    toolchain: "1.88.0"
```
Use `@master` (not `@stable`) to respect the pinned MSRV `1.88.0`. Do not use `@stable` in release.yml e2e jobs — that would drift from the CI-verified toolchain.

Note: existing `release.yml` line 45 uses `dtolnay/rust-toolchain@stable` without a pinned version. The e2e job should pin to `1.88.0` for consistency with CI.

### Pattern: `continue-on-error: true` with TODO comment (D-10)
**Source:** RESEARCH.md pitfall 5; no existing analog in the codebase (first use)
```yaml
continue-on-error: true   # TODO: flip to false after template-alignment phase ships
```
This is a deliberate temporary state. The planner must record the flip condition in PLAN.md explicitly.

---

## Verification Commands (for planner to encode as post-implementation steps)

These are not patterns to copy, but acceptance criteria the planner should include:

```bash
# Confirm no native-tls/openssl in ferro-cli tree
cargo tree -p ferro-cli --edges no-dev -e features \
  | grep -E 'native-tls|openssl-sys|openssl-' | wc -l
# Expected: 0

# Confirm ring is selected (not aws-lc-rs)
cargo tree -p ferro-cli --edges no-dev | grep "ring v"
cargo tree -p ferro-cli --edges no-dev | grep "aws-lc" | wc -l
# Expected: ring present, aws-lc count = 0

# Workspace build
cargo build -p ferro-cli
cargo test --all-features
cargo deny check
```

---

## No Analog Found

No files in this phase lack a codebase analog. All patterns have concrete references above.

---

## Metadata

**Analog search scope:** All `*/Cargo.toml` files, `.github/workflows/`, `ferro-cli/tests/`
**Files scanned:** 17 Cargo.toml files (grep), 3 workflow files, 1 test file, 1 Dockerfile
**Pattern extraction date:** 2026-06-14
