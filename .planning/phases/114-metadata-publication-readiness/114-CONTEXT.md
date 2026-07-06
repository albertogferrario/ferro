# Phase 114: Metadata & Publication Readiness - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Make all crates publication-ready with complete Cargo.toml metadata, crate-level doc comments, and expanded READMEs. Targets: ferro-broadcast, ferro-theme, ferro-projections (Cargo.toml gaps), ferro-json-ui, ferro-lang, ferro-whatsapp (README stubs), framework crate (`#![warn(missing_docs)]`).

</domain>

<decisions>
## Implementation Decisions

### README depth & structure
- Concise overview format: ~30-50 lines per README
- Consistent template across all three crates: title, description, features list, usage example, link to docs, license
- Links point to hosted docs site (docs.ferro-rs.dev) for detailed documentation
- Code examples: Claude's discretion on runnable vs illustrative per crate

### missing_docs strategy
- Add `#![warn(missing_docs)]` to framework crate and fix ALL warnings — no partial state
- Fix all warnings even if it touches 50+ files — publication-ready means clean build
- Doc comment style: Claude's discretion — one-liners for obvious items, more detail for complex APIs

### Cargo.toml metadata
- `homepage` = `https://ferro-rs.dev` for all crates (domain redirects to docs now, future landing page)
- `readme` = `"README.md"` pointing to each crate's own README
- `categories`: Claude's discretion based on crates.io conventions
- Fill missing fields: `readme`, `homepage`, `categories` on ferro-broadcast, ferro-theme, ferro-projections

### Claude's Discretion
- Code example style per README (runnable vs illustrative)
- Doc comment depth per undocumented item
- Category selection for crates.io

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- framework/Cargo.toml: reference for complete metadata (has readme, homepage, categories, keywords)
- ferro-json-ui/src/lib.rs: already has comprehensive `//!` crate-level doc comments (META-04 partially satisfied)
- ferro-lang/src/lib.rs: already has `//!` crate-level doc comments (META-04 partially satisfied)

### Established Patterns
- Workspace inherits: `version.workspace = true`, `edition.workspace = true`, `license.workspace = true`
- Repository URL: `https://github.com/albertogferrario/ferro` (consistent across all crates)
- Keywords: 5 keywords per crate, last one always "ferro"

### Integration Points
- Cargo.toml changes: no code impact, metadata only
- `#![warn(missing_docs)]`: added to framework/src/lib.rs, may cascade warnings across all pub items
- README expansion: standalone files, no code integration

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 114-metadata-publication-readiness*
*Context gathered: 2026-03-27*
