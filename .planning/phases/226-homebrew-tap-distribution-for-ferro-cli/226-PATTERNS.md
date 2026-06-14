# Phase 226: Homebrew Tap Distribution for ferro-cli - Pattern Map

**Mapped:** 2026-06-14
**Files analyzed:** 5 (2 new in this repo, 1 edit in this repo, 2 staged-for-operator)
**Analogs found:** 4 / 5 (1 file has no in-repo Ruby analog)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `scripts/bump-homebrew-formula.sh` | utility / CI script | batch (download → transform → push) | `scripts/install.sh` | role-match (same shell conventions, same 4 targets) |
| `.github/workflows/release.yml` (new `bump-homebrew-formula` job) | CI config | event-driven (tag push → post-release job) | `release.yml` `update-install-script` job (lines 120–143) | exact (same job shape, same event guard, same git-commit-push pattern) |
| `homebrew/Formula/ferro.rb.tpl` | config / template | transform (placeholders → rendered Ruby) | no in-repo Ruby analog | no match — use RESEARCH.md template |
| `homebrew/tap-ci/tests.yml` (staged for operator) | CI config (external repo) | event-driven | `.github/workflows/release.yml` job structure (shape only) | partial (different repo target) |
| `docs/src/getting-started/installation.md` + `README.md` | documentation | — | existing files (edit, not create) | exact (edit the `## Installing the CLI` / `## Quick Start` sections already present) |

---

## Pattern Assignments

### `scripts/bump-homebrew-formula.sh` (utility, batch)

**Analog:** `scripts/install.sh`

**Shell conventions** (lines 1–6): use `#!/bin/sh` for POSIX portability in install.sh — but install.sh itself uses `set -e` only (not `set -euo pipefail`). The bump script runs only in CI (bash is guaranteed on GitHub runners), so upgrade to `#!/usr/bin/env bash` + `set -euo pipefail` for safer CI execution. Still, install.sh is the shape reference.

**Shebang + error-exit pattern** (`scripts/install.sh` lines 1–6):
```sh
#!/bin/sh
# Ferro Framework Installer
# Usage: curl -fsSL ... | sh

set -e
```

**Repo/binary constants pattern** (`scripts/install.sh` lines 9–11):
```sh
REPO="albertogferrario/ferro"
BINARY_NAME="ferro"
INSTALL_DIR="${FERRO_INSTALL_DIR:-$HOME/.ferro/bin}"
```
Copy this constant-block style for the bump script's constants (`TAP_REPO`, `TEMPLATE`, `BASE_URL`).

**OS/arch detection and the 4 target triples** (`scripts/install.sh` lines 38–70):
```sh
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)   OS="linux" ;;
        Darwin)  OS="darwin" ;;
        MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
        *)       error "Unsupported operating system: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)    ARCH="x86_64" ;;
        arm64|aarch64)   ARCH="aarch64" ;;
        *)               error "Unsupported architecture: $ARCH" ;;
    esac

    PLATFORM="${OS}-${ARCH}"
}
```

**Tarball naming used in install.sh** (`scripts/install.sh` lines 86–91):
```sh
ARCHIVE_NAME="ferro-${VERSION}-${PLATFORM}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"
```

The bump script must use the SAME four target strings that install.sh maps to:
- `aarch64-apple-darwin` (Darwin + arm64/aarch64)
- `x86_64-apple-darwin` (Darwin + x86_64/amd64)
- `aarch64-unknown-linux-gnu` (Linux + arm64/aarch64)
- `x86_64-unknown-linux-gnu` (Linux + x86_64/amd64)

These are the same four `matrix.target` values from `release.yml` lines 24–37. The bump script does NOT do runtime OS detection (it processes all four); it uses these strings as literal constants.

**Error helper pattern** (`scripts/install.sh` lines 20–35):
```sh
info()    { printf "${CYAN}info${NC}: %s\n" "$1"; }
success() { printf "${GREEN}success${NC}: %s\n" "$1"; }
warn()    { printf "${YELLOW}warn${NC}: %s\n" "$1"; }
error()   { printf "${RED}error${NC}: %s\n" "$1"; exit 1; }
```
The bump script is CI-only so color codes are optional, but use `echo` / `printf` logging before each major step (same as install.sh's `info` calls).

**curl download pattern** (`scripts/install.sh` lines 105–111):
```sh
if command -v curl > /dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_NAME"
elif command -v wget > /dev/null; then
    wget -q "$DOWNLOAD_URL" -O "$ARCHIVE_NAME"
else
    error "curl or wget is required"
fi
```
Bump script simplifies to `curl -fsSL "$url" | shasum -a 256 | awk '{print $1}'` (piped; no file save needed).

**Version prefix strip.** install.sh receives `VERSION` already with `v` prefix from GitHub (tag name). Bump script must do `VER="${TAG#v}"` to strip it for the formula `version` field, while keeping `$TAG` for the URL paths. install.sh does NOT strip the prefix (it uses the tag directly in URLs) — the bump script diverges here intentionally.

---

### `.github/workflows/release.yml` — new `bump-homebrew-formula` job (CI config, event-driven)

**Analog:** existing `update-install-script` job in `release.yml` (lines 120–143)

This is the exact-match analog: same `if: github.event_name == 'push'` guard, same `needs: release`, same `runs-on: ubuntu-latest`, same `actions/checkout@v4`, same git-config + commit + push shell block.

**Full `update-install-script` job** (`release.yml` lines 120–143):
```yaml
  update-install-script:
    name: Update Install Script
    if: github.event_name == 'push'
    needs: release
    runs-on: ubuntu-latest
    permissions:
      contents: write

    steps:
      - uses: actions/checkout@v4

      - name: Update repo in install script
        run: |
          REPO="${{ github.repository }}"
          sed -i "s|REPO=\"albertogferrario/ferro\"|REPO=\"$REPO\"|g" scripts/install.sh
          sed -i "s|REPO=\"albertogferrario/ferro\"|REPO=\"$REPO\"|g" scripts/create-app.sh

      - name: Commit changes
        run: |
          git config user.name github-actions
          git config user.email github-actions@github.com
          git add scripts/install.sh scripts/create-app.sh
          git diff --staged --quiet || git commit -m "chore: update install scripts with repo name"
          git push
```

**New `bump-homebrew-formula` job — copy this shape:**
- Same `if: github.event_name == 'push'` guard (lines 17, 95, 122, 148 establish this convention throughout release.yml)
- `needs: release` (waits for tarballs to be attached to the GitHub Release, not just `needs: build`)
- `runs-on: ubuntu-latest`
- `actions/checkout@v4` to get the bump script from this repo
- Single run step calling `bash scripts/bump-homebrew-formula.sh "${{ github.ref_name }}"` with `HOMEBREW_TAP_TOKEN` injected via `env:`
- No `permissions: contents: write` needed here (the bump script pushes to the TAP repo using the PAT, not to this repo using `GITHUB_TOKEN`)

**Difference from update-install-script:** the bump job does NOT need `permissions: contents: write` (it does not push to this repo). It needs `HOMEBREW_TAP_TOKEN` secret only. The git operations are inside the bump script, authenticating via the embedded PAT.

**Secrets usage pattern** — reference from `release.yml` line 117:
```yaml
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```
The bump job uses:
```yaml
        env:
          HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
```

**Event guard convention** (`release.yml` lines 17, 95, 122, 148 — all tag-only jobs use):
```yaml
    if: github.event_name == 'push'
```
The bump job MUST use the same guard. Do not use `if: startsWith(github.ref, 'refs/tags/')` — that is not the established pattern here.

**Job placement:** slot `bump-homebrew-formula` between `release` and `update-install-script` in the YAML. Both `needs: release` — they are siblings, not sequential.

---

### `homebrew/Formula/ferro.rb.tpl` (config / template)

**No in-repo Ruby analog.** There are no `.rb` files anywhere in the repository.

**Source of truth:** Use the complete template from RESEARCH.md verbatim (lines 410–457 of 226-RESEARCH.md). Key structural choices verified from the locally installed `ejson.rb` Shopify formula and Homebrew Formula-Cookbook docs:
- Class name: `Ferro` (capitalized, matches Homebrew class-name convention)
- `on_macos do / on_arm do` nested-block DSL (NOT the deprecated `case/when Hardware::CPU.arm?` style)
- One `url` + `sha256` per leaf block
- `def install` block with `bin.install "ferro"` (binary is at archive root — verified from release.yml `tar -czvf ... ferro` at line 77)
- `test do` block with `assert_match version.to_s` + `system bin/"ferro", "new", ...`
- `livecheck` block pointing to `:stable` + `:github_latest`
- Placeholder tokens: `VERSION_PLACEHOLDER`, `SHA256_MACOS_ARM64`, `SHA256_MACOS_X86_64`, `SHA256_LINUX_AARCH64`, `SHA256_LINUX_X86_64`

**`ferro --version` output format (Assumption A3 from RESEARCH.md):** Before finalizing the `test do` block, verify by checking the e2e-tag job output or running `cargo run -p ferro-cli -- --version` locally. If output is `ferro 0.2.X` (with binary name prefix), the assert must match accordingly: `assert_match "#{version}", shell_output("#{bin}/ferro --version")`. If it is bare `0.2.X`, use `version.to_s`.

---

### `homebrew/tap-ci/tests.yml` (CI config, staged for operator — external repo)

**No in-repo analog** for tap-specific Homebrew CI. The shape of `.github/workflows/release.yml` provides the general `jobs:` / `steps:` / `uses:` YAML structure, but the content is entirely Homebrew-specific.

**Source of truth:** Use the complete `tests.yml` snippet from RESEARCH.md (lines 529–568). The key decisions already made in research:
- `brew audit --strict` on push; add `--online` on PR only (avoids URL-not-yet-propagated failures)
- `ruby -c Formula/ferro.rb` as the fastest syntax check (no Homebrew binary required)
- `Homebrew/actions/setup-homebrew@main` with `secrets.GITHUB_TOKEN` for runner setup
- `actions/cache@v4` for Homebrew RubyGems
- Run `brew test-bot --only-tap-syntax` for push; `brew test-bot --only-formulae` for PR

**Operator instructions:** This file is staged in `homebrew/tap-ci/` (or provided as literal content in the plan). The operator places it at `albertogferrario/homebrew-ferro/.github/workflows/tests.yml`.

---

### `docs/src/getting-started/installation.md` + `README.md` (documentation, edit)

**Analog:** the files themselves — these are edits, not new files.

**Current install section in `docs/src/getting-started/installation.md`** (lines 9–15):
```markdown
## Installing the CLI

Install the Ferro CLI globally:

```bash
cargo install ferro-cli
```
```

**Planner action:** Expand this section to list three install methods in priority order: Homebrew first (zero-prerequisites, recommended for Mac/Linux), curl one-liner second, cargo third. Structure:

```markdown
## Installing the CLI

### Homebrew (macOS and Linux — recommended)

```bash
brew install albertogferrario/ferro/ferro
```

### curl installer (macOS and Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/albertogferrario/ferro/main/scripts/install.sh | sh
```

### Cargo (requires Rust)

```bash
cargo install ferro-cli
```
```

**Current `README.md` Quick Start** (lines 15–22):
```markdown
## Quick Start

```bash
cargo install ferro-cli
ferro new myapp
cd myapp
ferro serve
```
```

**Planner action:** Prepend the `brew install` one-liner before the `cargo install` line, or make `brew install` the shown path and move `cargo install` to a secondary note. The exact wording is Claude's discretion per D-06.

---

## Shared Patterns

### Event guard for tag-only jobs
**Source:** `release.yml` (lines 17, 95, 122, 148)
**Apply to:** the new `bump-homebrew-formula` job
```yaml
if: github.event_name == 'push'
```
This is the established idiom for "runs only on tag push, not on `workflow_dispatch` or `schedule`."

### `needs: release` dependency chain
**Source:** `release.yml` lines 121 (`update-install-script`) and 148 (`e2e-tag`)
**Apply to:** `bump-homebrew-formula`
```yaml
needs: release
```
Both existing post-release jobs (`update-install-script`, `e2e-tag`) set `needs: release` — `bump-homebrew-formula` is a sibling, not sequential to either.

### git config + idempotent commit pattern
**Source:** `release.yml` lines 138–143 (`update-install-script` step)
```yaml
          git config user.name github-actions
          git config user.email github-actions@github.com
          git add scripts/install.sh scripts/create-app.sh
          git diff --staged --quiet || git commit -m "chore: update install scripts with repo name"
          git push
```
**Apply to:** bump script's git block (inside the shell script, not YAML). Use the same `git diff --staged --quiet && exit 0` pattern to make the commit step idempotent (no-op if formula already at this version). Note: bump script uses `github-actions[bot]` name + `github-actions[bot]@users.noreply.github.com` email (the canonical bot identity for external-repo commits; the update-install-script job pushes to THIS repo so it uses the shorter `github-actions` form — either works).

### Tarball naming convention
**Source:** `release.yml` lines 71–77 (Prepare artifact step) and `scripts/install.sh` lines 86–91
**Apply to:** bump script (URL construction) and `ferro.rb.tpl` (URL patterns)
```
ferro-${VERSION}-${TARGET}.tar.gz
```
Where VERSION is the full tag including `v` prefix (e.g. `v0.2.59`) and TARGET is one of the four Rust target triples. The formula `version` field uses VERSION without the `v` prefix.

### `actions/checkout@v4` as first step
**Source:** All jobs in `release.yml`
**Apply to:** `bump-homebrew-formula` job
```yaml
    steps:
      - uses: actions/checkout@v4
```
This is required in the bump job to access `scripts/bump-homebrew-formula.sh` and `homebrew/Formula/ferro.rb.tpl` from this repo.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `homebrew/Formula/ferro.rb.tpl` | config / template | transform | No Ruby files anywhere in the repository. Use RESEARCH.md lines 410–457 as the source of truth. |
| `homebrew/tap-ci/tests.yml` | CI config | event-driven | Targets an external repo (`homebrew-ferro`); no Homebrew-specific CI in this repo. Use RESEARCH.md lines 529–568. |

---

## Metadata

**Analog search scope:** `scripts/`, `.github/workflows/`, `docs/src/`, `README.md`
**Files read:** `scripts/install.sh`, `.github/workflows/release.yml`, `.github/workflows/publish.yml` (trigger structure), `docs/src/getting-started/installation.md`, `README.md`
**Pattern extraction date:** 2026-06-14
