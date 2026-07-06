---
phase: 225-release-workflow-rustls-migration-and-e2e-cli-from-release-t
reviewed: 2026-06-14T00:00:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - .github/workflows/release.yml
  - ferro-cli/Cargo.toml
  - framework/Cargo.toml
  - ferro-queue/Cargo.toml
  - ferro-mcp/Cargo.toml
  - ferro-orm/Cargo.toml
  - ferro-audit/Cargo.toml
  - ferro-migration/Cargo.toml
  - ferro-projection/Cargo.toml
  - ferro-deployments/Cargo.toml
  - ferro-reservation/Cargo.toml
  - ferro-mcp-oauth/Cargo.toml
  - ferro-mcp-server/Cargo.toml
  - app/Cargo.toml
  - ferro-notifications/Cargo.toml
  - ferro-whatsapp/Cargo.toml
  - ferro-api-mcp/Cargo.toml
  - ferro-ai/Cargo.toml
findings:
  critical: 0
  warning: 1
  info: 3
  total: 4
status: issues_found
---

# Phase 225: Code Review Report

**Reviewed:** 2026-06-14T00:00:00Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

This phase migrates all workspace crates from native-tls/OpenSSL to rustls (ring provider) and
adds e2e CI jobs to the GitHub Actions release workflow. The TLS migration is correct across all
18 reviewed files: every `reqwest` dependency carries both `default-features = false` and
`rustls-tls`; every `sea-orm` / `sea-orm-migration` dependency uses `runtime-tokio-rustls`;
`lettre` in `ferro-notifications` uses `default-features = false` + `tokio1-rustls-tls`. No
`aws-lc-rs` or `native-tls` dependency is introduced by any reviewed crate.

One warning-level issue was found in `release.yml`: the `build` and `release` jobs lack an
event-type guard, so every scheduled and manual (`workflow_dispatch`) run triggers a full
five-platform cross-compilation matrix unnecessarily, and the `release` job then fails (no tag
ref), making those workflow runs appear failed when the only intended job is `e2e-drift`.

Three info-level items are noted: a residual `native-tls` chain via `ferro-stripe` (not in scope
of this phase but present in the workspace), an unpinned action reference in the e2e jobs, and a
pre-existing `thiserror` version split between some reviewed crates.

## Warnings

### WR-01: `build` and `release` jobs run unconditionally on `schedule` and `workflow_dispatch`

**File:** `.github/workflows/release.yml:14-115`

**Issue:** The workflow has three triggers: `push` (tag-scoped to `v*`), `workflow_dispatch`, and
`schedule`. The `build` job has no `if:` condition, so it runs on all three. On `schedule` and
`workflow_dispatch` events the five-platform cross-compilation matrix executes to produce release
artifacts that no downstream job needs (the `e2e-drift` job installs `ferro-cli` from crates.io
directly, and `e2e-tag` is guarded by `if: github.event_name == 'push'`). After building, the
`release` job (also unconditional) runs and fails because `softprops/action-gh-release` requires a
tag ref; `github.ref` on schedule/manual runs is `refs/heads/master`. The failure cascades: the
workflow run shows as failed, obscuring the real signal from `e2e-drift`.

**Fix:** Add an event-type condition to both `build` and `release` (and `update-install-script`
which inherits the problem via `needs: release`):

```yaml
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    if: github.event_name == 'push'   # tag push only — schedule/dispatch use e2e-drift
    ...

  release:
    name: Create Release
    needs: build
    if: github.event_name == 'push'
    ...

  update-install-script:
    name: Update Install Script
    needs: release
    if: github.event_name == 'push'
    ...
```

This reduces weekly schedule runs to one lightweight job (`e2e-drift`) and makes
`workflow_dispatch` also run only `e2e-drift`, which is the intended behavior.

## Info

### IN-01: Residual `native-tls` chain via `ferro-stripe` (out-of-scope crate)

**File:** `ferro-stripe/Cargo.toml:13-18` (not in review scope for this phase)

**Issue:** `ferro-stripe` depends on `async-stripe = { version = "0.41", default-features = false,
features = ["runtime-tokio-hyper"] }`. The `runtime-tokio-hyper` feature selects `hyper 0.14` +
`hyper-tls`, which pulls `native-tls` and therefore `openssl` into the workspace dependency graph
(confirmed in `Cargo.lock`: `native-tls 0.2.14` -> `openssl`). The reviewed crates are fully
clean; the contamination comes only from `ferro-stripe`. The release binary (`cargo build -p
ferro-cli`) is unaffected because `ferro-cli` does not depend on `ferro-stripe`. However,
`cargo test --all-features` compiles `ferro-stripe`, so the "no external build tooling"
requirement (which native-tls/OpenSSL violates on some targets) is not met for the full test run.

**Fix:** Switch to `async-stripe`'s rustls variant in a follow-up phase:

```toml
# ferro-stripe/Cargo.toml
async-stripe = { version = "0.41", default-features = false, features = [
    "runtime-tokio-hyper-rustls",
    "billing",
    "checkout",
    "connect",
    "webhook-events",
] }
```

Verify `async-stripe 0.41` ships this feature; if not, pin to the lowest version that does.

### IN-02: `dtolnay/rust-toolchain@master` is an unpinned action reference

**File:** `.github/workflows/release.yml:153, 196`

**Issue:** The two e2e jobs (`e2e-tag`, `e2e-drift`) use `dtolnay/rust-toolchain@master` while
the `build` job uses the pinned `@stable` tag. `@master` is the tip of the default branch of that
action repository. While `dtolnay/rust-toolchain` is low-risk (it is the Rust toolchain action
maintained by a trusted Rust contributor), unpinned action refs are a best-practice violation:
a force-push or compromise of the repo could silently change CI behavior.

**Fix:** Pin to a specific SHA or the `@stable` tag for consistency:

```yaml
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: "1.88.0"
```

Using `@stable` with an explicit `toolchain:` version achieves the same pinned behavior as
`@master` without the unpinned-ref risk.

### IN-03: `thiserror` version split in two reviewed crates

**File:** `ferro-mcp-oauth/Cargo.toml:24`, `ferro-mcp-server/Cargo.toml:28`

**Issue:** `ferro-mcp-oauth` pins `thiserror = "1"` and `ferro-mcp-server` pins `thiserror =
"1.0"`, while the rest of the workspace (16 of the 18 reviewed crates, plus additional workspace
members) use `thiserror = "2"`. Cargo resolves these as separate semver-incompatible crates, so
both `thiserror 1.x` and `thiserror 2.x` compile in the same workspace. This is not a bug but
doubles the compile cost for these crates and signals that `ferro-mcp-oauth` and `ferro-mcp-server`
have not been updated to align with the workspace standard. This is a pre-existing condition not
introduced by this phase.

**Fix:** Bump both to `thiserror = "2"` in a follow-up cleanup:

```toml
# ferro-mcp-oauth/Cargo.toml
thiserror = "2"

# ferro-mcp-server/Cargo.toml
thiserror = "2"
```

Check for any API differences between `thiserror 1` and `thiserror 2` before bumping
(`thiserror 2` changed the `#[error(transparent)]` behavior slightly).

---

_Reviewed: 2026-06-14T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
