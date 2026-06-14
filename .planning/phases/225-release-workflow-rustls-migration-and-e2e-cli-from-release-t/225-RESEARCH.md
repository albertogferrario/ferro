# Phase 225: Release Workflow rustls Migration and E2E CLI-from-Release Test — Research

**Researched:** 2026-06-14
**Domain:** Rust TLS backend migration (native-tls → rustls/ring), CI/CD cross-compilation, GitHub Actions e2e
**Confidence:** HIGH (all critical claims verified from cargo metadata, crates.io feature tables, official docs)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** reqwest → `default-features = false, features = ["blocking", "json", "rustls-tls"]`; sea-orm + sea-orm-migration → `runtime-tokio-rustls`; lettre → `tokio1-rustls-tls`. Applied workspace-wide.
- **D-02:** rustls crypto provider = **ring**, not aws-lc-rs. Researcher must confirm exact feature wiring.
- **D-03:** ferro-wallet's `openssl = "0.10"` is out of scope.
- **D-04:** Drop cross/Docker for aarch64-unknown-linux-gnu; build natively with rustup target add + gcc-aarch64-linux-gnu. Fallback: keep cross for that target only if a residual C dep blocks it.
- **D-05:** cargo-deny must stay green after the swap.
- **D-06:** e2e runs actual released `ferro` binary, not `cargo run -p ferro-cli`. Builds against published ferro-rs.
- **D-07:** e2e job in release.yml (needs: build) + workflow_dispatch + schedule cron.
- **D-08:** Test surface = COMP-04 sequence (ferro new → make:auth → make:scaffold ×N → make:job → cargo build). Reuse benchmark_new_project.rs.
- **D-09:** Complement, do not replace, existing fast scaffold-smoke job.
- **D-10:** From-release e2e may go RED against current published ferro-rs (COMP-04 drift). Planner must pick: (a) run alignment phase first, or (b) land e2e in continue-on-error mode.

### Claude's Discretion
- Mechanism for aarch64 cross-linker (`.cargo/config.toml` vs workflow env)
- Whether to add x86_64-unknown-linux-musl as a fully-static target
- Cron cadence for scheduled from-release run
- `--release` vs debug build of generated app in e2e

### Deferred Ideas (OUT OF SCOPE)
- Scaffold-template ↔ published-library API alignment (separate phase)
- ferro-wallet OpenSSL → rustls/ring coherence follow-up
- x86_64-unknown-linux-musl fully-static target (only if trivial)
</user_constraints>

---

## Summary

This phase has two coupled deliverables: a workspace-wide TLS backend swap from native-tls/OpenSSL to rustls/ring, and a new e2e CI job that exercises the actual released `ferro` binary against the published `ferro-rs` library.

**The TLS migration is clean and low-risk.** All research confirms that the D-02 constraint (ring provider, not aws-lc-rs) is naturally satisfied by the feature flags specified in D-01: reqwest 0.12.x `rustls-tls` resolves through `__rustls-ring`, sea-orm 1.x `runtime-tokio-rustls` resolves through sqlx `tls-rustls-ring`, and lettre `tokio1-rustls-tls` resolves through the `ring` feature — all verified from live `cargo metadata`. No explicit `ring` dependency or `CryptoProvider::install_default()` call is needed. Ring 0.17.14 is already in the workspace lockfile (pulled by ferro-storage which already uses `rustls-tls`).

**The aarch64 cross-compile without `cross`/Docker requires explicit env setup for ring.** Ring 0.17.x needs the target C compiler (`aarch64-linux-gnu-gcc`) announced via `CC_aarch64_unknown_linux_gnu` in addition to the linker. Without this env var, ring's build script can misdetect or pick up the wrong toolchain.

**The e2e design is well-understood.** The existing `scaffold_builds_against_workspace_ferro` test in `benchmark_new_project.rs` contains the full 5-step sequence; adapting it for the from-release case requires replacing the `CARGO_BIN_EXE_ferro` path with an externally-downloaded binary path and removing the `[patch.crates-io]` block. The two-mode design (release artifact in tag-triggered runs, `cargo install ferro-cli` in scheduled/dispatch-only runs) requires the e2e job to detect its trigger and acquire the binary accordingly.

**D-10 sequencing risk is real.** The published ferro-rs 0.2.55 scaffold had 52 compile errors (COMP-04). If the from-release e2e runs against the current published version it will fail. The planner must pick `continue-on-error: true` until a template-alignment phase ships and publishes a clean version.

**Primary recommendation:** Land the TLS migration (D-01..D-05) in one wave; land the e2e in a second wave with `continue-on-error: true` so the job exists but does not block release. When template alignment lands, flip `continue-on-error: false`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| TLS provider selection | Build system (Cargo features) | None | Feature flags in Cargo.toml determine which TLS crate is compiled in; no runtime config |
| Cross-compilation linker | CI/CD runner env | `.cargo/config.toml` | Linker identity must be known to the build system before compilation; env var or config.toml both work |
| Released binary acquisition | CI job step | None | `actions/download-artifact` for tag runs; `cargo install` for scheduled/dispatch |
| e2e scaffold sequence | CI job shell steps | ferro-cli test harness | Runs the real `ferro` binary; can reuse test harness code as a model but must run the installed binary |
| Published library compilation | Generated app's `cargo build` | None | The generated Cargo.toml pins `ferro-rs` from crates.io; no workspace path-dep |
| cargo-deny audit | CI deny job | None | Evaluates dependency graph licenses and advisories; no code changes needed |

---

## Standard Stack

### Core (TLS migration)
| Library | Current Version (workspace) | Replaces | Why |
|---------|----------------------------|----------|-----|
| reqwest | 0.12.28 | reqwest default-tls | `rustls-tls` feature → `__rustls-ring` → ring, no libssl needed |
| sea-orm | 1.1.19 | runtime-tokio-native-tls | `runtime-tokio-rustls` → sqlx `tls-rustls-ring` → ring |
| sea-orm-migration | 1.1.19 | runtime-tokio-native-tls | Same chain as sea-orm |
| lettre | 0.11.19 | tokio1-native-tls | `tokio1-rustls-tls` → `rustls-tls` → ring |
| ring | 0.17.14 (already in lockfile) | openssl-sys | Ring is already transitively pulled by ferro-storage; no new dependency |

### Supporting (CI)
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| actions/upload-artifact | v4 | Upload released binary as artifact | In build matrix jobs |
| actions/download-artifact | v4 | Download binary in e2e job | In e2e-from-release job |
| gcc-aarch64-linux-gnu | system apt | Cross-linker for aarch64 target | Only in aarch64-unknown-linux-gnu build matrix row |
| cargo install ferro-cli | latest stable | Acquire binary in scheduled/dispatch runs | When no release artifact is available |

**Version verification:** [VERIFIED: cargo metadata] — reqwest 0.12.28, sea-orm 1.1.19, ring 0.17.14 are live workspace versions as of research date.

---

## TLS Feature Wiring — Verified Provider Chain

This is the load-bearing finding for D-02. All chains verified from live `cargo metadata` output.

### reqwest 0.12.28

```
rustls-tls
  → rustls-tls-webpki-roots
    → rustls-tls-webpki-roots-no-provider (webpki-roots + hyper-rustls/webpki-tokio + __rustls)
    → __rustls-ring (hyper-rustls/ring, tokio-rustls/ring, rustls/ring, quinn/ring)
```

`rustls-tls` **resolves to ring**. [VERIFIED: cargo metadata, reqwest 0.12.28 feature table]

The current ferro-cli declaration `reqwest = { version = "0.12", features = ["blocking", "json"] }` activates the **default** feature which includes `default-tls` → `native-tls-crate`. This is the OpenSSL path. Fix: `reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }`.

Note: ferro-storage already uses the correct declaration `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }` — the same pattern applies to ferro-cli.

### sea-orm 1.1.19 + sqlx 0.8.6

```
sea-orm runtime-tokio-rustls
  → sqlx?/runtime-tokio-rustls        (sea-orm feature chain)
  → sqlx runtime-tokio-rustls
    → runtime-tokio + tls-rustls-ring  (sqlx 0.8.6 feature definition)
    → tls-rustls-ring-webpki           (sqlx tls-rustls-ring chain)
      → sqlx-core/_tls-rustls-ring-webpki
        → _tls-rustls + rustls/ring + webpki-roots
```

`runtime-tokio-rustls` **resolves to ring via sqlx 0.8.6's own aliasing**. [VERIFIED: cargo metadata, both sea-orm 1.1.19 and sqlx 0.8.6 feature tables]

Critically: `sqlx tls-rustls` is an alias for `tls-rustls-ring` in sqlx 0.8.6 (not aws-lc-rs). The aws-lc-rs path requires the explicit `tls-rustls-aws-lc-rs` feature.

### lettre 0.11.19

```
tokio1-rustls-tls
  → tokio1-rustls + rustls-tls
  → rustls-tls → [webpki-roots, rustls, ring]
  → ring → rustls?/ring
```

`tokio1-rustls-tls` **resolves to ring**. [VERIFIED: cargo metadata, lettre 0.11.19 feature table]

### Summary: No explicit ring pinning needed

None of the three migrations require adding an explicit `ring` dependency or calling `CryptoProvider::install_default()` at runtime. The feature flags in D-01 naturally select ring throughout the chain as of current crate versions. If sea-orm or sqlx upgrades to a version where `runtime-tokio-rustls` defaults to aws-lc-rs, the verification command will catch it.

---

## Architecture Patterns

### System Architecture Diagram

```
Cargo.toml feature change
  (runtime-tokio-native-tls → runtime-tokio-rustls)
        │
        ▼
sea-orm 1.1.19 → sqlx 0.8.6 [runtime-tokio-rustls]
                       │
                       ▼
              tls-rustls-ring → tls-rustls-ring-webpki
                       │
                       ▼
              sqlx-core: rustls/ring + webpki-roots
                       │
                       ▼
              ring 0.17.14 (already in lockfile)
                       │
                       ▼
              NO openssl-sys, NO native-tls in ferro-cli tree


reqwest `default-features=false, features=[rustls-tls]`
        │
        ▼
  __rustls-ring → rustls/ring + hyper-rustls/ring
        │
        ▼
  ring 0.17.14 (same crate, deduplicated)
  NO native-tls-crate, NO tokio-native-tls, NO openssl-sys


release.yml tag trigger
  ├── build (matrix: 4 targets)
  │     ├── x86_64-linux (ubuntu-latest, native)
  │     ├── aarch64-linux (ubuntu-latest, cross-linker, D-04)
  │     ├── x86_64-darwin (macos-latest, native)
  │     ├── aarch64-darwin (macos-latest, native)
  │     └── x86_64-windows (windows-latest, native)
  │
  ├── release (needs: build) — GitHub release creation
  │
  ├── e2e-from-release (needs: build) — NEW JOB
  │     ├── download x86_64-linux ferro artifact
  │     ├── chmod +x ./ferro
  │     ├── run COMP-04 sequence (ferro new → make:auth → make:scaffold ×N → make:job)
  │     ├── cargo build (against published ferro-rs from crates.io)
  │     └── assert exit 0
  │
  └── update-install-script (needs: release)


schedule / workflow_dispatch (no release artifact available)
  └── e2e-from-release (standalone trigger)
        ├── cargo install ferro-cli --locked (acquire binary)
        └── same COMP-04 sequence + cargo build
```

### Recommended Project Structure Changes

```
.cargo/config.toml                  # Add [target.aarch64-unknown-linux-gnu] linker
.github/workflows/release.yml       # Add e2e-from-release job; drop cross; add apt step
ferro-cli/Cargo.toml               # reqwest: add default-features=false, rustls-tls; sea-orm: swap TLS feature
framework/Cargo.toml               # sea-orm: runtime-tokio-native-tls → runtime-tokio-rustls
ferro-queue/Cargo.toml             # same swap
ferro-mcp/Cargo.toml               # same swap
ferro-orm/Cargo.toml               # same swap
ferro-audit/Cargo.toml             # same swap
ferro-migration/Cargo.toml         # same swap
ferro-projection/Cargo.toml        # same swap
ferro-deployments/Cargo.toml       # same swap
ferro-reservation/Cargo.toml       # same swap
ferro-mcp-oauth/Cargo.toml         # same swap (two occurrences: [dependencies] + [dev-dependencies])
ferro-mcp-server/Cargo.toml        # same swap
app/Cargo.toml                     # same swap (sea-orm + sea-orm-migration)
ferro-notifications/Cargo.toml     # lettre: tokio1-native-tls → tokio1-rustls-tls
```

### Pattern 1: reqwest rustls-tls (correct form)

```toml
# Source: ferro-storage/Cargo.toml (already the model in this workspace)
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```

The `default-features = false` is mandatory. Without it, reqwest enables `default-tls` (native-tls) even when `rustls-tls` is also listed — both would be compiled.

### Pattern 2: sea-orm rustls swap

```toml
# Before:
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
# After:
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"] }
```

Only the TLS runtime feature changes. All other features (`sqlx-sqlite`, `sqlx-postgres`, `macros`, `with-uuid`, `with-chrono`, etc.) remain untouched.

### Pattern 3: aarch64 cross-linker in release.yml

```yaml
- name: Install cross-compilation toolchain (aarch64)
  if: matrix.target == 'aarch64-unknown-linux-gnu'
  run: |
    sudo apt-get update -q
    sudo apt-get install -y --no-install-recommends gcc-aarch64-linux-gnu

- name: Build (native, with cross-linker for aarch64)
  if: '!matrix.cross'
  run: cargo build --release --target ${{ matrix.target }} -p ferro-cli
  env:
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: aarch64-linux-gnu-gcc
    CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc
```

The `CC_aarch64_unknown_linux_gnu` env var is required specifically for ring's build script. Ring uses the `cc` crate to compile its C/asm files; without explicit `CC_*`, it may attempt to use the host compiler or misdetect `arm-linux-gnueabihf-gcc` (32-bit ARM) on some runner configurations. [VERIFIED: ring issue #2131, cargo-dist issue #1378 — these are the canonical documented failures]

Alternatively, add to `.cargo/config.toml`:
```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```
But `.cargo/config.toml` only sets the linker, not `CC`. The env var approach covers both, making the workflow self-contained.

### Pattern 4: e2e-from-release job structure

```yaml
e2e-from-release:
  name: E2E from released binary
  needs: build           # Only when triggered by tag push
  if: github.event_name == 'push' || github.event_name == 'workflow_dispatch' || github.event_name == 'schedule'
  runs-on: ubuntu-latest
  continue-on-error: true   # D-10: flip to false once template alignment ships
  steps:
    - uses: actions/checkout@v4

    # Mode A: tag push — download the artifact built in the build job
    - name: Download linux x86_64 artifact
      if: github.event_name == 'push'
      uses: actions/download-artifact@v4
      with:
        name: ferro-x86_64-unknown-linux-gnu
        path: ./dist

    - name: Extract binary (tag push)
      if: github.event_name == 'push'
      run: |
        tar -xzf ./dist/ferro-*.tar.gz -C ./dist
        chmod +x ./dist/ferro
        echo "$PWD/dist" >> $GITHUB_PATH

    # Mode B: scheduled / workflow_dispatch — install from crates.io
    - name: Install ferro-cli from crates.io (scheduled/dispatch)
      if: github.event_name != 'push'
      run: |
        cargo install ferro-cli --locked
        # ferro binary is now in $HOME/.cargo/bin, already on PATH

    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: "1.88.0"

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
        env RUSTFLAGS="" cargo build
```

Note: `ferro new` currently expects to run from a parent directory and creates the project subdir. The exact invocation pattern is already proven in `benchmark_new_project.rs` (uses `current_dir(tmp.path())`).

### Anti-Patterns to Avoid

- **`rustls-tls` without `default-features = false` on reqwest:** Both TLS stacks compile; native-tls stays in the tree. Always set `default-features = false` for reqwest.
- **Relying on `tls-rustls` alone in future sea-orm versions:** sqlx 0.8.6 aliases `tls-rustls` to `tls-rustls-ring`, but this is not guaranteed forever. The verification command (see Validation Architecture) catches any drift.
- **Patching the generated Cargo.toml with `[patch.crates-io]` in the from-release e2e:** That would build against the workspace ferro path-dep, defeating the entire purpose of the test. The workspace-smoke test does this; the from-release e2e must NOT.
- **Using `cross` for all targets:** After the rustls migration, `cross` / Docker is only needed if a non-TLS C dependency remains. Drop it for aarch64-linux-gnu once rustls is in.
- **Adding `ring` as an explicit workspace dependency:** Ring is already in the lockfile via ferro-storage. Explicit addition creates a duplicate-version risk if ring releases a new version; let the feature flags pull it transitively.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TLS in Rust | Custom TLS layer | rustls + ring | Memory-safe, no OpenSSL build dep, already proven in ferro-storage |
| Cross-compilation | Docker or cross tool | rustup target add + gcc-aarch64-linux-gnu apt | Once OpenSSL is gone, the only C needed is the ring assembler handled by gcc |
| Crypto provider selection | Manual CryptoProvider::install_default() | Feature flags (rustls-tls, runtime-tokio-rustls, tokio1-rustls-tls) | The crate-level feature chain selects ring automatically; no runtime call needed |
| Binary acquisition in CI | Rebuild the binary in the e2e job | actions/download-artifact@v4 (tag) / cargo install (scheduled) | Must use the exact published artifact; rebuilding would test path-deps, not the release |

---

## Complete Workspace-Wide Migration Checklist

All files require exactly one change per occurrence unless noted. [VERIFIED: grep of all workspace Cargo.toml files]

### TLS Runtime Feature Swap (native-tls → rustls)

| Crate | File | Change |
|-------|------|--------|
| ferro-cli | ferro-cli/Cargo.toml | sea-orm: native-tls → rustls (line 36) |
| ferro-cli | ferro-cli/Cargo.toml | sea-orm-migration: native-tls → rustls (line 35) |
| ferro-cli | ferro-cli/Cargo.toml | reqwest: add `default-features=false`, add `rustls-tls` (line 48) |
| framework | framework/Cargo.toml | sea-orm: native-tls → rustls (line 52) |
| ferro-queue | ferro-queue/Cargo.toml | sea-orm: native-tls → rustls (line 20) |
| ferro-mcp | ferro-mcp/Cargo.toml | sea-orm: native-tls → rustls (line 27) |
| ferro-orm | ferro-orm/Cargo.toml | sea-orm: native-tls → rustls (line 19) |
| ferro-audit | ferro-audit/Cargo.toml | sea-orm: native-tls → rustls (line 25) |
| ferro-migration | ferro-migration/Cargo.toml | sea-orm: native-tls → rustls (line 20) |
| ferro-projection | ferro-projection/Cargo.toml | sea-orm: native-tls → rustls (line 30) |
| ferro-deployments | ferro-deployments/Cargo.toml | sea-orm: native-tls → rustls (line 19) |
| ferro-reservation | ferro-reservation/Cargo.toml | sea-orm: native-tls → rustls (line 39) |
| ferro-mcp-oauth | ferro-mcp-oauth/Cargo.toml | sea-orm [dependencies]: native-tls → rustls (line 16) |
| ferro-mcp-oauth | ferro-mcp-oauth/Cargo.toml | sea-orm [dev-dependencies]: native-tls → rustls (line 30) |
| ferro-mcp-server | ferro-mcp-server/Cargo.toml | sea-orm: native-tls → rustls (line 30) |
| app | app/Cargo.toml | sea-orm: native-tls → rustls (line 16) |
| app | app/Cargo.toml | sea-orm-migration: native-tls → rustls (line 15) |
| ferro-notifications | ferro-notifications/Cargo.toml | lettre: tokio1-native-tls → tokio1-rustls-tls (line 27) |

**Total occurrences: 18 changes across 15 files.**

Note: Other reqwest usages (ferro-notifications, ferro-ai, ferro-api-mcp, ferro-mcp, ferro-whatsapp) that use `reqwest = { version = "0.12", features = ["json"] }` also implicitly pull native-tls via reqwest's default feature. D-01 scopes the mandatory fix to `ferro-cli`. For coherence, these should also add `default-features = false, features = ["...", "rustls-tls"]` in this phase (D-01 says "applied workspace-wide" for sea-orm; the reqwest coherence pass is a judgment call for the planner — document it explicitly in PLAN.md).

---

## Common Pitfalls

### Pitfall 1: reqwest `default-features` omission
**What goes wrong:** `reqwest = { version = "0.12", features = ["blocking", "json", "rustls-tls"] }` without `default-features = false` still compiles native-tls because `default-tls` is in reqwest's `default` feature.
**Why it happens:** Cargo feature unification — adding a feature does not remove default features unless explicitly disabled.
**How to avoid:** Always pair `rustls-tls` with `default-features = false` for reqwest. The existing ferro-storage declaration is the correct model.
**Warning signs:** `cargo tree -p ferro-cli --edges no-dev | grep native-tls` still shows output after the change.

### Pitfall 2: ring build-script compiler mismatch for aarch64
**What goes wrong:** `ring` build script invokes `arm-linux-gnueabihf-gcc` (32-bit ARM) instead of `aarch64-linux-gnu-gcc` when `CC_aarch64_unknown_linux_gnu` is not explicitly set, producing "error trying to exec 'cc1'" or "ARM assembler must define __ARM_ARCH".
**Why it happens:** Ring's build script (using the `cc` crate) probes for a C compiler. On ubuntu-latest, it may fall back to the wrong toolchain if the env is ambiguous.
**How to avoid:** Set both `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc` and `CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc` in the release.yml step environment. Install `gcc-aarch64-linux-gnu` first.
**Warning signs:** Build matrix aarch64 job fails on ring compilation with "cc1" or "ARM_ARCH" in the error message.

### Pitfall 3: e2e adds [patch.crates-io] (defeats the test)
**What goes wrong:** The `scaffold_builds_against_workspace_ferro` test adds a `[patch.crates-io]` block to build against path-deps. If the e2e job copies this pattern, it tests the workspace ferro, not the published ferro-rs. COMP-04 drift would not be caught.
**Why it happens:** Copy-paste from the existing test.
**How to avoid:** The from-release e2e must never add `[patch.crates-io]`. The generated `Cargo.toml` must be used as-is, resolving `ferro-rs` from crates.io.
**Warning signs:** The e2e job runs on a machine with no internet access OR its `cargo build` output shows "Compiling ferro-rs (path = ...)" instead of "Downloading ferro-rs".

### Pitfall 4: `needs: build` on a job that also fires on schedule
**What goes wrong:** `needs: build` only works when the `build` job runs. On `schedule` or `workflow_dispatch` triggers, the `build` job does not run, so `needs: build` would cause the e2e to be skipped entirely.
**Why it happens:** GitHub Actions job dependency (`needs:`) is workflow-run-scoped; if the dependency job doesn't exist in the run, the dependent job is skipped.
**How to avoid:** The e2e job must have conditional logic: if triggered by tag push, use `needs: build` and download-artifact; if triggered by schedule/dispatch, run as a standalone job that installs ferro-cli from crates.io. This likely requires two separate job definitions or a single job with `needs` omitted and artifact download gated by `github.event_name == 'push'`. Simpler: split into two jobs, one with `needs: build` (tag-only), one standalone (schedule/dispatch).

### Pitfall 5: D-10 — from-release e2e is immediately red
**What goes wrong:** The published ferro-rs at the time of phase ship still carries COMP-04 drift. The e2e job fails on `cargo build`. If `continue-on-error: false` (required job), it blocks all releases.
**Why it happens:** The from-release e2e intentionally tests the published artifact, which may not be aligned yet.
**How to avoid:** Ship the e2e with `continue-on-error: true`. Add a prominent comment: "# TODO: flip to false after template-alignment phase ships." Track as a follow-up action in PLAN.md.
**Warning signs:** The e2e job is required (no `continue-on-error`) and the current published version still produces 52 compile errors.

### Pitfall 6: disk pressure from `cargo build` of generated app
**What goes wrong:** The generated app's `cargo build` inside the e2e job downloads and compiles all of ferro-rs and its transitive deps from scratch (cold cache on the runner). Combined with the ferro workspace build in the same run, this can overflow the GH runner disk.
**Why it happens:** The workspace has `profile.dev debug=false` etc., but the generated app is a separate Cargo workspace on a temp path. It will use runner defaults (debug=true) and download the full crates.io dep set.
**How to avoid:** Set `CARGO_PROFILE_DEV_DEBUG=false` and `CARGO_INCREMENTAL=0` in the e2e job environment. Or use `cargo build` (not `--release`) and live with it — the generated app's dep graph is much smaller than the full ferro workspace. Swatinem/rust-cache is NOT applicable here (different workspace path). Consider `CARGO_HOME` caching manually for the generated app's registry.
**Warning signs:** "ld: No space left on device" or "link.exe: fatal error" in the e2e job cargo build step.

---

## Code Examples

### Verification Command: Confirm no native-tls/openssl-sys after migration

```bash
# Source: CLAUDE.md codec pattern + cargo-tree docs
# Must return EMPTY output after the migration
cargo tree -p ferro-cli --edges no-dev -e features \
  | grep -E 'native-tls|openssl-sys|openssl-|aws-lc-sys'
```

If this returns any output, the migration is incomplete.

### Verification Command: Confirm ring is selected (not aws-lc-rs)

```bash
# Must show ring in the ferro-cli tree
cargo tree -p ferro-cli --edges no-dev | grep "^.*ring v"
# Must show NO aws-lc-sys
cargo tree -p ferro-cli --edges no-dev | grep "aws-lc" | head -5
```

### Feature chain inspection

```bash
cargo metadata --format-version 1 \
  | python3 -c "
import json,sys
data=json.load(sys.stdin)
for p in data['packages']:
    if p['name'] == 'sqlx':
        feats = p['features']
        print('tls-rustls:', feats.get('tls-rustls'))
        print('tls-rustls-ring:', feats.get('tls-rustls-ring'))
        print('runtime-tokio-rustls:', feats.get('runtime-tokio-rustls'))
"
```

Expected output (sqlx 0.8.6):
```
tls-rustls: ['tls-rustls-ring']
tls-rustls-ring: ['tls-rustls-ring-webpki']
runtime-tokio-rustls: ['runtime-tokio', 'tls-rustls-ring']
```

### `cargo build` of generated app in e2e (environment frugality)

```bash
# In the e2e job step:
env \
  RUSTFLAGS="" \
  CARGO_PROFILE_DEV_DEBUG=false \
  CARGO_INCREMENTAL=0 \
  cargo build
```

`RUSTFLAGS=""` clears `-Dwarnings` (global CI env) so the generated app's starter warnings don't fail the build — same reasoning as `scaffold_builds_against_workspace_ferro`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| OpenSSL via native-tls | rustls + ring | 2023–2024 ecosystem shift | No system libssl-dev, cross-compile-clean |
| combined runtime+tls features in sqlx | separate runtime + tls features recommended | sqlx 0.8 | `runtime-tokio-rustls` still works but aliases to ring; explicit `tls-rustls-ring-webpki` is the forward-compatible form |
| reqwest `rustls-tls` = ring | reqwest `rustls` (0.13+) = aws-lc-rs | reqwest 0.13 restructured features | ferro is on 0.12.x where `rustls-tls` is still ring-backed; migration to 0.13 is a separate concern |
| cross/Docker for aarch64 builds | native rustup target add + gcc cross-linker | pure-Rust TLS era | Requires only apt gcc-aarch64-linux-gnu + env vars |

**Deprecated/outdated in this workspace:**
- `runtime-tokio-native-tls` on sea-orm: blocks cross-compile, breaks cold debian install, adds openssl-sys dep chain
- `tokio1-native-tls` on lettre: same issue, inconsistent with rest of workspace after D-01

---

## reqwest Coherence Scope (Research Note)

The D-01 decision says the sea-orm coherence pass is "workspace-wide" but specifically names `ferro-cli` for reqwest. However, five other crates use `reqwest = { version = "0.12", features = ["json"] }` without `default-features = false`:

- ferro-notifications: two occurrences
- ferro-ai: one occurrence (optional, behind feature)
- ferro-api-mcp: one occurrence
- ferro-mcp: one occurrence
- ferro-whatsapp: one occurrence

These all pull native-tls via reqwest defaults. They are not in `ferro-cli`'s dependency tree (the release binary), so they don't affect `cargo install ferro-cli`. However, they do affect `--all-features` CI builds and the workspace's openssl-sys exposure.

**Recommendation for planner:** Include these as a coherence sub-task in the same wave as the sea-orm sweep (D-01 intent is clear: "One TLS backend = one source of truth"). Flag as in-scope for D-01 even though not called out explicitly.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| gcc-aarch64-linux-gnu | aarch64 cross-compile | ✗ (must be installed in CI step) | apt: current | Install via `sudo apt-get install gcc-aarch64-linux-gnu` in release.yml |
| cargo install ferro-cli | e2e scheduled mode | ✗ (installed in job) | crates.io latest | N/A — this is the binary under test |
| actions/download-artifact@v4 | e2e tag-push mode | ✓ (GH native) | v4 | N/A |
| ring 0.17.14 | TLS across workspace | ✓ (already in Cargo.lock) | 0.17.14 | N/A |
| cargo-deny | deny CI job | ✓ (EmbarkStudios/cargo-deny-action@v2) | v2 | N/A |

**Missing dependencies with no fallback:** None that block execution.

**Missing dependencies with fallback:** gcc-aarch64-linux-gnu (D-04) — must be added as an apt install step.

---

## Validation Architecture

### TLS Migration Acceptance Criteria

These are the checks the planner should encode as post-implementation verification steps:

```bash
# 1. No native-tls or OpenSSL in ferro-cli dependency tree
cargo tree -p ferro-cli --edges no-dev -e features \
  | grep -E 'native-tls|openssl-sys|openssl-' | wc -l
# Expected: 0

# 2. No aws-lc-rs in ferro-cli tree (D-02 compliance)
cargo tree -p ferro-cli --edges no-dev \
  | grep -E 'aws-lc-sys|aws-lc-rs' | wc -l
# Expected: 0

# 3. ring IS present (TLS is still working)
cargo tree -p ferro-cli --edges no-dev | grep "^.*ring v"
# Expected: at least one line showing ring v0.17.x

# 4. cargo build succeeds
cargo build -p ferro-cli
# Expected: exit 0

# 5. cargo test --all-features stays green (the key workspace gate)
cargo test --all-features
# Expected: exit 0, all tests pass

# 6. cargo-deny stays green
cargo deny check
# Expected: exit 0 (ring 0.17.14 license = Apache-2.0 AND ISC, both already in allow list)
```

### E2E Test Acceptance Criteria

```bash
# From release.yml e2e job:
# 1. ferro binary runs (from artifact or cargo install)
ferro --version
# Expected: prints version, exit 0

# 2. Scaffold sequence completes without error (mirrors COMP-04)
ferro new bench-app --no-interaction --no-git
cd bench-app
ferro make:auth
ferro make:scaffold --no-smart-defaults -q -y --api Article title:string body:text
ferro make:scaffold --no-smart-defaults -q -y --api Product name:string price:float
ferro make:scaffold --no-smart-defaults -q -y --api Order status:string total:float
ferro make:scaffold --no-smart-defaults -q -y Post title:string body:text
ferro make:job EmailNotification
# Expected: all exit 0

# 3. cargo build against published ferro-rs succeeds (the COMP-04 catch)
RUSTFLAGS="" CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo build
# Expected: exit 0 (this is what was failing with 52 errors)
```

### Phase Test Map

| D-ID | Behavior | Test Type | Command | Notes |
|------|----------|-----------|---------|-------|
| D-01/D-02 | No native-tls in ferro-cli tree | Structural (cargo tree) | `cargo tree -p ferro-cli ... \| grep native-tls \| wc -l` → 0 | Run after Cargo.toml edits |
| D-01/D-02 | No aws-lc-rs in ferro-cli tree | Structural (cargo tree) | `cargo tree -p ferro-cli ... \| grep aws-lc \| wc -l` → 0 | D-02 proof |
| D-01 | Workspace compiles cleanly | Build | `cargo build --all-features` | Catches compilation regressions |
| D-01/D-05 | cargo-deny stays green | Deny check | `cargo deny check` | Ring license already in allow list |
| D-04 | aarch64 binary produced without cross | Release build | aarch64 matrix job exit 0 | Observed in CI |
| D-06/D-08 | COMP-04 scaffold sequence completes | e2e (CI) | e2e-from-release job exit 0 | continue-on-error=true initially (D-10) |
| D-09 | Existing scaffold-smoke unaffected | Existing test | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro` | Must still pass |

### Test Framework (for CI jobs)
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust stdlib test runner) + GitHub Actions |
| Quick run command | `cargo test -p ferro-cli scaffold_builds_against_workspace_ferro -- --nocapture` |
| Full suite command | `cargo test --all-features` |
| Phase gate | cargo tree grep checks + cargo test --all-features green + e2e job present in release.yml |

---

## cargo-deny Impact Analysis

**Adding ring (D-01 result):** Ring 0.17.14 license is `Apache-2.0 AND ISC`. Both are already in deny.toml's `[licenses] allow` list. No deny.toml changes needed. [VERIFIED: cargo metadata + deny.toml allow list comparison]

**Removing native-tls / openssl-sys:** These carried no known active advisories in the current deny.toml (only RUSTSEC-2026-0141 for lettre/BoringTLS boring backend, which we're not using, was already ignored). Removing them strictly reduces the advisory surface.

**Ring advisories:** RUSTSEC-2025-0009 (AES panic when overflow checking enabled) is patched in ring ≥0.17.13. The workspace is on 0.17.14. No deny.toml ignore entry needed. [VERIFIED: rustsec.org advisory database]

**RUSTSEC-2025-0007 / RUSTSEC-2025-0010:** Advisory for ring < 0.17 being unmaintained. Not applicable (we're on 0.17.14).

**Net result on deny:** The migration reduces the advisory surface (removes openssl-sys exposure), adds ring at a patched version, requires no deny.toml changes.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The from-release e2e job's `needs: build` structure (two-mode design for tag vs schedule) can be cleanly implemented in a single release.yml job with conditional steps | E2E job structure | Would require splitting into two separate workflow files or accepting that scheduled runs are not possible in the current release.yml structure |
| A2 | `gcc-aarch64-linux-gnu` on ubuntu-latest provides all the C toolchain pieces ring needs (no additional assembler beyond what gcc ships) | aarch64 cross-compile | If ring needs `binutils-aarch64-linux-gnu` for `aarch64-linux-gnu-as`, add it to the apt install step |
| A3 | `cargo install ferro-cli --locked` in the scheduled e2e mode installs the correct current version from crates.io without requiring a published Cargo.lock | E2E scheduled mode | `--locked` requires Cargo.lock in the crate; if ferro-cli is published without Cargo.lock, drop `--locked` |

**No assumptions on D-02 (ring provider) — fully verified from cargo metadata.**

---

## Open Questions

1. **reqwest coherence scope (other crates)**
   - What we know: D-01 explicitly names ferro-cli for reqwest; five other crates use reqwest without `default-features=false`.
   - What's unclear: Whether the planner should include these in the same wave (scope expansion) or note them as follow-up.
   - Recommendation: Include in the same wave; D-01 states "One TLS backend = one source of truth."

2. **`needs: build` two-mode design**
   - What we know: A job with `needs: build` is skipped when `build` doesn't run (schedule/dispatch). A job without `needs` can't download artifacts from the build job.
   - What's unclear: The exact GitHub Actions YAML pattern for a single job that conditionally depends on another — GH Actions does not support runtime-conditional `needs`.
   - Recommendation: Implement as two separate jobs: `e2e-tag` (needs: build, only on `push`) and `e2e-drift` (no needs, only on `schedule`/`workflow_dispatch`). Alternatively, accept that the scheduled drift-check installs from crates.io and runs separately; the two jobs share the same step definitions.

3. **D-10 sequencing: when does continue-on-error flip to false?**
   - What we know: The published scaffold at time of research has COMP-04 drift (52 errors).
   - What's unclear: When the template-alignment phase is planned and whether it will ship before or after Phase 225.
   - Recommendation: Ship Phase 225 with `continue-on-error: true` and add a TODO comment referencing the alignment phase. The planner should document the flip condition explicitly.

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V6 Cryptography | yes | ring 0.17.14 — no hand-rolled crypto; standard TLS |
| V2 Authentication | no (this phase is CI/build plumbing) | — |
| V5 Input Validation | no | — |

**Known Threat Patterns:**

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Supply chain: test downloads binary from untrusted source | Tampering | `cargo install --locked` pins versions; `actions/download-artifact` is within the same workflow run — no external download |
| TLS downgrade after migration | Tampering | rustls only supports TLS 1.2/1.3; no downgrade path |
| ring < 0.17.13 AES panic (RUSTSEC-2025-0009) | DoS | Workspace is on ring 0.17.14 (patched) |

---

## Sources

### Primary (HIGH confidence)
- `cargo metadata` output (live workspace) — feature chains for reqwest 0.12.28, sea-orm 1.1.19, sqlx 0.8.6, lettre 0.11.19, ring 0.17.14 [VERIFIED]
- `ferro-cli/tests/benchmark_new_project.rs` (this repo) — exact test structure and gating [VERIFIED]
- Context7 `/seanmonstar/reqwest` — reqwest rustls feature documentation, rustls-no-provider docs [CITED: context7]
- `deny.toml` (this repo) — current allowed licenses and ignored advisories [VERIFIED]
- `RUSTSEC-2025-0009` — ring AES panic, patched in ≥0.17.13 [CITED: rustsec.org/advisories/RUSTSEC-2025-0009.html]

### Secondary (MEDIUM confidence)
- `github.com/SeaQL/sea-orm/blob/master/Cargo.toml` — sea-orm feature definitions (confirmed matches cargo metadata) [CITED]
- `github.com/launchbadge/sqlx/blob/main/sqlx-core/Cargo.toml` — sqlx-core internal feature definitions [CITED]
- `briansmith/ring` issues #1789, #2131, `axodotdev/cargo-dist` issue #1378 — ring aarch64 cross-compile CC env var requirement [CITED]
- `docs.rs/crate/reqwest/0.12.15/features` — reqwest 0.12 TLS feature names confirmed [CITED]

### Tertiary (LOW confidence)
- General WebSearch results about aarch64 GH Actions patterns — used for pattern guidance only; specific env vars verified from ring issues

---

## Metadata

**Confidence breakdown:**
- TLS provider chain (D-02): HIGH — verified from live `cargo metadata`
- Workspace file list (D-01): HIGH — verified from grep of all Cargo.toml files
- aarch64 cross-compile env vars: MEDIUM — documented in ring issues/cargo-dist, but exact GH Actions ubuntu-latest behavior not integration-tested
- e2e job structure: MEDIUM — GitHub Actions conditional job mechanics well-documented; two-mode design is a known pattern

**Research date:** 2026-06-14
**Valid until:** 90 days (stable ecosystem; reqwest/sea-orm/sqlx features unlikely to change provider defaults in patch releases)
