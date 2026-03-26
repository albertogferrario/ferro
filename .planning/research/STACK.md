# Stack Research

**Domain:** Rust framework documentation audit tooling
**Researched:** 2026-03-26
**Confidence:** HIGH

## Context

This research covers tooling and methodology for the v11.0 Framework Consolidation Audit milestone. The
goal is auditing documentation accuracy/completeness and philosophy consistency across 20 Rust crates
(~90,000 lines). The existing CI already runs `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS:
-Dwarnings`, which catches broken intra-doc links and invalid HTML. This research identifies what
additional tooling is needed and how well-documented Rust frameworks approach doc quality.

---

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `rustdoc` (built-in) | Rust 1.88.0 (stable) | API documentation generation and lint enforcement | The authoritative source; already in CI with `-Dwarnings`. Enforces `rustdoc::broken_intra_doc_links`, `rustdoc::invalid_codeblock_attributes`, `rustdoc::bare_urls`, `rustdoc::invalid_html_tags` on every build. |
| `#![warn(missing_docs)]` (built-in) | Rust 1.88.0 (stable) | Detect undocumented public API items | The canonical tool for doc coverage. Add per-crate, escalate to `deny` in CI once addressed. Works on stable, unlike `--show-coverage`. |
| `mdBook` | Latest stable (used in `docs/`) | User-facing documentation site | Already in use. Generates the book at docs.ferro-rs.dev. No change needed to the tool itself. |
| `mdbook-linkcheck` | 0.7.7 | Broken link detection in mdBook | Detects broken internal and external links in the book source. Last release 2022 but still functional; used by the Rust project itself. Catches dead `docs.ferro-rs.dev/` cross-references. |

### Supporting Tools

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `cargo-rdme` | 1.4.8 (stable) | Sync README.md from crate-level `//!` docs | After fixing crate-level docs, run to keep per-crate READMEs in sync. One command per crate. Stable toolchain. |
| `cargo-semver-checks` | 0.44.0+ (stable) | Detect API consistency violations and breaking changes | Run during the audit to catch accidental API breaks that occur during cleanup. 245 lints as of 2026. |
| `cargo-deny` | Already configured (`deny.toml`) | Security and license consistency | Already in CI. No change needed. |
| `RUSTDOCFLAGS=-D rustdoc::missing_crate_level_docs` | Built-in, stable | Enforce that every crate has a crate-level doc comment | Add to the audit CI step to identify crates missing top-level `//!` docs. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo doc --no-deps --all-features --open` | Build and view the full API docs locally | Use to visually audit what docs.rs will show. The `--no-deps` flag keeps it fast. |
| `RUSTDOCFLAGS="-D warnings -D missing_docs"` (local) | Find all undocumented public items in a single pass | Run locally before adding `#![warn(missing_docs)]` to a crate to see the full scope. Not for CI yet — too many violations likely exist. |
| `cargo test --doc --all-features` | Run all doctests | Already included in `cargo test --all-features`. Verifies code examples compile and behave correctly. |
| `mdbook build` | Build and validate user-facing book | Run locally to catch mdBook-specific issues. Add `mdbook-linkcheck` output to CI output. |

---

## Installation

```bash
# mdbook-linkcheck — add to book.toml, then install the binary
cargo install mdbook-linkcheck

# cargo-rdme — for README sync from crate docs
cargo install cargo-rdme

# cargo-semver-checks — API consistency
cargo install cargo-semver-checks

# mdbook itself (if not present)
cargo install mdbook
```

Add to `docs/book.toml` for link checking:
```toml
[output.linkcheck]
follow-web-links = false
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `#![warn(missing_docs)]` per-crate | `RUSTDOCFLAGS='--show-coverage'` | The `--show-coverage` flag is nightly-only and unstable (tracking issue #58154). `missing_docs` is stable and integrates with CI `-D warnings`. Do not use `--show-coverage` on a stable toolchain project. |
| `cargo-rdme` | `cargo-sync-rdme` | `cargo-sync-rdme` requires nightly toolchain. `cargo-rdme` works on stable. For a stable-pinned workspace (1.88.0 MSRV), `cargo-rdme` is the correct choice. |
| `mdbook-linkcheck` | `cargo-deadlinks` | `cargo-deadlinks` (0.8.1, last release 2021) checks rustdoc HTML output, not mdBook source. For the user-facing docs book, `mdbook-linkcheck` is the right tool. For rustdoc HTML, the built-in `rustdoc::broken_intra_doc_links` lint in CI is already sufficient. |
| Manual audit + `missing_docs` | Custom CI scripts parsing rustdoc JSON | The rustdoc JSON output format is unstable. `cargo-semver-checks` already uses it internally. Writing custom scripts against unstable JSON is maintenance burden. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `RUSTDOCFLAGS='--show-coverage'` | Nightly-only (`-Z unstable-options` required). Doesn't integrate with stable CI. Known regression: deletes existing docs. | `#![warn(missing_docs)]` in each crate |
| `rustdoc::missing_doc_code_examples` lint | Nightly-only lint, not available on stable 1.88.0. | Manual review of existing examples; `cargo test --doc` to verify they work |
| `cargo-sync-rdme` | Requires nightly toolchain. Cannot use with MSRV 1.88.0. | `cargo-rdme` (stable) |
| `rust-semverver` | Requires nightly, abandoned/maintenance mode. | `cargo-semver-checks` (stable, actively maintained) |
| Adding `#![deny(missing_docs)]` immediately to all crates | Will cause CI failures before docs are written; blocks all other work. | Add `#![warn(missing_docs)]` first, fix incrementally, escalate to `deny` only after a crate is complete. |

---

## How Well-Documented Rust Frameworks Approach Doc Quality

### Axum (tokio-rs/axum)

Axum uses structured module-level docs with markdown headers, inline code examples throughout,
and `#[cfg_attr(docsrs, feature(doc_cfg))]` for feature-gated documentation on docs.rs. It
does not use a blanket `#![warn(missing_docs)]` — instead it enforces docs selectively. The
primary enforcement is `rustdoc::broken_intra_doc_links` in CI.

**Pattern:** Every public type and method has a one-sentence summary, followed by a code example
with the `# use` boilerplate hidden behind `#` lines.

### Rust API Guidelines (official reference)

The official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/documentation.html)
define the standard for crate documentation quality. The relevant items for this audit:

| Guideline ID | Requirement |
|-------------|-------------|
| C-CRATE-DOC | Crate root (`//!` comment) is thorough and includes examples |
| C-EXAMPLE | Every public module, struct, enum, function, trait, and type has a rustdoc example |
| C-QUESTION-MARK | Examples use `?`, not `unwrap()` (hide boilerplate with `#` lines) |
| C-FAILURE | Functions document panics (`# Panics`), errors (`# Errors`), safety (`# Safety`) |
| C-LINK | Prose uses intra-doc links (`[`TypeName`]`) to related items |
| C-HIDDEN | `#[doc(hidden)]` used for internal implementation details |

### Agent-First Documentation Patterns

For an agent-first framework specifically, documentation must satisfy both human readers and
LLM context windows. Findings from examining AI agent framework documentation (Rig, AutoAgents,
Kowalski) and the Ferro AGENTS.md:

1. **Machine-readable structure is paramount.** Agents parse docs for patterns to replicate.
   `# Panics`, `# Errors`, `# Examples` sections with consistent headings are more parseable
   than prose descriptions.

2. **Every type needs a purpose statement.** Agents infer usage from the first sentence of a
   doc comment. "A builder for X" or "Extracts Y from a request" — concrete, single-sentence.

3. **Examples must be minimal but complete.** Agent-generated code copies the nearest example
   verbatim. Long examples with 20 setup lines generate incorrect code. Examples should be
   the shortest valid usage.

4. **Error messages are docs.** For Ferro specifically, the `# Errors` section on handler
   functions and middleware should describe exactly what HTTP status is returned and when.
   Agents use this to generate correct error handling.

5. **Cross-crate relationships need explicit documentation.** When `ferro-cache` is used with
   `framework`, the integration point must be documented in both crates. Agents cannot infer
   cross-crate relationships from types alone.

---

## The Documentation Audit Approach

The methodology used by well-maintained Rust projects (Tokio, Axum, SeaORM) follows this
sequence, which maps directly to the audit phases:

### Phase 1: Mechanical Audit (tooling-enforced)
Run these checks to surface all mechanical gaps:

```bash
# Find all broken intra-doc links (already enforced in CI)
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Find all public items with no doc comment (run per-crate to scope the work)
RUSTDOCFLAGS="-D warnings -D missing_docs" cargo doc --no-deps --features [crate-features] -p [crate-name]

# Find broken links in mdBook
mdbook build docs/ 2>&1 | grep -E "ERROR|WARN"

# Verify all doc examples compile and pass
cargo test --doc --all-features
```

### Phase 2: Quality Audit (human review)
Check against C-EXAMPLE, C-FAILURE, C-LINK guidelines. No tool automates this.

For each public type, verify:
- First sentence answers "what is this for?"
- Code example shows why, not just how
- `# Errors` section present if the function returns `Result`
- `# Panics` section present if the function can panic
- Intra-doc links used for all references to other types

### Phase 3: Philosophy Audit (human review)
Verify agent-first consistency:
- Does the doc read correctly in an LLM context window?
- Are patterns documented once, not scattered?
- Do error messages match what the framework actually returns?
- Is the "agent-first" framing consistent — tools, not just types?

### Phase 4: Fix and Enforce
After addressing all gaps:
- Add `#![warn(missing_docs)]` to each crate as it's completed
- Add mdbook-linkcheck to CI
- Add `cargo-semver-checks` to CI

---

## Stack Patterns by Variant

**If adding `#![warn(missing_docs)]` to a crate with many violations:**
- Add `#![allow(missing_docs)]` at the crate root first
- Then add `#![warn(missing_docs)]` to individual modules as they are fixed
- This is how the Tokio project handled it during their documentation push

**If a crate has internal implementation modules exposed as `pub(crate)`:**
- Do not add doc comments to `pub(crate)` items — `missing_docs` does not require them
- Focus doc effort on the public API surface

**If user-facing docs (mdBook) contradict API docs (rustdoc):**
- Fix rustdoc first (authoritative), then update mdBook to match
- Inertia, JSON-UI, and auth are the highest-risk sections based on codebase growth since docs were written

---

## Version Compatibility

| Tool | Rust Toolchain | Notes |
|------|----------------|-------|
| `rustdoc` lints | 1.88.0 stable | `broken_intra_doc_links`, `missing_crate_level_docs` stable since ~1.52 |
| `#![warn(missing_docs)]` | 1.88.0 stable | Stable since 1.0 |
| `cargo-rdme` | 1.88.0 stable | Stable toolchain required |
| `mdbook-linkcheck` | Any | CLI binary, not tied to Rust version |
| `cargo-semver-checks` | 1.88.0 stable | Uses rustdoc JSON internally but exposes stable CLI |
| `--show-coverage` | Nightly only | **Do not use** — requires `-Z unstable-options` |
| `missing_doc_code_examples` lint | Nightly only | **Do not use** — unstable |

---

## What the Audit Will Find (Predicted Gaps)

Based on examining the codebase structure and existing doc patterns across the 20 crates:

| Crate | Predicted Gap | Severity |
|-------|--------------|----------|
| `framework/src/lib.rs` | No crate-level `//!` doc comment at all | HIGH |
| `framework/src/` modules | Mixed — some modules have docs, many functions don't | HIGH |
| `ferro-json-ui` | Good crate-level docs; individual component functions likely sparse | MEDIUM |
| `ferro-mcp` | Minimal docs — MCP is internal tooling but used by AI agents | HIGH |
| `ferro-projections` | Complex types, likely underdocumented relative to complexity | HIGH |
| `ferro-api-mcp` | Binary crate, docs less critical but CLI help text needs accuracy | MEDIUM |
| mdBook (`docs/`) | Features added in v8.1, v9.0, v10.0 likely underdocumented | HIGH |
| `ferro-theme` | New crate, likely sparse docs | MEDIUM |

---

## Sources

- [Rustdoc lints — official reference](https://doc.rust-lang.org/rustdoc/lints.html) — all lint names and stability (HIGH confidence)
- [Rust API Guidelines — documentation section](https://rust-lang.github.io/api-guidelines/documentation.html) — C-CRATE-DOC through C-HIDDEN checklist (HIGH confidence)
- [Rust API Guidelines — checklist](https://rust-lang.github.io/api-guidelines/checklist.html) — complete guideline IDs (HIGH confidence)
- [Axum lib.rs source](https://github.com/tokio-rs/axum/blob/main/axum/src/lib.rs) — axum documentation style and lint configuration (HIGH confidence)
- [cargo-semver-checks 0.44.0](https://crates.io/crates/cargo-semver-checks) — 245 lints, stable toolchain (HIGH confidence)
- [cargo-rdme 1.4.8](https://docs.rs/crate/cargo-rdme/latest) — stable README sync tool (HIGH confidence)
- [mdbook-linkcheck 0.7.7](https://github.com/Michael-F-Bryan/mdbook-linkcheck) — book link checking (MEDIUM confidence — last release 2022, still functional)
- [cargo-deadlinks 0.8.1](https://docs.rs/crate/cargo-deadlinks/latest) — HTML link checker, last release 2021 (LOW — superceded by built-in lint for intra-doc links)
- [rustdoc --show-coverage tracking issue #58154](https://github.com/rust-lang/rust/issues/58154) — confirmed unstable, nightly-only (HIGH confidence)
- [RFC 1574 — more API documentation conventions](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.md) — Panics/Errors/Safety section conventions (HIGH confidence)
- [ferro CI workflow](/.github/workflows/ci.yml) — confirmed existing `RUSTDOCFLAGS: -Dwarnings` and `cargo doc` job (HIGH confidence — direct codebase inspection)

---

*Stack research for: v11.0 Framework Consolidation Audit — documentation tooling methodology*
*Researched: 2026-03-26*
