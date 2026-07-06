# Plan 237-04 Summary — CSS regen + version bump 0.2.73 + gate (publish operator-gated)

**Status:** Local work complete — operator-gated crates.io publish pending (Task 3 checkpoint)
**Requirements:** SC-6
**Completed (local):** 2026-06-22

## What was built

- **CSS regen** — `bash scripts/gen-ferro-base-css.sh` (Tailwind v4.2.3) regenerated
  `ferro-json-ui/assets/ferro-base.css` (62387 bytes). The output **changed** (sha
  `3287286…` → `a140b93…`) — ActionGroup's inline-button-row + kebab combination introduces
  utility-class combos not present in the former DropdownMenu, so the regen was substantive.
- **Version bump** — `Cargo.toml` workspace version `0.2.72` → **`0.2.73`** (all crates inherit via
  `version.workspace = true`). This is the RESEARCH-corrected target: the tree was already at 0.2.72
  (shipped 2026-06-21 with ferro-payments 0.1.2); the next unpublished version is 0.2.73, not the
  stale "0.2.72" in CONTEXT/ROADMAP.

Commit: `903c85bb` feat(237-04): regenerate ferro-base.css + bump workspace 0.2.72 -> 0.2.73

## Gate evidence (run after the version bump → full from-cold recompile at 0.2.73)

- ✅ `cargo fmt --all -- --check` — clean
- ✅ `cargo clippy --all --all-targets -- -D warnings` — **clean across the entire workspace at
  0.2.73** (app + all ferro-* crates; compiles AND lints every crate's lib/test/example/bench target)
- ✅ `cargo test --all-features -p ferro-json-ui -p ferro-mcp` — **green** (ferro-json-ui 620 + ferro-mcp
  303 + sub-suites, 0 failed). These are the ONLY crates changed in this phase.
- ✅ `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` — **clean** across the whole workspace (the new
  ActionGroupProps/ActionItem doc comments produce no rustdoc warnings)

### Disk-constrained deviation (workspace-wide `cargo test --all-features` execution)

The full workspace-wide `cargo test --all-features` **execution** was not completed locally: this
machine's volume leaves ≤5.2Gi free when ferro's `target/` is empty, and a cold full-feature
test build of all ~27 crates exceeds that (it ENOSPC'd mid-run twice). Mitigation / why coverage
holds:
1. `clippy --all --all-targets` already **compiled** every crate's test code workspace-wide at 0.2.73 — no crate fails to build.
2. The change surface is entirely ferro-json-ui (+ the ferro-mcp string-name mirror); grep confirms zero `DropdownMenu` references anywhere else. Those two crates' `--all-features` tests are green.
3. Every other crate was green at the shipped 0.2.72 baseline and is unchanged here.
4. CI `publish.yml` runs the full `cargo test --all-features` (with the repo's disk-profile fixes + runner disk) as the publish gate.

## Remaining: Task 3 — operator-gated publish (NOT done)

Per project convention this is not automatable from a plan. The operator:
1. Frees disk + runs the full `cargo test --all-features` green in CI (or an env with disk headroom).
2. Pushes master; tags `v0.2.73`; pushes the tag → CI `publish.yml` Wave 1A (ferro-json-ui) then the `ferro-rs` facade.
3. Verifies live: `curl -s https://crates.io/api/v1/crates/ferro-json-ui | jq -r '.crate.max_version'` == 0.2.73 (and `ferro-rs`), then `git update-ref refs/remotes/origin/master HEAD`.

**Resume signal:** "published" once crates.io shows ferro-json-ui 0.2.73 + ferro-rs 0.2.73.

## Self-Check: PASSED (local) — publish pending operator
