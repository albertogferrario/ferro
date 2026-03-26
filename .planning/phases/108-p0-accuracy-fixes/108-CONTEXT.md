# Phase 108: P0 Accuracy Fixes - Context

**Gathered:** 2026-03-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Remove all actively wrong information from user-facing docs: stale `ferro_rs::` import paths, `// TODO: Implement` stubs presented as working code, and false claims about feature status (JSON-UI, S3 storage, MCP tool counts). Scope is docs/src/, README.md, and any doc-adjacent files with factual errors.

</domain>

<decisions>
## Implementation Decisions

### Import path normalization
- Replace all 24 `ferro_rs::` occurrences with `ferro::` across 3 files (multi-tenancy.md, actions.md, data-binding.md)
- Pure grep-replace, no structural changes

### TODO stub replacement
- Replace all 9 TODO stubs in docs/src/reference/cli.md with minimal but real logic
- Use generic names (Item, Resource) not domain-specific (User, Order)
- Each example should have 1-2 lines of real logic — just enough to not be a stub
- Covers: make:controller (2 handlers), make:action, make:listener, make:job, make:migration (up+down), make:task
- middleware.md TODO stub is OUT OF SCOPE — deferred to Phase 113 (Pattern Coherence)

### Storage docs
- Remove "coming soon" note for S3 in docs/src/features/storage.md — S3 is shipped

### MCP tool counts
- Fix all tool count claims across docs to reflect actual count
- Claude decides exact ("57 tools") vs approximate ("50+ tools") per context
- Verify the "5 tools validated" in docs/src/features/api-mcp.md dry-run example against actual ferro-api-mcp behavior — fix if wrong

### README audit
- Full accuracy audit of README.md — fix all factually wrong claims, not just the JSON-UI "Work in Progress" line
- No tone or positioning changes — accuracy only. Agent-first rewrite is Phase 112
- Milestone listing approach at Claude's discretion

### Claude's Discretion
- Migration example table structure (whatever demonstrates the pattern best)
- Exact vs approximate tool count per context
- README milestone listing format (latest + link vs full list)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<code_context>
## Existing Code Insights

### Files to Fix
- `docs/src/features/multi-tenancy.md` — 8 `ferro_rs::` occurrences
- `docs/src/json-ui/actions.md` — 8 `ferro_rs::` occurrences
- `docs/src/json-ui/data-binding.md` — 8 `ferro_rs::` occurrences
- `docs/src/reference/cli.md` — 9 TODO stubs (handlers, middleware, action, listener, job, migration, task)
- `docs/src/features/storage.md` — "coming soon" for S3 at line 285
- `docs/src/features/api-mcp.md` — "5 tools validated" dry-run output to verify
- `README.md` — "Work in Progress" for JSON-UI at line 61, plus full accuracy audit

### Established Patterns
- Docs use mdBook format in docs/src/ with SUMMARY.md navigation
- Code examples in docs use fenced code blocks with `rust` language tag

### Integration Points
- No code changes — docs-only phase
- Success criteria are grep-verifiable (zero `ferro_rs::` in docs/src/, zero `// TODO: Implement` in cli.md)

</code_context>

<deferred>
## Deferred Ideas

- middleware.md TODO stub fix — Phase 113 (Pattern Coherence)

</deferred>

---

*Phase: 108-p0-accuracy-fixes*
*Context gathered: 2026-03-26*
