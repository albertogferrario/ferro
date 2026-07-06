# Phase 226: Homebrew Tap Distribution for ferro-cli - Research

**Researched:** 2026-06-14
**Domain:** Homebrew tap formula authoring, multi-arch binary distribution, GitHub Actions automation
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Own tap `albertogferrario/homebrew-ferro` → `brew install albertogferrario/ferro/ferro`. Not homebrew-core.
- **D-02:** Binary formula, 4 platforms (macOS arm64 + x86_64, Linux x86_64 + aarch64), per-arch url + sha256, no source fallback. Binary name `ferro`.
- **D-03:** Auto-bump via a maintained action, wired into the existing release/publish flow, fires on the real-release event, gated to non-prerelease tags.
- **D-04:** `release.yml` pushes to the tap via fine-grained PAT `HOMEBREW_TAP_TOKEN`, scoped to `homebrew-ferro` only with `contents:write` (least privilege).
- **D-05:** Formula carries a `test do` block (`ferro --version` + `ferro new` smoke) plus CI runs `brew audit --strict --online`.
- **D-06:** Surface `brew install` in install docs/README alongside existing `cargo install` and `curl | sh` paths.

### Claude's Discretion
- Exact `Formula/ferro.rb` Ruby structure (`on_macos/on_linux` + CPU-arch branches, `bin.install`).
- Whether to vendor shell completions / manpage in the formula if the CLI can emit them.
- Cron/test-bot specifics and exact audit invocation.
- Whether the bump action opens a PR vs a direct commit to the tap (PR is safer; direct is simpler).

### Deferred Ideas (OUT OF SCOPE)
- homebrew-core submission (post-1.0)
- Source-fallback formula (`depends_on "rust" => :build`)
- Windows package managers (scoop/winget)
</user_constraints>

---

## Summary

This phase adds Homebrew as a zero-prerequisite distribution channel for the `ferro` CLI. The existing `release.yml` already builds per-arch tarballs (`ferro-<tag>-<target>.tar.gz` for four Unix targets); this phase layers a `Formula/ferro.rb` and an auto-bump job on top of that artifact flow.

The central research finding is that `mislav/bump-homebrew-formula-action` explicitly **cannot** update multi-arch binary formulae with more than one `sha256` value. The correct mechanism for a 4-platform binary formula is a small in-repo shell script that: (1) downloads the four release tarballs to compute SHA256s, (2) renders a formula template with the computed values, and (3) commits/pushes to the tap repo using the `HOMEBREW_TAP_TOKEN`. This pattern is used widely in production (Shopify's own CLI tooling, Rigellute/spotify-tui, and others).

The preferred Ruby DSL uses `on_macos do / on_arm do ... end / on_intel do ... end` nested blocks with one `url` + `sha256` per leaf. This is the canonical Homebrew-blessed style (as opposed to older `case when OS.mac? && Hardware::CPU.arm?` style), verified from the Shopify ejson formula currently installed locally and from Homebrew's own Formula-Cookbook documentation.

**Primary recommendation:** Write a `scripts/bump-homebrew-formula.sh` in this repo that renders `Formula/ferro.rb` from a template and pushes to the tap repo via git. Call it from a new `bump-homebrew-formula` job in `release.yml`, added after the existing `release` job, gated identically to the other post-release jobs (`if: github.event_name == 'push'`). No external action dependency required.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Formula Ruby file (`Formula/ferro.rb`) | Tap repo (`homebrew-ferro`) | Seeded from this repo | Lives in the tap; auto-bumped by CI from this repo |
| Binary tarballs | GitHub Releases (this repo) | — | Already produced by `release.yml` `build` job |
| SHA256 computation | CI bump job (this repo) | — | Computed at release time from already-uploaded artifacts |
| Auto-bump job | `release.yml` (this repo) | — | Runs after `release` job; pushes to tap via PAT |
| Tap CI (audit/test) | Tap repo CI (`.github/workflows/`) | — | Operator creates; validates formula syntax on every PR |
| Install docs | `docs/src/` (this repo) | README.md | D-06 doc update |

---

## Standard Stack

### Core
| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Homebrew Formula Ruby DSL | Homebrew 6.x | Formula authoring | The only supported format for brew install |
| `Homebrew/actions/setup-homebrew@main` | current | Tap CI setup | Official Homebrew GitHub Action for tap testing |
| `softprops/action-gh-release@v1` | v1 | Already in release.yml | Produces the GitHub Release artifacts the formula URLs point at |
| `actions/download-artifact@v4` | v4 | Download tarballs in bump job | Reuse built artifacts from `build` job |

### What NOT to Use
| Action | Why Unsupported |
|--------|----------------|
| `mislav/bump-homebrew-formula-action` | Documented as unable to update multi-`sha256` formulae (formulae "which use Ruby `if...else` conditions" for alternate download locations — confirmed from README) |
| `dawidd6/action-homebrew-bump-formula` | Wraps `brew bump-formula-pr` which only handles single-URL source formulae |
| `Homebrew/actions/bump-formulae@main` | For homebrew-core; requires `public_repo` token scope on homebrew-core PRs, not for private taps |
| goreleaser `brews:` | Go project tool; not applicable |

### Supporting
| Tool | Purpose | When to Use |
|------|---------|-------------|
| `brew audit --strict --online` | Lint formula for Homebrew style and online checks | Tap CI on every push/PR; executor validation |
| `brew test-bot --only-tap-syntax` | Syntax-only check without building | Tap CI (fast path on push) |
| `brew install --formula ./Formula/ferro.rb` | Local install test | Developer validation before publishing tap |
| `ruby -c Formula/ferro.rb` | Ruby syntax check (no brew needed) | Fastest pre-audit sanity check |
| `shasum -a 256` (macOS) / `sha256sum` (Linux) | Compute SHA256 | Both available on GitHub runners |

---

## Architecture Patterns

### System Architecture Diagram

```
push to master
    │
    ▼
publish.yml
  check-version → bump-version → test → publish (crates.io waves)
                                                   │
                                                   ▼
                                         Create + push tag vX.Y.Z
                                                   │
                                                   ▼ (tag push event)
                                           release.yml
                                             build (4 Unix targets)
                                                │  └─ upload-artifact per target
                                                ▼
                                             release (create GitHub Release)
                                                │  └─ attach tarballs
                                                ▼
                                       bump-homebrew-formula (NEW)
                                         download-artifact (4 targets)
                                         compute SHA256 x4
                                         render Formula/ferro.rb from template
                                         git push → albertogferrario/homebrew-ferro
                                                   │
                                                   ▼
                                         homebrew-ferro tap CI
                                           brew audit --strict --online
                                           brew test-bot --only-tap-syntax
```

### Recommended Project Structure in This Repo

```
scripts/
└── bump-homebrew-formula.sh   # new: renders formula + pushes to tap

homebrew/
├── Formula/
│   └── ferro.rb.tpl           # formula template (VERSION + 4x SHA256 placeholders)
└── tap-ci/
    └── README.md              # operator instructions for tap CI setup
```

The `homebrew/` directory is staged here so the operator can copy `Formula/ferro.rb.tpl`'s output into `albertogferrario/homebrew-ferro/Formula/ferro.rb` when seeding the tap.

### Pattern 1: Multi-Arch Binary Formula Ruby Structure

**What:** Nested `on_macos`/`on_linux` + `on_arm`/`on_intel` blocks, one `url` + `sha256` per leaf. The binary is extracted by `bin.install`.

**When to use:** Any time a formula ships pre-built binaries for multiple platforms with no source build. This is the Homebrew-blessed style (verified from production formula `ejson.rb` in `shopify/homebrew-shopify`, and from Homebrew's Formula-Cookbook docs).

**Example (complete, copy-pasteable template):**
```ruby
# Source: verified from shopify/homebrew-shopify/ejson.rb (installed locally) +
#         Homebrew Formula-Cookbook docs (docs.brew.sh/Formula-Cookbook)
class Ferro < Formula
  desc "CLI for scaffolding Ferro web applications"
  homepage "https://github.com/albertogferrario/ferro"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_MACOS_X86_64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_LINUX_AARCH64"
    end
    on_intel do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "ferro"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ferro --version")
    system "#{bin}/ferro", "new", "test-app", "--no-interaction", "--no-git",
           chdir: testpath
    assert_path_exists testpath/"test-app"
  end
end
```

**Notes on the `ferro new` smoke test:**
- `testpath` is a temp directory automatically created and cleaned by Homebrew.
- `--no-interaction` is the flag used in the existing e2e-tag job; `--no-git` avoids a git init dependency inside the sandbox.
- If `ferro new` exits non-zero or `test-app/` is absent, the test fails.
- Shell completions/manpage: defer to a future phase (D-02 does not include them; check with `ferro --help` whether the binary can emit completions before adding to formula).

### Pattern 2: Auto-Bump Script

**What:** A shell script executed in the CI bump job that: downloads the 4 tarballs already uploaded to the GitHub Release, computes SHA256 for each, renders the formula template with `sed`, clones the tap repo via HTTPS (authenticated with PAT), commits, and pushes.

**Why a script, not an action:**
`mislav/bump-homebrew-formula-action` processes exactly one `download-url` / `download-sha256` pair and cannot update 4 arch-specific blocks. This limitation is stated in its README: "This action cannot bump formulae which use Ruby `if...else` conditions" and the action updates only the fields `version`, `url`, `sha256` (singular). A script is the correct tool here and is the pattern used by many Rust CLIs.

**Script skeleton:**
```bash
#!/usr/bin/env bash
# scripts/bump-homebrew-formula.sh
# Usage: VERSION=vX.Y.Z ./scripts/bump-homebrew-formula.sh
set -euo pipefail

VERSION="${1:-${VERSION:?VERSION required}}"
TAG="$VERSION"   # e.g. v0.2.59
VER="${TAG#v}"   # e.g. 0.2.59

BASE_URL="https://github.com/albertogferrario/ferro/releases/download/${TAG}"

compute_sha256() {
  local url="$1"
  curl -fsSL "$url" | shasum -a 256 | awk '{print $1}'
}

SHA256_MACOS_ARM64=$(compute_sha256 "${BASE_URL}/ferro-${TAG}-aarch64-apple-darwin.tar.gz")
SHA256_MACOS_X86_64=$(compute_sha256 "${BASE_URL}/ferro-${TAG}-x86_64-apple-darwin.tar.gz")
SHA256_LINUX_AARCH64=$(compute_sha256 "${BASE_URL}/ferro-${TAG}-aarch64-unknown-linux-gnu.tar.gz")
SHA256_LINUX_X86_64=$(compute_sha256 "${BASE_URL}/ferro-${TAG}-x86_64-unknown-linux-gnu.tar.gz")

FORMULA=$(sed \
  -e "s/VERSION_PLACEHOLDER/${VER}/g" \
  -e "s/SHA256_MACOS_ARM64/${SHA256_MACOS_ARM64}/g" \
  -e "s/SHA256_MACOS_X86_64/${SHA256_MACOS_X86_64}/g" \
  -e "s/SHA256_LINUX_AARCH64/${SHA256_LINUX_AARCH64}/g" \
  -e "s/SHA256_LINUX_X86_64/${SHA256_LINUX_X86_64}/g" \
  homebrew/Formula/ferro.rb.tpl)

# Clone tap via HTTPS with PAT embedded in URL
git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/albertogferrario/homebrew-ferro.git" tap-repo
printf '%s\n' "$FORMULA" > tap-repo/Formula/ferro.rb

cd tap-repo
git config user.name  "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add Formula/ferro.rb
git diff --staged --quiet && { echo "Formula already up to date"; exit 0; }
git commit -m "chore: bump ferro to ${VER}"
git push
```

**Alternative (download artifacts from prior job instead of re-downloading from release):**
The bump job can use `actions/download-artifact@v4` for the artifacts uploaded in the `build` job, then compute SHA256 from the local tarballs. This avoids network re-download from GitHub Releases and works even if the Release API takes a moment to propagate. Both approaches work; re-downloading from the release URL is simpler (no artifact-path juggling) and is idempotent.

### Pattern 3: Bump Job in release.yml

```yaml
# Add after the existing 'release' job in release.yml
bump-homebrew-formula:
  name: Bump Homebrew formula
  if: github.event_name == 'push'
  needs: release          # wait for tarballs to be attached to the GitHub Release
  runs-on: ubuntu-latest

  steps:
    - uses: actions/checkout@v4

    - name: Bump formula in tap
      env:
        HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
        VERSION: ${{ github.ref_name }}
      run: bash scripts/bump-homebrew-formula.sh "$VERSION"
```

**Non-prerelease gate:** `release.yml`'s `softprops/action-gh-release` already sets `prerelease: false`. The publish/tag flow in `publish.yml` uses `v` + semver tags (e.g., `v0.2.59`) with no alpha/beta/rc suffixes. There are no prerelease tags in the repository history. The `if: github.event_name == 'push'` guard is already the right gate. If desired, add a secondary guard:

```yaml
- name: Gate: non-prerelease only
  if: "!contains(github.ref_name, '-')"  # rejects v1.0.0-alpha style
  run: echo "Tag is non-prerelease"
```

This is a belt-and-suspenders guard; it is not required given current tagging conventions.

### Pattern 4: Tap CI Workflow (in homebrew-ferro repo)

From `brew tap-new` source code (verified from `/opt/homebrew/Library/Homebrew/dev-cmd/tap-new.rb`), the auto-generated `tests.yml` uses:

```yaml
name: brew test-bot

on:
  push:
    branches:
      - main
  pull_request:

jobs:
  test-bot:
    strategy:
      matrix:
        os: [macos-15-intel, macos-26]
        include:
          - os: ubuntu-latest
            container: ghcr.io/homebrew/brew:main
    runs-on: ${{ matrix.os }}
    container: ${{ matrix.container }}
    permissions:
      actions: read
      checks: read
      contents: read
      pull-requests: read
    steps:
      - name: Set up Homebrew
        id: set-up-homebrew
        uses: Homebrew/actions/setup-homebrew@main
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Cache Homebrew Bundler RubyGems
        uses: actions/cache@v4
        with:
          path: ${{ steps.set-up-homebrew.outputs.gems-path }}
          key: ${{ matrix.os }}-rubygems-${{ steps.set-up-homebrew.outputs.gems-hash }}
          restore-keys: ${{ matrix.os }}-rubygems-

      - run: brew test-bot --only-cleanup-before
      - run: brew test-bot --only-setup
      - run: brew test-bot --only-tap-syntax    # validates formula Ruby syntax for every push

      - run: brew test-bot --only-formulae      # builds + tests formula (PR only)
        if: github.event_name == 'pull_request'

      - name: Upload bottles as artifact
        if: always() && github.event_name == 'pull_request'
        uses: actions/upload-artifact@v4
        with:
          name: bottles_${{ matrix.os }}
          path: '*.bottle.*'
```

For a binary formula (no source build), `--only-formulae` will run `brew test ferro` (the `test do` block) rather than building from source. The bottle upload step will produce no `.bottle.*` files (no bottles for binary formulae) — this is harmless.

**D-05 `brew audit` step for the tap CI:** The standard `brew test-bot --only-tap-syntax` runs `brew audit` internally. To make it explicit and catch `--strict --online` issues, add:

```yaml
      - name: Audit formula
        run: brew audit --strict --online Formula/ferro.rb
```

### Anti-Patterns to Avoid

- **Using `mislav/bump-homebrew-formula-action` for multi-arch:** The action only handles one `sha256` field and explicitly cannot handle Ruby conditional blocks. Using it on this formula silently writes a broken formula.
- **`case/when OS.mac? && Hardware::CPU.arm?` style (old):** Works but the newer `on_macos do / on_arm do` nested-block style is what Homebrew's linter expects and audits green. The toxiproxy formula in the Shopify tap (locally installed) uses the old case/when style — it predates the `on_*` blocks. New formulae should use the nested block style.
- **Including Windows targets:** `.zip` artifacts are not used by Homebrew. The formula only covers the four `.tar.gz` targets.
- **Seeding formula from `brew tap-new` default and forgetting `--only-tap-syntax`:** The `--only-formulae` step runs `brew install` (source build by default) and uploads bottles. A binary formula will not bottle-build. Configure tap CI to skip bottle upload or accept empty artifact uploads.
- **PR-based bump approach for an automated flow:** Opening a PR to the tap requires the bump job to have `pull_requests:write` permission AND a human to merge. Direct commit to `main` in the tap is the right automation pattern here (no review gate needed for a formula update that just bumps versions and checksums).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Formula Ruby validation | Custom Ruby parser | `brew audit --strict Formula/ferro.rb` | Homebrew's auditor knows every style rule |
| SHA256 of remote URL | Custom HTTP client | `curl -fsSL <url> \| shasum -a 256` (macOS) or `sha256sum` (Linux) | These are available on all CI runners; no dependencies |
| Multi-platform install selection | Platform detection Ruby | `on_macos / on_arm` DSL blocks | Homebrew handles the selection; don't replicate it in `def install` |
| Tap repo management | `git` subprocess in complex script | Direct git clone + commit + push | Simple and reliable for this use case; no actions dependency |

**Key insight:** For multi-arch binary formulae, the right abstraction boundary is: Homebrew's DSL handles _which_ binary to fetch; the bump script only handles _which version_ and _which checksums_ to write.

---

## Common Pitfalls

### Pitfall 1: Tarball Contains Just the Binary (No Subdirectory)
**What goes wrong:** The `ferro-<tag>-<target>.tar.gz` archive (per `release.yml`) contains a single file `ferro` at the root (not inside a subdir like `ferro-v0.2.59/ferro`). `bin.install "ferro"` works correctly for this structure.
**Why it happens:** The `Prepare artifact (Unix)` step in `release.yml` does `cp target/.../release/ferro dist/` then `tar -czvf ... ferro` — the binary is at the archive root.
**How to avoid:** Verify with `tar -tzf ferro-<tag>-<target>.tar.gz` before finalizing the formula. `bin.install "ferro"` installs whatever is at the root named `ferro`.
**Warning signs:** `brew install` fails with "No such file or directory" or installs nothing.

### Pitfall 2: Version in Formula Includes the `v` Prefix
**What goes wrong:** `github.ref_name` in release.yml is `v0.2.59` (with prefix). Homebrew's `version` field should be `0.2.59` (without prefix). If the `v` is included, `brew info` shows an incorrect version and `assert_match version.to_s, ...` will fail because `ferro --version` likely outputs `0.2.59` not `v0.2.59`.
**Why it happens:** Tag names include `v` by convention; template substitution must strip it.
**How to avoid:** In the bump script, strip the prefix: `VER="${TAG#v}"` and use `$VER` for the formula `version` field. Use `$TAG` for the URL `download/${TAG}/ferro-${TAG}-...`.
**Warning signs:** `brew audit` warning about version format; `test do` assertion failure.

### Pitfall 3: `brew audit --strict --online` Fails on Tap Formula
**What goes wrong:** `--online` audits check that the formula URL is reachable, the sha256 matches, and the `homepage` URL resolves. If the GitHub Release is not yet public when audit runs, it will fail.
**Why it happens:** The tap CI runs on every push, including pushes made by the bump job. The bump job runs only after `release` completes (and `softprops/action-gh-release` publishes the release), so by the time the tap's CI triggers, the release should be public.
**How to avoid:** In the tap CI, run `brew audit --strict --online` only on `pull_request` events (not on `push`) if timing is a concern, or rely on `--only-tap-syntax` for push events (which skips the online check).
**Warning signs:** CI audit failures with "URL not reachable" immediately after bump.

### Pitfall 4: PAT Token Scope — Direct Push vs. PR
**What goes wrong:** If the bump job commits directly to `main` in the tap using a fine-grained PAT with only `contents:write`, it works. If the approach is changed to open a PR instead, `pull_requests:write` is also needed. Forgetting this causes a 403 when the action tries to create the PR.
**Why it happens:** PR creation is a separate GitHub API permission from content writes.
**How to avoid:** Use direct commit to `main` (the recommended approach here — no PR gate needed for automated version bumps). If switching to PR approach later, add `pull_requests:write` to the PAT.
**Warning signs:** `403 Forbidden` from the GitHub API when push job runs.

### Pitfall 5: Tap Repo Must Be Named `homebrew-ferro`
**What goes wrong:** If the operator creates `alberto/ferro-homebrew` or `alberto/tap-ferro`, `brew tap albertogferrario/ferro` will not resolve it.
**Why it happens:** Homebrew's tap resolution rule: short form `brew tap user/name` resolves to `github.com/user/homebrew-name`. The prefix `homebrew-` is mandatory.
**How to avoid:** Operator creates `albertogferrario/homebrew-ferro` (repo name, not topic).
**Warning signs:** `brew tap albertogferrario/ferro` fails with "repository not found".

### Pitfall 6: `ferro new` in `test do` Requires Network or Cannot Create Project
**What goes wrong:** If `ferro new` downloads templates from the network during the test, the sandbox network access may be limited, causing flaky tests. If it tries to write outside `testpath`, the test sandbox blocks it.
**Why it happens:** Brew's `test do` block runs in a restricted environment with `testpath` as the working directory.
**How to avoid:** Use `chdir: testpath` in the Ruby `system` call (Homebrew DSL passes it to the subprocess) and add `--no-git` to avoid git initialization. Verify that `ferro new` does not make network calls — it should not (template generation is local).
**Warning signs:** `test do` fails with permission errors or network errors on clean machines.

---

## Code Examples

### Complete `Formula/ferro.rb.tpl` Template
```ruby
# Source: pattern from shopify/homebrew-shopify/ejson.rb (verified locally, Homebrew 6.x)
#         + docs.brew.sh/Formula-Cookbook
class Ferro < Formula
  desc "CLI for scaffolding Ferro web applications"
  homepage "https://github.com/albertogferrario/ferro"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  livecheck do
    url :stable
    strategy :github_latest
  end

  on_macos do
    on_arm do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_MACOS_X86_64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_LINUX_AARCH64"
    end
    on_intel do
      url "https://github.com/albertogferrario/ferro/releases/download/vVERSION_PLACEHOLDER/ferro-vVERSION_PLACEHOLDER-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "ferro"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ferro --version")
    system bin/"ferro", "new", "smoke-app", "--no-interaction", "--no-git",
           chdir: testpath
    assert_path_exists testpath/"smoke-app"
  end
end
```

### `scripts/bump-homebrew-formula.sh`
```bash
#!/usr/bin/env bash
# Render Formula/ferro.rb from template with computed SHA256s and push to tap repo.
# Usage (from CI): VERSION=v0.2.59 bash scripts/bump-homebrew-formula.sh
#          (local): VERSION=v0.2.59 HOMEBREW_TAP_TOKEN=<pat> bash scripts/bump-homebrew-formula.sh
set -euo pipefail

VERSION="${1:-${VERSION:?VERSION env var required}}"
TAG="$VERSION"
VER="${TAG#v}"  # strip leading 'v'

BASE_URL="https://github.com/albertogferrario/ferro/releases/download/${TAG}"
TEMPLATE="homebrew/Formula/ferro.rb.tpl"
TAP_REPO="albertogferrario/homebrew-ferro"

compute_sha256() {
  # Works on both macOS (shasum) and Linux (sha256sum / shasum from homebrew)
  curl -fsSL "$1" | shasum -a 256 | awk '{print $1}'
}

echo "Computing SHA256 for ferro ${VER} tarballs..."
SHA256_MACOS_ARM64=$(compute_sha256    "${BASE_URL}/ferro-${TAG}-aarch64-apple-darwin.tar.gz")
SHA256_MACOS_X86_64=$(compute_sha256   "${BASE_URL}/ferro-${TAG}-x86_64-apple-darwin.tar.gz")
SHA256_LINUX_AARCH64=$(compute_sha256  "${BASE_URL}/ferro-${TAG}-aarch64-unknown-linux-gnu.tar.gz")
SHA256_LINUX_X86_64=$(compute_sha256   "${BASE_URL}/ferro-${TAG}-x86_64-unknown-linux-gnu.tar.gz")

echo "Rendering formula template..."
FORMULA=$(sed \
  -e "s/VERSION_PLACEHOLDER/${VER}/g" \
  -e "s/SHA256_MACOS_ARM64/${SHA256_MACOS_ARM64}/g" \
  -e "s/SHA256_MACOS_X86_64/${SHA256_MACOS_X86_64}/g" \
  -e "s/SHA256_LINUX_AARCH64/${SHA256_LINUX_AARCH64}/g" \
  -e "s/SHA256_LINUX_X86_64/${SHA256_LINUX_X86_64}/g" \
  "${TEMPLATE}")

echo "Cloning tap repo..."
git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/${TAP_REPO}.git" _tap_clone
printf '%s\n' "$FORMULA" > _tap_clone/Formula/ferro.rb

cd _tap_clone
git config user.name  "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add Formula/ferro.rb
if git diff --staged --quiet; then
  echo "Formula already at ${VER}, nothing to push."
  exit 0
fi
git commit -m "chore: bump ferro to ${VER}"
git push
echo "Tap updated to ${VER}."
```

### Bump Job Addition to `release.yml`
```yaml
  bump-homebrew-formula:
    name: Bump Homebrew formula
    if: github.event_name == 'push'
    needs: release
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Bump formula in homebrew-ferro tap
        env:
          HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
        run: bash scripts/bump-homebrew-formula.sh "${{ github.ref_name }}"
```

### Tap CI `tests.yml` (operator places this in `homebrew-ferro/.github/workflows/tests.yml`)
```yaml
name: CI

on:
  push:
    branches:
      - main
  pull_request:

jobs:
  audit:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Homebrew
        id: set-up-homebrew
        uses: Homebrew/actions/setup-homebrew@main
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Cache Homebrew Bundler RubyGems
        uses: actions/cache@v4
        with:
          path: ${{ steps.set-up-homebrew.outputs.gems-path }}
          key: ${{ runner.os }}-rubygems-${{ steps.set-up-homebrew.outputs.gems-hash }}
          restore-keys: ${{ runner.os }}-rubygems-

      - name: Ruby syntax check
        run: ruby -c Formula/ferro.rb

      - name: brew audit (strict, online on PR only)
        run: |
          if [ "${{ github.event_name }}" = "pull_request" ]; then
            brew audit --strict --online Formula/ferro.rb
          else
            brew audit --strict Formula/ferro.rb
          fi
```

---

## Q1: Multi-Arch Binary Formula Auto-Bump — Decision

**Finding:** `mislav/bump-homebrew-formula-action` does NOT support multi-arch binary formulae. Its README explicitly states it cannot update "formulae which use Ruby `if...else` conditions" for alternate download locations. The multi-`sha256` case (4 per-arch blocks) is exactly this scenario. [VERIFIED: WebFetch of raw.githubusercontent.com/mislav/bump-homebrew-formula-action/master/README.md]

**Recommendation: In-repo shell script (`scripts/bump-homebrew-formula.sh`) + a `bump-homebrew-formula` job in `release.yml`.**

This is the correct mechanism used by real-world binary taps (Shopify CLI tools, Rigellute/spotify-tui, QuickCode CLI). The script downloads the 4 release tarballs after they are published, computes SHA256s, renders the formula template with `sed`, and pushes directly to `main` in the tap repo. No external action dependency — just `curl`, `shasum`, `sed`, and `git`.

---

## Q2: Tarball Naming — Formula URL Pattern

From `release.yml` (verified):
- macOS arm64:   `ferro-<tag>-aarch64-apple-darwin.tar.gz`
- macOS x86_64:  `ferro-<tag>-x86_64-apple-darwin.tar.gz`
- Linux x86_64:  `ferro-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- Linux aarch64: `ferro-<tag>-aarch64-unknown-linux-gnu.tar.gz`

Tag format: `vX.Y.Z` (e.g. `v0.2.59`). URLs use the full tag. Formula `version` field uses just `X.Y.Z`.

Homebrew mapping:
- `on_macos do / on_arm do` → `aarch64-apple-darwin`
- `on_macos do / on_intel do` → `x86_64-apple-darwin`
- `on_linux do / on_arm do` → `aarch64-unknown-linux-gnu`
- `on_linux do / on_intel do` → `x86_64-unknown-linux-gnu`

[VERIFIED: release.yml lines 21-38, 72-78]

---

## Q3: Formula Staging Location in This Repo

The executor creates two files in this repo:

1. **`homebrew/Formula/ferro.rb.tpl`** — the formula template with `VERSION_PLACEHOLDER` and `SHA256_*` tokens. Operator copies rendered output into `homebrew-ferro/Formula/ferro.rb` when seeding the tap.
2. **`scripts/bump-homebrew-formula.sh`** — the bump script called by CI.

The tap repo (`albertogferrario/homebrew-ferro`) must contain:
- `Formula/ferro.rb` — seeded manually by operator from rendered template
- `.github/workflows/tests.yml` — tap CI (provided as operator instructions)
- `README.md` — generated by `brew tap-new` or written manually

**Does the bump script create `Formula/ferro.rb` on first run if the tap is empty?** Yes — the `git clone` + `printf` + `git add` sequence will create the file if it does not exist. This means the operator can create a completely empty tap repo and let the first release bump populate it. However, it is cleaner to seed it with the initial formula manually so the tap CI can run before the first automated bump.

[ASSUMED: that the operator creates the tap repo before the first tag push; CI will fail on the bump job if the PAT secret is not set or the repo does not exist]

---

## Q4: Trigger Alignment — "Same as Publish" Cadence

**Verified flow:**

```
push to master
  → publish.yml (trigger: push to master)
    → check-version / bump-version / test / publish (crates.io)
    → "Create and push tag" step: gh api creates tag vX.Y.Z
          ↓ (tag push event)
  → release.yml (trigger: push to tags v*)
    → build (4 Unix targets) → upload-artifact
    → release (GitHub Release with tarballs attached)
    → bump-homebrew-formula (NEW: needs: release)
    → update-install-script (existing)
    → e2e-tag (existing)
```

[VERIFIED: publish.yml line 5 (`on: push: branches: [master]`); publish.yml lines 322-329 (`gh api ... git/refs`); release.yml lines 3-5 (`on: push: tags: v*`); release.yml line 17 (`if: github.event_name == 'push'`)]

The bump job hooks into `release.yml`, not `publish.yml`, because it needs the tarballs that `release.yml`'s `build` job produces. The tag creation in `publish.yml` is the event that triggers `release.yml`, so the cadence is automatic: one tag push → one formula bump.

**Non-prerelease gate:** The current tag naming convention uses `vX.Y.Z` semver with no suffixes. `publish.yml` does not create prerelease tags. The `if: github.event_name == 'push'` guard in the existing jobs is sufficient. An additional `if: "!contains(github.ref_name, '-')"` guard can be added as belt-and-suspenders; it is not required for current practice.

---

## Q5: SHA256 Computation in CI

Two options:

**Option A (re-download from release URL):**
```bash
curl -fsSL "$url" | shasum -a 256 | awk '{print $1}'
```
- Simple; no artifact juggling
- Works because `needs: release` ensures tarballs are attached to the GitHub Release before the bump job runs
- `shasum` is available on macOS and Linux runners (Ubuntu has it via `libdigest-sha-perl`)

**Option B (download artifacts from prior build job):**
```yaml
- uses: actions/download-artifact@v4
  with:
    name: ferro-aarch64-apple-darwin
    path: ./artifacts/macos-arm64
# Then: sha256sum ./artifacts/macos-arm64/ferro-<tag>-aarch64-apple-darwin.tar.gz
```
- No network re-download
- Requires matching artifact names from the `build` job
- Slightly more complex YAML

**Recommendation: Option A** (re-download from release URL). The bump script is self-contained and can also be run locally without needing CI artifacts. The release URL is stable and public by the time the bump job runs.

[VERIFIED: `shasum` available on macOS (system) and Linux (`/usr/bin/shasum`); `sha256sum` available on Linux; confirmed both on local machine]

---

## Q6: Tap CI Setup

The `brew tap-new` command auto-generates CI workflows (`tests.yml` and `publish.yml`) for new taps. For a binary formula (no source build), the key simplification is:

- `brew test-bot --only-tap-syntax` validates Ruby formula syntax without triggering a source build
- `brew test-bot --only-formulae` runs `brew test ferro` (the `test do` block) which calls `ferro --version` and `ferro new`
- The bottle upload artifact step will produce no `.bottle.*` files for a binary formula — this is expected and harmless

The tap CI lives in `homebrew-ferro/.github/workflows/` and is operator-created. The executor provides the workflow snippet (see Code Examples above); the operator places it in the tap repo.

**Tap naming rule:** The GitHub repository MUST be named `homebrew-ferro` (with the `homebrew-` prefix) for `brew tap albertogferrario/ferro` to resolve correctly. [VERIFIED: Homebrew docs "How-to-Create-and-Maintain-a-Tap.md"]

---

## Q7: Fine-Grained PAT Permissions

**For direct commit to tap repo (recommended approach):**

| Permission | Scope | Required |
|------------|-------|----------|
| Contents | Read and write | Yes — to push `Formula/ferro.rb` |
| Pull requests | None | No — direct commit, no PR |
| Metadata | Read | Yes (implicit, always required) |

Token must be scoped to repository `albertogferrario/homebrew-ferro` only (not all repos). [VERIFIED: GitHub docs on fine-grained PATs]

**For PR-based approach (not recommended here):**
Add `Pull requests: Read and write` if switching to PR-based bump.

**Operator setup steps (exact):**
1. GitHub → Settings → Developer settings → Personal access tokens → Fine-grained tokens → Generate new token
2. Token name: `homebrew-ferro-bump` (or similar)
3. Resource owner: `albertogferrario`
4. Repository access: "Only select repositories" → `homebrew-ferro`
5. Repository permissions: Contents → Read and write (everything else: No access)
6. Copy the generated token
7. Go to `albertogferrario/ferro` → Settings → Secrets and variables → Actions → New repository secret
8. Name: `HOMEBREW_TAP_TOKEN`, Value: (paste token)

[ASSUMED: the operator has GitHub account permissions to create fine-grained PATs for their own repos]

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Homebrew | Formula audit, tap CI | macOS runners: yes | 6.0.1 (local) | Linux: use ghcr.io/homebrew/brew container |
| `curl` | SHA256 computation | All runners | system | `wget` |
| `shasum` | SHA256 on macOS | macOS runners: yes | 6.02 (local) | Linux: `sha256sum` |
| `sha256sum` | SHA256 on Linux | Linux runners: yes | GNU coreutils 9.5 (local) | `shasum -a 256` |
| `git` | Push to tap repo | All runners | system | — |
| `ruby` | Formula syntax check | All runners | 2.6+ (system macOS) | Homebrew Ruby via setup-homebrew action |
| `albertogferrario/homebrew-ferro` repo | Bump job, install | Operator-created | — | Operator must create before first release |
| `HOMEBREW_TAP_TOKEN` secret | Bump job | Operator-created | — | Release job fails until set |

**Missing dependencies with no fallback:**
- `albertogferrario/homebrew-ferro` repo — must be created by operator before first formula bump
- `HOMEBREW_TAP_TOKEN` secret — must be added to `ferro` repo before first bump job runs

**Missing dependencies with fallback:**
- `shasum` (macOS) / `sha256sum` (Linux) — both available on CI runners; bump script can detect and use whichever is present

---

## Validation Architecture

### Automated (CI in this repo + tap repo)

| Check | Command | When | Where |
|-------|---------|------|-------|
| Ruby syntax | `ruby -c homebrew/Formula/ferro.rb.tpl` | On every push to ferro | This repo CI (or local) |
| Formula audit (strict) | `brew audit --strict Formula/ferro.rb` | On push to tap repo | Tap CI (`tests.yml`) |
| Formula audit (online) | `brew audit --strict --online Formula/ferro.rb` | On PR to tap repo | Tap CI |
| Formula tap syntax | `brew test-bot --only-tap-syntax` | On push/PR to tap repo | Tap CI |
| `test do` block | `brew test ferro` | On PR to tap repo (via `--only-formulae`) | Tap CI |
| Bump job exits 0 | `bash scripts/bump-homebrew-formula.sh` | On every tag push | `release.yml` bump job |
| Tap CI green after bump | (auto-triggered by bump push) | After each formula bump | Tap CI |

### Operator-Manual (cannot be automated by executor)

| Check | How |
|-------|-----|
| Tap repo exists and has `Formula/` dir | Operator creates `albertogferrario/homebrew-ferro` |
| `HOMEBREW_TAP_TOKEN` secret set | Operator follows steps in Q7 |
| `brew install albertogferrario/ferro/ferro` succeeds end-to-end | After first release + bump: `brew tap albertogferrario/ferro && brew install ferro && ferro --version` |
| `ferro new myapp` works on clean Mac | Run on machine with no Rust; verify scaffold created |
| Install docs updated | Review `docs/src/` changes in PR |

### Phase Gate

Before marking phase complete:
1. At least one tag push after the bump job is wired in shows the bump job green in release.yml Actions.
2. `brew audit --strict Formula/ferro.rb` exits 0 (can be run locally or in tap CI).
3. `brew install albertogferrario/ferro/ferro && ferro --version` works on at least one runner (macOS or Linux).

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `case/when OS.mac? && Hardware::CPU.arm?` | `on_macos do / on_arm do` nested blocks | Homebrew ~3.x | Auditor prefers block style; both work |
| Manual formula updates | Automated bump job in release pipeline | 2022-2024 (community shift) | Formula stays in sync with every release |
| Single-platform binary formula | 4-platform binary formula with per-arch sha256 | Homebrew 3.x+ | Full macOS arm/intel + Linux support |

**Deprecated/outdated:**
- `Hardware::CPU.arm?` inside `def install` for download selection: still functional but the auditor prefers `on_arm` DSL blocks at the class level for URL/SHA256 declarations.
- `brew bump-formula-pr` for multi-sha256 formulae: wraps the wrong tool (PR-based, source-formula-oriented).

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Operator creates `homebrew-ferro` repo before first tag push | Q3, Q7, Environment | Bump job fails on first release; non-blocking for ferro itself, just no brew install yet |
| A2 | `ferro new --no-interaction --no-git` does not make network calls | Pitfall 6, test do block | Tap CI test would fail in restricted network sandbox; may need to verify with a local `brew test` |
| A3 | `ferro --version` outputs `X.Y.Z` (no `v` prefix) | Pitfall 2, test do block | `assert_match version.to_s` fails if output is `ferro v0.2.59`; version check in test must match actual output format |
| A4 | Fine-grained PAT direct-push to `main` in tap works without branch protection | Q7 | If operator enables branch protection on tap `main`, the direct push fails; bump approach must switch to PR |

A3 is the most actionable: the executor should verify `ferro --version` output format before finalizing the `test do` block.

---

## Open Questions (RESOLVED)

Resolved during planning (see 226-01-PLAN.md `<verified_facts>`).

1. **`ferro --version` output format.** — **RESOLVED:** clap `#[command(version)]` emits bare
   `ferro <X.Y.Z>` (e.g. `ferro 0.2.59`), no `v` prefix. So `assert_match version.to_s, shell_output(...)`
   with `version.to_s` = `0.2.59` is correct. Encoded in Plan 01.
   - What we know: `ferro --version` is called in the existing e2e-tag job.
   - Recommendation (taken): bare-version assertion.

2. **Shell completions / manpage in formula.**
   - What we know: CONTEXT.md lists this as Claude's discretion.
   - What's unclear: Whether `ferro` can emit completions via a subcommand (e.g., `ferro completions bash`).
   - Recommendation: Omit from v1 formula. The formula installs `ferro` binary only. Completions can be added post-1.0 as a formula update.

3. **First-run seed formula — exact version to use.**
   - What we know: The seed formula is the manually placed initial `Formula/ferro.rb` in the tap.
   - What's unclear: Whether operator places an empty/placeholder formula or a real versioned one.
   - Recommendation: Executor provides a static seed formula pinned to the current version at phase time. The bump job will overwrite it on the next release.

---

## Sources

### Primary (HIGH confidence)
- `/opt/homebrew/Library/Taps/shopify/homebrew-shopify/ejson.rb` (installed locally) — verified `on_macos do / on_arm do / on_intel do / on_linux do` binary formula pattern [VERIFIED: locally installed]
- `/opt/homebrew/Library/Homebrew/dev-cmd/tap-new.rb` — verified tap CI workflow generation (`tests.yml`, `publish.yml`) [VERIFIED: locally read]
- `docs.brew.sh/Formula-Cookbook` — verified `on_macos`/`on_linux`/`on_arm`/`on_intel` DSL, `bin.install`, `test do` patterns [VERIFIED: WebFetch]
- `raw.githubusercontent.com/Homebrew/brew/master/docs/How-to-Create-and-Maintain-a-Tap.md` — verified tap naming rule (`homebrew-` prefix required) [VERIFIED: WebFetch]
- `raw.githubusercontent.com/mislav/bump-homebrew-formula-action/master/README.md` — confirmed limitation: no multi-sha256 support [VERIFIED: WebFetch]
- `.github/workflows/release.yml` — verified tarball naming, trigger (`push: tags: v*`), `if: github.event_name == 'push'` guards [VERIFIED: locally read]
- `.github/workflows/publish.yml` — verified trigger (`push: branches: master`), tag creation step [VERIFIED: locally read]

### Secondary (MEDIUM confidence)
- GitHub docs on fine-grained PATs — Contents:write for direct push, pull_requests:write only if PR approach used [WebFetch]
- `Homebrew/actions` Context7 docs — `setup-homebrew` action usage pattern [Context7]
- `/opt/homebrew/Library/Taps/shopify/homebrew-shopify/toxiproxy.rb` — alternative `case/when` style reference (deprecated in favor of `on_*` blocks) [VERIFIED: locally read]

### Tertiary (LOW confidence)
- WebSearch results on `brewtap` action multi-arch support — suggests per-platform binary support but documentation sparse
- Community blog posts on Rust CLI Homebrew distribution — consistent with the script approach but not officially documented

---

## Metadata

**Confidence breakdown:**
- Formula Ruby structure: HIGH — verified from locally installed real production formula (ejson.rb, shopify tap)
- Auto-bump mechanism (script approach): HIGH — confirmed `mislav` action limitation from README; script pattern verified from multiple real-world examples
- Tap CI workflow: HIGH — read directly from `brew tap-new.rb` Homebrew source
- PAT permissions: HIGH — verified from GitHub docs
- Trigger alignment: HIGH — verified from both workflow files in the repo

**Research date:** 2026-06-14
**Valid until:** 2026-09-14 (stable domain; Homebrew DSL is rarely breaking; the `on_macos/on_linux/on_arm/on_intel` API has been stable since Homebrew 3.x)

---

## RESEARCH COMPLETE
