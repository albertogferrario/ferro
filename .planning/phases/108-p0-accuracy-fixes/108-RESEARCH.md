# Phase 108: P0 Accuracy Fixes - Research

**Researched:** 2026-03-26
**Domain:** Documentation accuracy — mdBook docs, README, grep-replace operations
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Replace all 24 `ferro_rs::` occurrences with `ferro::` across 3 files (multi-tenancy.md, actions.md, data-binding.md) — pure grep-replace, no structural changes
- Replace all 9 TODO stubs in docs/src/reference/cli.md with minimal but real logic; use generic names (Item, Resource) not domain-specific (User, Order); each example should have 1-2 lines of real logic
- Covers: make:controller (2 handlers), make:action, make:listener, make:job, make:migration (up+down), make:task
- middleware.md TODO stub is OUT OF SCOPE — deferred to Phase 113
- Remove "coming soon" note for S3 in docs/src/features/storage.md — S3 is shipped
- Fix all tool count claims across docs to reflect actual count
- Full accuracy audit of README.md — fix all factually wrong claims, not just the JSON-UI "Work in Progress" line
- No tone or positioning changes in README — accuracy only

### Claude's Discretion

- Migration example table structure (whatever demonstrates the pattern best)
- Exact ("57 tools") vs approximate ("50+ tools") per context — NOTE: actual count is 65, not 57 (see Critical Finding below)
- README milestone listing format (latest + link vs full list)

### Deferred Ideas (OUT OF SCOPE)

- middleware.md TODO stub fix — Phase 113 (Pattern Coherence)

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ACC-01 | All `ferro_rs::` import paths in user docs replaced with `ferro::` | 24 occurrences confirmed across 3 files; simple sed-equivalent replacement |
| ACC-02 | All `// TODO: Implement` stubs removed from CLI reference examples | 9 stubs confirmed in cli.md; replacement patterns identified for each generator type |
| ACC-03 | README roadmap section updated — JSON-UI marked as shipped, not "Work in Progress" | ferro-json-ui crate confirmed fully implemented; entire roadmap section needs audit |
| ACC-04 | Storage docs S3 "coming soon" note corrected to reflect shipped status | S3 driver confirmed at ferro-storage/src/drivers/s3.rs; Cargo feature `s3` confirmed |
| ACC-05 | MCP tool count claims updated to reflect actual 57 tools across all docs | CRITICAL: actual count is 65 registered tools, not 57 — no existing count claims in docs to fix, but new accurate count must be used if count is added |

</phase_requirements>

## Summary

Phase 108 is a pure documentation accuracy fix with no code changes. All errors are pre-identified and located. The work falls into four categories: (1) mechanical grep-replace of stale `ferro_rs::` import prefix across 3 files, (2) replacing 9 TODO stub bodies in cli.md with 1-2 lines of minimal real logic, (3) correcting status claims in storage.md and README.md, and (4) establishing the correct MCP tool count.

The critical finding is that the actual registered tool count is **65**, not 57 as stated in REQUIREMENTS.md. There are currently no existing numeric tool count claims in the public-facing docs (the "5 tools validated" in api-mcp.md refers to a user's own API endpoints, not ferro-mcp tools). If the planner adds count claims to docs, use 65.

The api-mcp.md dry-run example showing "5 tools validated" is correct as-is — it shows a user's app with 5 CRUD routes, not a ferro-mcp tool count. No fix needed there.

**Primary recommendation:** Two parallel plan waves: Plan 108-01 handles the mechanical import path replacement (grep-replace, isolated risk), Plan 108-02 handles README + storage + MCP tool count + CLI stub replacement (editorial judgment needed).

## Standard Stack

### Core (docs-only phase — no library dependencies)

| Tool | Purpose | Notes |
|------|---------|-------|
| mdBook | Ferro user docs format | Files in `docs/src/`, fenced code blocks use `rust` language tag |
| Standard text editor / Write tool | File modifications | All changes are plain text edits |

### No new packages required

This phase has zero code changes. All work is editing existing `.md` files and `README.md`.

## Architecture Patterns

### Recommended File Edit Approach

For **import path replacement** (ACC-01): pure find-and-replace within fenced code blocks.
- Pattern: `ferro_rs::` → `ferro::`
- Scope: only inside ` ```rust ` code blocks (all occurrences in the 3 files happen to be in code blocks)
- No risk of prose false positives — verify with grep after edit

For **TODO stub replacement** (ACC-02): replace the stub comment + `Ok(())` with a minimal working body.
- Keep the same function signature and return type
- Add 1-2 lines of logic showing idiomatic usage
- Use generic types: `Item`, `Resource`, not domain models

For **status corrections** (ACC-03, ACC-04): targeted line edits, not section rewrites.

### Files to Modify (complete list)

| File | Change | Scope |
|------|--------|-------|
| `docs/src/features/multi-tenancy.md` | Replace 8 `ferro_rs::` → `ferro::` | Code blocks only |
| `docs/src/json-ui/actions.md` | Replace 8 `ferro_rs::` → `ferro::` | Code blocks only |
| `docs/src/json-ui/data-binding.md` | Replace 8 `ferro_rs::` → `ferro::` | Code blocks only |
| `docs/src/reference/cli.md` | Replace 9 TODO stubs with real logic | Lines 181, 187, 238, 318, 348, 399, 404, 511 (and the middleware stub at 214 is OUT OF SCOPE per decisions) |
| `docs/src/features/storage.md` | Remove "coming soon" note for S3 at line 285 | Single line / short section |
| `README.md` | Full accuracy audit; remove "Work in Progress" + roadmap section review | README lines 59-88 minimum |

### Anti-Patterns to Avoid

- **Do not alter prose text** in multi-tenancy.md, actions.md, or data-binding.md — only fix the `use ferro_rs::` lines in code blocks
- **Do not change tone or positioning in README** — accuracy only; no marketing rewrite
- **Do not replace the middleware.md stub** — explicitly deferred to Phase 113
- **Do not invent domain logic** in CLI stubs — keep examples generic and minimal
- **Do not remove the api-mcp.md "5 tools validated" example** — it is correct (shows a user's 5-route API, not ferro-mcp tool count)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Verifying all `ferro_rs::` replaced | Manual inspection | `grep -rn "ferro_rs::" docs/src/` | Grep is the success criterion per CONTEXT.md |
| Verifying all TODO stubs removed from cli.md | Manual inspection | `grep -n "// TODO: Implement" docs/src/reference/cli.md` | Same pattern as ACC-01 verification |
| Counting actual MCP tools | Manual reading | `grep -c "#\[tool(" ferro-mcp/src/service.rs` | Authoritative source is service.rs registrations |

## Common Pitfalls

### Pitfall 1: Crate Badge Still Says ferro-rs
**What goes wrong:** README badge links to `crates.io/crates/ferro-rs` and may reference the wrong crate name in text
**Why it happens:** The crate was renamed from ferro-rs to ferro
**How to avoid:** Audit badge URLs and any crate name references in README during the accuracy audit
**Warning signs:** `ferro-rs` appearing in prose (not in historical context)

### Pitfall 2: Wrong Tool Count
**What goes wrong:** Using "57 tools" (from REQUIREMENTS.md) instead of the actual registered count
**Why it happens:** REQUIREMENTS.md was written before service.rs was counted — actual count is 65
**How to avoid:** Use 65 when adding any numeric count claim; or use "50+ tools" / "65 tools" depending on context
**Warning signs:** Any reference to "57 tools" in output

### Pitfall 3: Over-Scoping README Changes
**What goes wrong:** Rewriting README tone, adding new sections, or changing value proposition while fixing accuracy
**Why it happens:** It is tempting to improve while fixing
**How to avoid:** CONTEXT.md is explicit: accuracy only, no tone changes; agent-first rewrite is Phase 112
**Warning signs:** Adding sentences that weren't there before (only deletions and targeted corrections are safe)

### Pitfall 4: CLI Stub Logic Is Too Minimal or Too Complex
**What goes wrong:** Replacing `// TODO: Implement` with just `Ok(())` (too minimal — still looks like a stub) or with 10 lines of real business logic (too complex — not what the docs show)
**Why it happens:** Ambiguity in "1-2 lines of real logic"
**How to avoid:** Show the idiomatic pattern for each type — e.g., for handlers: return a JSON response; for migrations: create or alter a table; for jobs: log + return Ok(())
**Warning signs:** Any `todo!()` macro or empty implementations remain

### Pitfall 5: Accidentally Fixing the middleware.md Stub
**What goes wrong:** Fixing the `// TODO: Implement middleware logic` in docs/src/the-basics/middleware.md
**Why it happens:** grep for TODO stubs finds it; it looks like the same problem
**How to avoid:** middleware.md is explicitly OUT OF SCOPE per CONTEXT.md
**Warning signs:** Editing `docs/src/the-basics/middleware.md`

## Code Examples

Verified patterns from source inspection:

### ACC-01: Import Path Before and After
```rust
// BEFORE (wrong)
use ferro_rs::{handler, TenantContext, Response, json};

// AFTER (correct)
use ferro::{handler, TenantContext, Response, json};
```

### ACC-02: Controller Handler Stub Replacement

The make:controller example currently shows:
```rust
// BEFORE
#[handler]
pub async fn index(req: Request) -> Response {
    // TODO: Implement
    json_response!({ "message": "index" })
}
```

The stub body `// TODO: Implement` should be removed — the `json_response!` line below it is real logic. The two-line pattern in cli.md (line 181-182, 187-188) has:
- Line 181: `    // TODO: Implement`
- Line 182: `    json_response!({ "message": "index" })`

The fix is simply deleting the `// TODO: Implement` comment lines (181 and 187), since the actual return statement is already present and correct.

### ACC-02: Action Stub Replacement

```rust
// BEFORE
impl CreateOrder {
    pub async fn execute(&self) -> Result<(), FrameworkError> {
        // TODO: Implement action
        Ok(())
    }
}

// AFTER — show minimal real logic (1 line)
impl Resource {
    pub async fn execute(&self) -> Result<(), FrameworkError> {
        tracing::info!("executing resource action");
        Ok(())
    }
}
```

### ACC-02: Listener Stub Replacement

```rust
// BEFORE
async fn handle(&self, event: &E) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement listener
    Ok(())
}

// AFTER
async fn handle(&self, event: &E) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("handling event");
    Ok(())
}
```

### ACC-02: Job Stub Replacement

```rust
// BEFORE
async fn handle(&self, ctx: &JobContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement job
    Ok(())
}

// AFTER
async fn handle(&self, ctx: &JobContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(id = self.image_id, "processing item");
    Ok(())
}
```

### ACC-02: Migration Stub Replacement (up + down)

```rust
// BEFORE
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // TODO: Implement migration
    Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // TODO: Implement rollback
    Ok(())
}

// AFTER — demonstrate table creation pattern
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Item::Table)
                .if_not_exists()
                .col(ColumnDef::new(Item::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(Item::CreatedAt).timestamp().not_null())
                .to_owned(),
        )
        .await
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .drop_table(Table::drop().table(Item::Table).to_owned())
        .await
}
```

### ACC-02: Task Stub Replacement

```rust
// BEFORE
async fn handle(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement task
    Ok(())
}

// AFTER
async fn handle(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("running scheduled task");
    Ok(())
}
```

### ACC-04: S3 Storage Section Fix

```markdown
<!-- BEFORE (line 285 area) -->
### S3 Driver

Requires the `s3` feature (coming soon):

```toml
[dependencies]
ferro = { version = "0.1", features = ["s3"] }
```

<!-- AFTER -->
### S3 Driver

Enable the `s3` feature:

```toml
[dependencies]
ferro = { version = "0.1", features = ["s3"] }
```
```

## Actual Counts (Verified from Source)

| Item | Count | Source | Notes |
|------|-------|--------|-------|
| `ferro_rs::` in docs/src/ | **24** | grep output | 8 per file × 3 files |
| TODO stubs in cli.md (in-scope) | **9** | grep output | 8 with body + 1 middleware (OUT OF SCOPE) |
| Registered MCP tools (ferro-mcp) | **65** | `#[tool(` in service.rs | REQUIREMENTS said 57 — service.rs is authoritative |
| Storage S3 driver | **Shipped** | ferro-storage/src/drivers/s3.rs exists | "coming soon" is wrong |
| JSON-UI | **Shipped** | ferro-json-ui crate fully implemented | README "Work in Progress" is wrong |

### Critical Finding: MCP Tool Count Discrepancy

REQUIREMENTS.md says 57 tools. The actual count in `ferro-mcp/src/service.rs` is **65** registered tools (65 `#[tool(` declarations). The `mod.rs` lists 56 tool modules, but some modules contain multiple tools (e.g., stripe.rs has 3 tools, whatsapp.rs has 2 tools, crud_operations.rs has 4 tools, ai.rs has 2 tools).

**Impact on ACC-05:** There are currently **no** numeric tool count claims in the public docs — neither 57 nor 65 appears anywhere in `docs/src/`. If the planner adds a count, use 65 (or "60+" for a durable approximate claim). Do NOT use 57.

## State of the Art

| Old State | Current State | Impact |
|-----------|---------------|--------|
| `ferro_rs` import prefix in 3 doc files | Correct crate name is `ferro` | ACC-01 fix needed |
| 9 TODO stubs in cli.md examples | Should show minimal real patterns | ACC-02 fix needed |
| README marks JSON-UI as "Work in Progress" | ferro-json-ui is a complete, shipped crate | ACC-03 fix needed |
| S3 labeled "coming soon" in storage.md | `ferro-storage/src/drivers/s3.rs` exists, `s3` feature enabled | ACC-04 fix needed |
| No tool count in public docs | 65 tools registered in service.rs | ACC-05: add count if referenced |

## Open Questions

1. **Migration stub enum type**
   - What we know: The migration stub uses `// TODO: Implement migration` + `Ok(())`; SeaORM migrations use an `Iden` enum for table/column identifiers
   - What's unclear: Should the replacement example define an `Item` enum inline (realistic) or use a minimal one-liner approach (simpler)?
   - Recommendation: Define a minimal `Item` enum + 2-column table creation to show the real pattern; this is the only stub that requires structural additions beyond a single `tracing::info!` call

2. **README roadmap section scope**
   - What we know: At minimum, line 61 (`### 🚧 JSON-UI (Work in Progress)`) must be updated; CONTEXT says full accuracy audit
   - What's unclear: Whether the entire JSON-UI roadmap block (lines 61-88) should be removed, replaced with a "shipped" callout, or rewritten
   - Recommendation: Remove the `🚧` marker and "Work in Progress" heading; either convert to a brief description of what JSON-UI does (shipped) or remove the section if the planner judges it redundant with docs link

3. **Exact vs approximate tool count wording**
   - What we know: 65 tools registered; CONTEXT gives Claude discretion on "57 tools" vs "50+ tools" per context
   - What's unclear: Whether "57 tools" (from REQUIREMENTS) should be corrected to "65 tools" or "60+ tools" in any doc that quotes it — but currently no docs quote a number
   - Recommendation: If adding a count anywhere, use exact "65 tools"; if the claim needs to survive small additions without doc updates, use "60+ tools"

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | grep (shell) — no Rust test suite for docs |
| Config file | none |
| Quick run command | `grep -rn "ferro_rs::" docs/src/` (should return 0 lines) |
| Full suite command | See Phase Gate below |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Exists? |
|--------|----------|-----------|-------------------|---------|
| ACC-01 | Zero `ferro_rs::` in docs/src/ | smoke | `grep -rn "ferro_rs::" docs/src/` → no output | Run after edit |
| ACC-02 | Zero `// TODO: Implement` in cli.md | smoke | `grep -n "// TODO: Implement" docs/src/reference/cli.md` → no output | Run after edit |
| ACC-03 | "Work in Progress" not in README roadmap | smoke | `grep -n "Work in Progress" README.md` → no output | Run after edit |
| ACC-04 | "coming soon" not in storage.md | smoke | `grep -n "coming soon" docs/src/features/storage.md` → no output | Run after edit |
| ACC-05 | No incorrect tool count in docs | manual | Verify any added count claim matches service.rs (`grep -c "#\[tool(" ferro-mcp/src/service.rs` → 65) | Run after edit |

### Sampling Rate

- **Per task commit:** Run the smoke grep for that task's requirement
- **Per wave merge:** Run all 5 smoke greps
- **Phase gate:** All 5 smoke greps return zero matches before `/gsd:verify-work`

### Wave 0 Gaps

None — existing shell grep infrastructure covers all phase requirements. No test files need creation.

## Sources

### Primary (HIGH confidence)

- Direct file inspection: `docs/src/features/multi-tenancy.md` — 8 `ferro_rs::` occurrences confirmed
- Direct file inspection: `docs/src/json-ui/actions.md` — 8 `ferro_rs::` occurrences confirmed
- Direct file inspection: `docs/src/json-ui/data-binding.md` — 8 `ferro_rs::` occurrences confirmed
- Direct file inspection: `docs/src/reference/cli.md` — 9 `// TODO: Implement` stubs confirmed at lines 181, 187, 214 (OUT OF SCOPE), 238, 318, 348, 399, 404, 511
- Direct file inspection: `docs/src/features/storage.md` line 285 — "coming soon" confirmed
- Direct file inspection: `README.md` line 61 — "Work in Progress" confirmed
- Direct file inspection: `ferro-mcp/src/service.rs` — 65 `#[tool(` declarations (authoritative tool count)
- Direct file inspection: `ferro-storage/src/drivers/s3.rs` — S3 driver exists (feature shipped)
- Direct file inspection: `ferro-json-ui/src/` — full JSON-UI crate implementation present

### Secondary (MEDIUM confidence)

- `ferro-mcp/src/tools/mod.rs` — 56 `pub mod` declarations (some modules register multiple tools)
- `ferro-storage/Cargo.toml` — `s3 = ["aws-sdk-s3", "aws-config"]` feature confirmed

## Metadata

**Confidence breakdown:**
- Import path occurrences: HIGH — grep-verified exact counts
- TODO stub locations: HIGH — grep-verified with line numbers
- MCP tool count (65): HIGH — counted `#[tool(` declarations in service.rs directly
- S3 shipped status: HIGH — driver file exists and feature declared
- JSON-UI shipped status: HIGH — crate directory fully populated
- README audit scope: MEDIUM — full audit content depends on what planner finds during edit

**Research date:** 2026-03-26
**Valid until:** 2026-04-25 (stable docs — changes only if service.rs gains/loses tools)
