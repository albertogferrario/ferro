# Phase 114: Metadata & Publication Readiness - Research

**Researched:** 2026-03-27
**Domain:** Rust crate publication metadata, cargo doc, missing_docs lint
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **README depth & structure:** ~30-50 lines per README. Consistent template: title, description, features list, usage example, link to docs, license. Links point to `https://docs.ferro-rs.dev`.
- **missing_docs strategy:** Add `#![warn(missing_docs)]` to framework crate and fix ALL warnings — no partial state. Fix all warnings even if it touches 50+ files. Publication-ready means clean build.
- **Cargo.toml metadata:** `homepage = "https://ferro-rs.dev"` for all crates. `readme = "README.md"` per crate. Fill missing fields on ferro-broadcast, ferro-theme, ferro-projections: `readme`, `homepage`, `categories`.

### Claude's Discretion
- Code example style per README (runnable vs illustrative)
- Doc comment depth per undocumented item
- Category selection for crates.io

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| META-01 | Cargo.toml metadata gaps fixed (ferro-broadcast, ferro-theme, ferro-projections) | Gap analysis below: 3 crates missing `readme`, `homepage`, and (for ferro-theme, ferro-projections) `categories` |
| META-02 | `#![warn(missing_docs)]` added to framework crate | 136 warnings quantified across 19 source files; fixable with doc comments |
| META-03 | Stub READMEs expanded (ferro-json-ui, ferro-lang, ferro-whatsapp) | All 3 README files confirmed ≤ 9 lines; template defined in decisions |
| META-04 | Crate-level `//!` doc comments added to ferro-json-ui and ferro-lang lib.rs | Already satisfied: ferro-json-ui has 39 lines, ferro-lang has 23 lines of `//!` comments |
</phase_requirements>

---

## Summary

Phase 114 is a publication-readiness pass covering three distinct work areas: Cargo.toml metadata gaps, a `#![warn(missing_docs)]` lint enforcement pass on the framework crate, and README expansion for stub crates.

The metadata gaps are straightforward: three crates (ferro-broadcast, ferro-theme, ferro-projections) are missing `readme`, `homepage`, and in two cases `categories`. The reference pattern is established in `framework/Cargo.toml` and the decisions lock the homepage URL to `https://ferro-rs.dev`.

The `missing_docs` work is the most substantial task. Running `cargo rustc --package ferro-rs --lib -- -W missing-docs` reveals 136 warning locations across 19 source files. The largest concentrations are in `validation/rules.rs` (24), `routing/macros.rs` (13), `http/resources/pagination.rs` (10), `inertia/context.rs` (9), and `http/cookie.rs` (9). None of these require structural changes — all gaps are missing doc comments on pub items.

META-04 is already satisfied: `ferro-json-ui/src/lib.rs` has 39 lines of `//!` doc comments and `ferro-lang/src/lib.rs` has 23. The planner should note this so those items are verified (not re-implemented) and the effort is credited.

**Primary recommendation:** Split into two plans — Plan 01: Cargo.toml metadata + `#![warn(missing_docs)]` in lib.rs + all doc comment fixups; Plan 02: README expansion + META-04 verification.

---

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| `cargo doc` | stable | Generate and verify API docs | Standard Rust toolchain |
| `#![warn(missing_docs)]` | N/A | Lint attribute for documentation coverage | Rust built-in; the only way to enforce doc coverage |

### Supporting
| Field | Value | Purpose | When to Use |
|-------|-------|---------|-------------|
| `readme` in Cargo.toml | `"README.md"` | Points crates.io to README for crate page | Required for any published crate with useful README |
| `homepage` in Cargo.toml | URL string | Shown on crates.io crate page | Use for official project site |
| `categories` in Cargo.toml | `["category-slug"]` | crates.io discovery taxonomy | Use slugs from crates.io category list |
| `keywords` in Cargo.toml | up to 5 strings | crates.io search indexing | Each crate already has keywords; no changes needed |

**crates.io category slugs used in this project:**
- `"web-programming"` — general web crates
- `"web-programming::http-server"` — server-side HTTP crates
- `"asynchronous"` — async runtime integrations
- `"internationalization"` — i18n/localization crates
- `"data-structures"` — when applicable

---

## Architecture Patterns

### Pattern 1: Cargo.toml Metadata Reference Pattern
**What:** All crates in this workspace use workspace inheritance for version, edition, and license. Unique per-crate fields are: description, repository, homepage, readme, keywords, categories.
**When to use:** Every crate — no exceptions for publication readiness.
**Example:**
```toml
# Source: framework/Cargo.toml (complete reference)
[package]
name = "ferro-rs"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "A Laravel-inspired web framework for Rust"
repository = "https://github.com/albertogferrario/ferro"
homepage = "https://ferro-rs.dev"
keywords = ["web", "framework", "async", "http", "ferro"]
categories = ["web-programming::http-server", "asynchronous"]
readme = "README.md"
```

### Pattern 2: missing_docs Attribute Placement
**What:** `#![warn(missing_docs)]` is a crate-level inner attribute placed at the top of `lib.rs`, before all other content except any `#![allow(...)]` overrides.
**When to use:** Any crate targeting publication-ready state.
**Example:**
```rust
// framework/src/lib.rs — top of file, before all pub mod declarations
#![warn(missing_docs)]
//! # Ferro
//!
//! A Laravel-inspired web framework for Rust.
//!
//! ...

pub mod api;
// ...
```

### Pattern 3: Crate-level //! doc comment block
**What:** Inner doc comments (`//!`) at the top of `lib.rs` document the crate itself. Visible on crates.io and `cargo doc` crate root page.
**When to use:** Every lib.rs that will be published.
**Example:**
```rust
//! # Ferro Broadcast
//!
//! WebSocket broadcasting and real-time channels for the Ferro framework.
//!
//! ## Example
//!
//! ```rust,ignore
//! // usage example
//! ```
```

### Pattern 4: README Template (from decisions)
**What:** Consistent ~30-50 line README across all crates.
**Structure:**
```markdown
# crate-name

One-line description.

## Features

- Feature 1
- Feature 2

## Usage

```rust
// code example (runnable or illustrative per crate)
```

## Documentation

See [docs.ferro-rs.dev](https://docs.ferro-rs.dev) for full documentation.

## License

MIT
```

### Anti-Patterns to Avoid
- **Partial missing_docs fixes:** Adding the lint attribute without fixing all warnings leaves a non-compiling state under `-D warnings`. The decision is all-or-nothing.
- **`#![deny(missing_docs)]`:** Too aggressive per REQUIREMENTS.md. Use `warn` only.
- **Workspace-wide missing_docs:** Apply only to framework crate per decisions. Other crates are out of scope for this phase.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Verifying doc completeness | Manual audit | `cargo rustc --lib -- -W missing-docs` | Compiler finds every undocumented pub item |
| Finding missing Cargo.toml fields | Manual grep | Compare against framework/Cargo.toml pattern | Single reference |
| Checking crates.io category validity | Guessing slugs | Use known slugs already in project | ferro-broadcast, ferro-lang already have valid categories |

---

## Common Pitfalls

### Pitfall 1: missing_docs on Re-exports
**What goes wrong:** `#![warn(missing_docs)]` warns about re-exported items from other crates if those items lack docs at their original definition site. Since framework/src/lib.rs re-exports hundreds of items from ferro-json-ui, ferro-broadcast, etc., there could be cascading warnings.
**Why it happens:** `missing_docs` applies to the public surface of the crate, including re-exports.
**How to avoid:** The actual warning count is known: 136 warnings, all in framework's own source files (not from external crate re-exports). The compiler only warns about items defined in the current crate, not re-exported items. This was verified empirically.
**Warning signs:** If the warning count exceeds 136, something changed in the codebase.

### Pitfall 2: `json_response!` and `text_response!` macros lack docs
**What goes wrong:** Two macros in lib.rs (`json_response!` and `text_response!`) appear in the 136 warnings. They need doc comments before adding the lint attribute.
**How to avoid:** Add `/// ...` doc comments directly above both `#[macro_export]` macros.

### Pitfall 3: homepage field update
**What goes wrong:** framework/Cargo.toml currently has `homepage = "https://github.com/albertogferrario/ferro"`. The decision is to change all homepages to `https://ferro-rs.dev`. This means the framework crate also needs a homepage update, not just the 3 gap crates.
**How to avoid:** Update framework's homepage field as part of META-01 work. The decisions are explicit: `homepage = "https://ferro-rs.dev"` for all crates.

### Pitfall 4: META-04 is already done
**What goes wrong:** Planning effort spent implementing something already complete.
**Why it happens:** CONTEXT.md notes both crates "already have" `//!` comments, but requirements still list META-04 as pending.
**How to avoid:** Plan 02 should verify (not implement) META-04 — a one-line check confirming both lib.rs files have `//!` blocks, then mark requirement satisfied.

### Pitfall 5: ferro-json-ui has most homepage/readme but not all crates do
**What goes wrong:** ferro-json-ui already has `homepage`, `readme`, and `categories` — it's not a META-01 target. ferro-lang has `readme` and `categories` but no `homepage`. ferro-whatsapp has `readme` but no `homepage` or `categories`.
**How to avoid:** Apply changes only to the three named gap crates (ferro-broadcast, ferro-theme, ferro-projections) per META-01. ferro-lang and ferro-whatsapp are README-only targets (META-03), but may also be missing `homepage` — worth a full audit pass.

---

## Code Examples

### Adding missing_docs lint attribute
```rust
// Source: Rust reference — inner attribute placement
// framework/src/lib.rs — first content line
#![warn(missing_docs)]

pub mod api;
// ...
```

### One-liner doc comment for struct fields
```rust
// framework/src/http/cookie.rs
pub struct CookieOptions {
    /// Cookie path scope.
    pub path: Option<String>,
    /// Cookie domain scope.
    pub domain: Option<String>,
    /// Maximum age in seconds.
    pub max_age: Option<i64>,
    // ...
}
```

### Variant-level doc comments
```rust
// framework/src/config/env.rs
pub enum Environment {
    /// Local development environment.
    Local,
    /// Staging/pre-production environment.
    Staging,
    /// Production environment.
    Production,
    // ...
}
```

### Minimal function doc comment
```rust
// One-liner for obviously named functions
/// Creates a new server instance with the given router.
pub fn new(router: impl Into<Router>) -> Self {
```

---

## Current Gap Analysis (Verified)

### META-01: Cargo.toml Metadata Gaps

| Crate | Missing `readme` | Missing `homepage` | Missing `categories` |
|-------|-----------------|-------------------|---------------------|
| ferro-broadcast | YES | YES | no (has `["web-programming", "asynchronous"]`) |
| ferro-theme | YES | YES | YES |
| ferro-projections | YES | YES | YES |
| framework | no | UPDATE (github → ferro-rs.dev) | no |
| ferro-lang | no | YES | no |
| ferro-whatsapp | no | YES | no |

Note: ferro-lang and ferro-whatsapp have homepage gaps too. The decision says "all crates" get `homepage = "https://ferro-rs.dev"`. The planner should include all crates missing homepage, not just the three explicitly named for META-01.

### META-02: missing_docs Warning Distribution

Total warnings: 136 across 19 files in framework/src/

| File | Warning Count | Item Types |
|------|---------------|-----------|
| `validation/rules.rs` | 24 | pub structs (rule types), const fn constructors |
| `routing/macros.rs` | 13 | pub macros re-exported as macro_rules |
| `http/resources/pagination.rs` | 10 | struct fields |
| `inertia/context.rs` | 9 | struct fields |
| `http/cookie.rs` | 9 | enum variants, struct fields |
| `session/driver/database.rs` | 8 | associated functions, struct fields |
| `metrics/mod.rs` | 8 | struct fields, functions |
| `schedule/expression.rs` | 7 | enum variants |
| `debug/mod.rs` | 7 | struct fields |
| `server.rs` | 6 | pub methods |
| `lib.rs` | 6 | crate doc, module docs, macros |
| `http/request.rs` | 4 | associated functions, struct fields |
| `http/response.rs` | 3 | functions |
| `config/mod.rs` | 3 | submodule docs |
| `config/env.rs` | 2 | enum variants |
| `routing/router.rs` | 1 | |
| `middleware/metrics.rs` | 1 | |
| `http/mod.rs` | 1 | |
| `database/config.rs` | 1 | enum variants |

### META-03: README Line Counts

| Crate | Current Lines | Target | Action |
|-------|--------------|--------|--------|
| ferro-json-ui | 9 | 30-50 | Expand |
| ferro-lang | 9 | 30-50 | Expand |
| ferro-whatsapp | 3 | 30-50 | Expand significantly |

### META-04: Crate-level `//!` doc comments

| Crate | Current //! lines | Status |
|-------|------------------|--------|
| ferro-json-ui | 39 | ALREADY DONE |
| ferro-lang | 23 | ALREADY DONE |

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `readme` path = relative | `readme = "README.md"` per crate | Always | Each crate's README is its own file |
| `homepage` not set | `homepage = "https://ferro-rs.dev"` | Phase 114 decision | Consistent crates.io presence |

**Note on framework homepage:** The framework Cargo.toml currently points to the GitHub repo (`https://github.com/albertogferrario/ferro`). Per decisions, this updates to `https://ferro-rs.dev` for consistency.

---

## Open Questions

1. **ferro-lang and ferro-whatsapp homepage gaps**
   - What we know: These crates are META-03 targets (README only), but both lack `homepage` field
   - What's unclear: Whether the decisions intend "all crates" to include these, or only the three META-01 crates
   - Recommendation: Add homepage to all crates missing it in the same Cargo.toml editing pass — one-line change per crate, zero risk

2. **ferro-theme and ferro-projections keywords count**
   - What we know: ferro-theme has 4 keywords ("theme", "tokens", "ferro", "sdui"), ferro-projections has 5
   - What's unclear: Whether to add a 5th keyword to ferro-theme for consistency
   - Recommendation: Planner's discretion — add "css" or "tailwind" as 5th keyword for ferro-theme

---

## Validation Architecture

> workflow.nyquist_validation not set in config.json — treating as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | None — workspace Cargo.toml |
| Quick run command | `cargo test --package ferro-rs` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| META-01 | Cargo.toml has readme/homepage/categories for target crates | manual-only | `grep -r "homepage\|readme\|categories" ferro-broadcast/Cargo.toml ferro-theme/Cargo.toml ferro-projections/Cargo.toml` | ✅ |
| META-02 | framework compiles with `#![warn(missing_docs)]` without new warnings | smoke | `cargo rustc --package ferro-rs --lib -- -W missing-docs 2>&1 \| grep "^warning:" \| wc -l` (must output 0) | ✅ |
| META-03 | READMEs contain meaningful content beyond 9 lines | manual-only | `wc -l ferro-json-ui/README.md ferro-lang/README.md ferro-whatsapp/README.md` | ✅ |
| META-04 | ferro-json-ui and ferro-lang lib.rs have crate-level //! doc comments | manual-only | `grep -c "^//!" ferro-json-ui/src/lib.rs ferro-lang/src/lib.rs` | ✅ Already satisfied |

### Sampling Rate
- **Per task commit:** `cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Per wave merge:** Same — this phase has no unit tests to add, only compilation correctness
- **Phase gate:** `cargo rustc --package ferro-rs --lib -- -W missing-docs 2>&1 | grep "^warning:" | wc -l` must output `0`

### Wave 0 Gaps
None — existing test infrastructure covers all phase requirements. This phase has no test files to create; validation is compilation and line-count checks.

---

## Sources

### Primary (HIGH confidence)
- Empirical: `cargo rustc --package ferro-rs --lib -- -W missing-docs` — 136 warnings across 19 files, list verified
- Empirical: `wc -l` on README files — confirmed 9/9/3 lines
- Empirical: `grep -c "^//!"` on lib.rs files — confirmed META-04 already satisfied
- `framework/Cargo.toml` — reference for complete metadata pattern

### Secondary (MEDIUM confidence)
- crates.io category taxonomy — slugs verified against already-used values in this project

### Tertiary (LOW confidence)
None

---

## Metadata

**Confidence breakdown:**
- Gap analysis: HIGH — directly observed from source files
- missing_docs count: HIGH — compiler-verified with exact file distribution
- Architecture: HIGH — Rust stable patterns, no external dependencies
- Pitfalls: HIGH — empirically verified (META-04 already done, homepage update applies to framework too)

**Research date:** 2026-03-27
**Valid until:** 2026-04-27 (stable domain — cargo metadata conventions don't change)
