# Phase 226: Homebrew Tap Distribution for ferro-cli - Context

**Gathered:** 2026-06-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Stand up a Homebrew tap so a new user can `brew install` the `ferro` CLI and run `ferro new`
with **no Rust toolchain, no `curl | sh`, no manual PATH**. This is a NEW, additive
distribution channel — the existing `cargo install ferro-cli` (crates.io) and
`scripts/install.sh` (`curl … | sh`) paths are unchanged.

**Killer feature:** the day-one experience — `brew install albertogferrario/ferro/ferro && ferro new myapp`
with zero prerequisites. This is the lowest-friction on-ramp for the Mac/Linux dev audience
and directly serves the PROJECT.md v1.0 "day-one experience" criterion.

Composes with Phase 225: `release.yml` already builds the per-arch tarballs
(`ferro-<tag>-<target>.tar.gz`, macOS arm64/x86_64 + Linux x86_64/aarch64), and the rustls
migration removed the openssl/pkg-config build dependency (so a source fallback, if ever added,
is clean).

**NOT in scope:** homebrew-core submission (deferred to post-1.0); Windows package managers
(scoop/winget); changing the existing cargo/curl install paths.

</domain>

<decisions>
## Implementation Decisions

### Tap & formula
- **D-01:** Own tap — a separate public repo `albertogferrario/homebrew-ferro` →
  `brew install albertogferrario/ferro/ferro`. NOT homebrew-core (avoids the pre-1.0
  notability/review gate and a likely `ferro` formula-name collision). Can graduate to core post-1.0.
- **D-02:** **Binary formula** off the GitHub release tarballs — covering **macOS arm64 + x86_64 AND
  Linux x86_64 + aarch64** (Homebrew-on-Linux works), per-arch `url` + `sha256`, no user-side compile.
  No source fallback in v1 (deferred; rustls makes it cheap to add later if an unsupported arch appears).
  The formula uses `on_macos`/`on_linux` + `Hardware::CPU.arm?`/Intel branches to pick the right tarball.
  Binary name is `ferro` (crate `ferro-cli`, published on crates.io as `ferro-rs`).

### Automation
- **D-03:** Auto-bump via a **maintained action** (e.g. `mislav/bump-homebrew-formula-action`),
  **wired into the existing release/publish automation** ("same as publish" — not a manual
  side-process). It fires on the real-release event (the same moment `release.yml` produces the
  tarballs / the crates.io publish happens), recomputes the per-arch SHA256s, and updates
  `Formula/ferro.rb` in the tap. **Gated to non-prerelease tags** so betas don't bump the formula.
- **D-04:** `release.yml` pushes the bump to the separate tap repo using a **fine-grained PAT**
  stored as secret `HOMEBREW_TAP_TOKEN`, scoped to **ONLY `homebrew-ferro`** with `contents:write`
  (least privilege). Operator creates the token + adds the secret.

### Quality gates
- **D-05:** Formula carries a `test do` block (`ferro --version`, plus a quick `ferro new` smoke in a
  temp dir) AND CI runs `brew audit --strict --online` (and, where practical, an install/test-bot pass)
  so a broken formula is caught before users hit it — mirrors Phase 225's e2e/detection philosophy.

### Docs
- **D-06:** Surface `brew install` in the install docs/README as the recommended new-user path,
  alongside (not replacing) the existing `cargo install` and `curl | sh` instructions.

### Claude's Discretion
- Exact `Formula/ferro.rb` Ruby structure (on_macos/on_linux + CPU-arch branches, `bin.install`).
- Whether to vendor shell completions / manpage in the formula if the CLI can emit them.
- Cron/test-bot specifics and the exact audit invocation.
- Whether the bump action opens a PR vs a direct commit to the tap (PR is safer; direct is simpler).

### Operator Actions (human — cannot be automated by the executor)
- Create the public `albertogferrario/homebrew-ferro` repo (with a `Formula/` dir; a README is nice-to-have).
- Create the fine-grained PAT (contents:write on `homebrew-ferro` only) and add the `HOMEBREW_TAP_TOKEN`
  secret to the `ferro` repo.
- Pushing the `release.yml` edits requires the `workflow` gh-token scope (`gh auth refresh -s workflow`
  or SSH/PAT) — same note as Phase 225.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Release / publish automation (the integration points)
- `.github/workflows/release.yml` — produces the per-arch tarballs the formula points at; gains the
  auto-bump step. NOTE: Phase 225 just added `e2e-tag`/`e2e-drift` jobs and `if: github.event_name == 'push'`
  guards on `build`/`release`/`update-install-script` — the bump job must slot in consistently.
- `.github/workflows/publish.yml` — the crates.io publish flow; reference for the "same as publish"
  cadence/trigger the user wants the formula bump aligned with.
- `scripts/install.sh` — existing `curl | sh` installer (`REPO="albertogferrario/ferro"`, `BINARY_NAME="ferro"`,
  installs to `~/.ferro/bin`); docs must stay consistent with it.

### Artifact shape
- Release tarball naming: `ferro-<tag>-<target>.tar.gz`, targets:
  `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
  (windows is `.zip`, not used by brew). See `release.yml` build matrix + `225-*-SUMMARY.md`.
- `ferro-cli/Cargo.toml` — binary `ferro`, crate `ferro-cli`, published as `ferro-rs`.

### External docs (researcher to fetch CURRENT versions)
- Homebrew Formula Cookbook + "Bottles"/binary-formula guidance + `on_macos`/`on_linux`/`Hardware::CPU` API.
- `mislav/bump-homebrew-formula-action` README (inputs, token, PR-vs-commit, formula path).
- `brew audit --strict --online` and `brew test-bot` usage for tap CI.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `release.yml` already emits exactly the per-arch tarballs a binary formula needs — the formula's
  `url`s point straight at the GitHub release assets; no new build work.
- `scripts/install.sh` already does OS/arch detection mapping to those same tarballs — its mapping
  logic is the reference for which tarball each Homebrew platform branch should use.

### Established Patterns
- CI workflows pin toolchain `1.88.0`, use `dtolnay/rust-toolchain@master`; secrets via `${{ secrets.* }}`.
- Phase 225 established the convention of guarding tag-only jobs with `if: github.event_name == 'push'`
  and gating drift jobs to schedule/dispatch — the bump job should follow the same event-guard discipline.

### Integration Points
- A new job/step in `release.yml` (the bump), gated to real releases, using `HOMEBREW_TAP_TOKEN`.
- The separate `homebrew-ferro` repo (`Formula/ferro.rb` + optional tap CI for audit/test).
- Install docs/README.

### Constraints
- Project-agnostic-crate rule does NOT apply to repo-level CI/docs, but the tap repo and formula DO
  hardcode app identity (`albertogferrario/ferro`, `ferro`) — that is correct here (this is the app's
  own distribution, not a reusable `ferro-*` library crate).
- `cargo test --all-features` disk-fragility is irrelevant to this phase (no workspace build changes).

</code_context>

<specifics>
## Specific Ideas

- The measurable win: a brand-new user on a clean Mac with no Rust installed runs
  `brew install albertogferrario/ferro/ferro && ferro new myapp` and gets a working scaffold.
  The formula `test do` smoke (`ferro --version` + `ferro new`) is the structural guarantee that path works.
- Align the bump trigger with the existing publish flow so "release → crates.io + GitHub binaries + brew formula"
  all happen from one tag, no separate manual step.

</specifics>

<deferred>
## Deferred Ideas

- **homebrew-core submission** — post-1.0, once notability criteria are comfortably met.
- **Source-fallback formula** (`depends_on "rust" => :build`) — add only if an unsupported arch is requested;
  rustls already makes it a clean build.
- **Windows package managers** (scoop/winget) — separate distribution effort.

</deferred>

<deviation>
## Execution Deviation — token-free pivot (2026-06-14)

During execution the operator asked to eliminate the PAT entirely. **D-03/D-04 revised:**
the formula is no longer bumped by a PAT-authenticated push *from* `release.yml` *into* the
tap (cross-repo write → required `HOMEBREW_TAP_TOKEN`). Instead the **tap updates itself**:
`homebrew-ferro/.github/workflows/update-formula.yml` runs on a 6-hourly `schedule` +
`workflow_dispatch`, reads ferro's **public** releases, renders `Formula/ferro.rb` from
`Formula/ferro.rb.tpl` via `bin/update-formula.sh`, and commits to its **own** repo using the
built-in `GITHUB_TOKEN` (`permissions: contents: write`). **No PAT, no secret, no cross-repo
credential anywhere** — strictly less privilege. Tradeoff: the bump lands on the next poll tick
(or instantly via "Run workflow") rather than the exact second of release — irrelevant for `brew`.

Consequences:
- Removed from the ferro repo: the `bump-homebrew-formula` job in `release.yml`, `scripts/bump-homebrew-formula.sh`,
  and the `homebrew/` staging dir — all relocated into the self-contained tap (single source of truth).
- D-01 (tap repo) and the seeding are DONE: `albertogferrario/homebrew-ferro` created + seeded
  (`Formula/ferro.rb` + `.tpl`, `bin/update-formula.sh`, `.github/workflows/{tests,update-formula}.yml`);
  the update workflow runs green ("no published release yet — nothing to do").
- D-06 docs unchanged and still correct (`brew install albertogferrario/ferro/ferro`).
- The ONLY remaining step to go live: ferro must publish its first release (push+tag → release.yml
  builds the 4 tarballs); the tap then auto-bumps to real checksums and `brew install` works.
</deviation>

---

*Phase: 226-homebrew-tap-distribution-for-ferro-cli*
*Context gathered: 2026-06-14*
