# Phase 113: Pattern Coherence - Context

**Gathered:** 2026-03-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Standardize all code examples across docs to use a single consistent import style, ensure all handler examples use `#[handler]`, fix error propagation to use `?` instead of `.unwrap()`, and resolve the COMPONENT_CATALOG duplication between ferro-cli and ferro-mcp. No new framework features — pattern consistency only.

</domain>

<decisions>
## Implementation Decisions

### Import style
- All code examples use explicit imports from crate root: `use ferro::{Request, Response, Router};`
- No glob imports (`use ferro::*`) — replace all 37 occurrences across 6 files
- No sub-module paths (`use ferro::validation::Validator`) — crate root only, per Phase 110 rule
- Imports always visible in examples (no `# use` hidden lines)
- components.md has 28 glob imports to convert — per-component import lists at Claude's discretion

### Handler macro
- Every handler function example in docs gets `#[handler]` — no exceptions
- Non-handler functions (routes, services, policies) — Claude audits and fixes inconsistencies at discretion
- Return type style (`Response` vs full type) at Claude's discretion

### Error propagation
- Replace `.unwrap()` with `?` where appropriate — Claude's discretion on edge cases
- 32 occurrences across 10 files to audit
- Pragmatic approach: fix what needs fixing, leave infallible operations if clearly safe

### COMPONENT_CATALOG resolution
- Move to `ferro-json-ui` as `pub const COMPONENT_CATALOG: &str` (plain string, not structured type)
- Both ferro-cli and ferro-mcp already depend on ferro-json-ui — no new dependencies
- Remove duplicate definitions from `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs`
- Exact location in ferro-json-ui at Claude's discretion (lib.rs vs catalog.rs module)
- Record design decision resolution in PROJECT.md (updates the "Revisit" marker)

### Claude's Discretion
- Per-component import lists in components.md (exact imports per example)
- Non-handler macro consistency (services, policies, etc.)
- Return type presentation style
- unwrap() edge case judgment (infallible operations, test contexts)
- COMPONENT_CATALOG module location in ferro-json-ui

</decisions>

<specifics>
## Specific Ideas

- Phase 110 established: all ferro imports use explicit crate-root exports — this phase enforces that rule in all doc examples
- COMPONENT_CATALOG deduplication closes the "Revisit" marker in PROJECT.md Key Decisions table

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui` crate: both ferro-cli and ferro-mcp depend on it — natural home for shared COMPONENT_CATALOG
- mdBook format in `docs/src/` — `# ` prefix hides lines but we're showing imports visibly

### Established Patterns
- Phase 110 rule: explicit crate-root exports only (no ferro::prelude, no module paths)
- Phase 108 already fixed `ferro_rs::` → `ferro::` — this phase standardizes the import style within `ferro::`
- `#[handler]` macro is the standard handler pattern — 76 existing usages across 21 files

### Integration Points
- 37 files with `use ferro::` imports need audit
- 6 files with glob imports need conversion to explicit
- 10 files with `.unwrap()` need audit
- `ferro-cli/src/ai.rs` and `ferro-mcp/src/tools/json_ui_generate.rs` — COMPONENT_CATALOG source and target
- `ferro-json-ui/src/lib.rs` or new module — COMPONENT_CATALOG destination

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 113-pattern-coherence*
*Context gathered: 2026-03-27*
