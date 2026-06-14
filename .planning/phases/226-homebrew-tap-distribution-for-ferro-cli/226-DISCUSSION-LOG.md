# Phase 226: Homebrew Tap Distribution for ferro-cli - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.

**Date:** 2026-06-14
**Phase:** 226-homebrew-tap-distribution-for-ferro-cli
**Mode:** interactive discuss
**Areas discussed:** formula form & platforms, auto-bump mechanism, tap-push auth, formula QA gates

---

## Formula Form & Platforms

| Option | Description | Selected |
|--------|-------------|----------|
| Binary, macOS + Linux | Prebuilt binaries for mac arm64/x86_64 + Linux x86_64/aarch64 | ✓ |
| Binary, macOS only | mac arm64/x86_64 only | |
| Binary + source fallback | binaries + `depends_on "rust" => :build` | |

**User's choice:** Binary, macOS + Linux. Widest zero-toolchain reach off the existing release tarballs.

---

## Auto-bump Mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Action, stable tags only | maintained action, non-prerelease gate | (basis) |
| In-repo script | custom shell bump | |
| Manual for now | hand-bump SHAs | |

**User's choice (Other):** "Action, same as publish" — use a maintained action, wired into the existing
release/publish automation so the formula bump fires on the same real-release event as the artifact/crates.io
publish (not a manual side-process). Interpreted + recorded as D-03, gated to non-prerelease tags.

---

## Tap-push Auth

| Option | Description | Selected |
|--------|-------------|----------|
| Fine-grained PAT secret | repo-scoped token (HOMEBREW_TAP_TOKEN), contents:write on homebrew-ferro only | ✓ |
| Deploy key (SSH) | repo-scoped SSH key | |
| GitHub App token | short-lived app tokens | |

**User's choice:** Fine-grained PAT secret. Least-privilege, standard, one-time operator setup.

---

## Formula QA Gates

| Option | Description | Selected |
|--------|-------------|----------|
| test block + audit | `test do` (ferro --version/new) + `brew audit --strict --online` in CI | ✓ |
| test block only | test block, no audit job | |
| Minimal | bare formula | |

**User's choice:** test block + audit. Mirrors Phase 225's detect-before-users-hit-it philosophy.

---

## Locked Before Discussion (prior decision + roadmap)

- Own tap, not homebrew-core (pre-1.0 notability gate + `ferro` name collision).
- Binary formula off existing release tarballs.
- Auto-bump from release.yml.
- Surface `brew install` in docs.

## Deferred Ideas

- homebrew-core submission (post-1.0)
- source-fallback formula
- Windows package managers (scoop/winget)
