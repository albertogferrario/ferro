# Phase 142: ferro-mcp Parity - Research

**Researched:** 2026-04-20
**Domain:** ferro-mcp Stripe introspection tools — regex scanning, struct mutation, test fixture updates
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Drop hard-coded `src/stripe/listeners.rs` path. Walk all `.rs` files under `src/`.
**D-02:** Primary pattern `\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|`; secondary pattern `\.on::<(\w+)` for turbofish.
**D-03:** `WebhookEventInfo` removes `listener: String`, keeps `event_type` and `file`, adds `line: u32`.
**D-04:** `StripeWebhookEvents` wrapper unchanged.
**D-05:** No matches → return empty `events` vec.
**D-06:** Continue scanning `src/stripe/` for config status.
**D-07:** Add four boolean fields to `StripeConfigStatus`: `checkout_exists`, `refund_exists`, `account_exists`, `webhook_dir_exists`.
**D-08:** `scaffold_exists` and `scaffold_files` remain; `scaffold_files` listing stays flat (non-recursive).
**D-09:** Update `stripe_config_status` MCP tool description to mention capability-axis layout detection.
**D-10:** Retain `stripe_subscription_info` logic unchanged.
**D-11:** Update `stripe_subscription_info` tool description — clarifies the tool scans app-level billing migrations, not the framework subscription module.
**D-12:** No struct changes to `StripeSubscriptionInfo`, `ColumnInfo`, `IndexInfo`.
**D-13:** `stripe_webhook_events` description updated to describe SyncDispatcher handler scanning.
**D-14:** `stripe_config_status` description updated to mention capability-axis layout fields.
**D-15:** `stripe_subscription_info` description updated to clarify app-level billing table.
**D-16:** Update `test_webhook_events_parses_listeners` fixture to use closure syntax; assert `event_type` and `line`.
**D-17:** Add test for turbofish pattern `.on::<StripeCheckoutCompleted, _, _>(handler)`.
**D-18:** Add `test_config_status_capability_axis_fields`.
**D-19:** Remove or update `test_webhook_events_serializes` to drop `listener` field reference.
**D-20:** Workspace version 0.2.2 → 0.2.3.

### Claude's Discretion

- Whether to keep `listener: Option<String>` for backward compat or hard-remove. Recommended: hard-remove (feature branch).
- File-walk implementation: `walkdir` (already in workspace) or recursive `fs::read_dir`. Recommended: `walkdir` (already used throughout ferro-mcp).
- Whether `scaffold_files` listing becomes recursive into `webhook/`. Recommended: keep flat; use `webhook_dir_exists` for directory signal.

### Deferred Ideas (OUT OF SCOPE)

- Scanning for `Arc<SyncDispatcher>` provider registration.
- Per-file handler count or coverage metrics.
- `stripe_subscription_info` retirement — revisit at v1.0 audit.
</user_constraints>

---

## Summary

Phase 142 is a focused update to `ferro-mcp/src/tools/stripe.rs` and the three MCP tool description strings in `ferro-mcp/src/service.rs`. Phase 141 replaced the `impl Listener<EventType> for Struct` pattern with anonymous closures registered via `SyncDispatcher::on(|event: EventType| ...)`. The MCP introspection layer still scans for the old pattern, making it blind to the new architecture.

The work decomposes into three independent changes that can be planned and executed as separate tasks: (1) update `stripe_webhook_events` — new regex, new struct shape, file-tree walk; (2) update `stripe_config_status` — four new boolean fields, one test; (3) update all three tool descriptions in `service.rs` and bump the workspace version. The test surface is already established with `tempfile::TempDir` and the `cargo test -- stripe` suite confirms 10 tests passing against the old shape.

**Primary recommendation:** Use `walkdir` for the `src/` tree walk in `stripe_webhook_events` — it is already a direct dependency in `ferro-mcp/Cargo.toml` and is the idiomatic pattern used by every other tool in the crate that needs recursive directory traversal.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Webhook handler discovery | ferro-mcp tool | — | Static source analysis; reads app source, no runtime involvement |
| Scaffold layout detection | ferro-mcp tool | — | File-system existence checks on app source tree |
| MCP tool description strings | ferro-mcp service layer | — | `#[tool(description = ...)]` attributes in `service.rs` |
| JSON schema for changed structs | rmcp proc-macro | — | Schema regenerated automatically at compile time when struct signatures change |
| Workspace version bump | Cargo.toml root | — | Single field change; covers all crates including ferro-mcp |

---

## Standard Stack

### Core (already in use — no new deps required)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `walkdir` | 2.x | Recursive `.rs` file traversal under `src/` | [VERIFIED: ferro-mcp/Cargo.toml] Already a direct dep; used in 15+ ferro-mcp files |
| `regex` | 1.x | Pattern matching for `.on(...)` closure forms | [VERIFIED: ferro-mcp/Cargo.toml] Already in use in `stripe.rs` |
| `serde` + `serde_json` | 1.x | Struct serialization to MCP JSON responses | [VERIFIED: ferro-mcp/Cargo.toml] Already in use |
| `tempfile` | 3.x | TempDir in tests | [VERIFIED: ferro-mcp/Cargo.toml dev-dependencies] |

**No new dependencies required for this phase.** All needed crates are already declared.

---

## Architecture Patterns

### System Architecture Diagram

```
App source (src/**/*.rs)
        │
        ▼
stripe_webhook_events()
  ├── WalkDir::new(project_root/src/)
  │     └── filter .rs extensions
  │           └── fs::read_to_string()
  │                 └── Regex captures_iter()
  │                       ├── pattern 1: .on(|ident: EventType| ...)
  │                       └── pattern 2: .on::<EventType (turbofish)
  │                             └── line number: content[..match.start()].lines().count() + 1
  └── Vec<WebhookEventInfo> { event_type, file, line }

App source (src/stripe/)
        │
        ▼
stripe_config_status()
  ├── dotenvy env-var scanning (unchanged)
  ├── src/stripe/ dir check (scaffold_exists, scaffold_files — unchanged, flat)
  └── four new is_file()/is_dir() checks:
        ├── src/stripe/checkout.rs  → checkout_exists
        ├── src/stripe/refund.rs    → refund_exists
        ├── src/stripe/account.rs   → account_exists
        └── src/stripe/webhook/     → webhook_dir_exists

service.rs #[tool(...)] attributes
        │
        └── description strings updated for all three tools
```

### Recommended Project Structure (no changes)

The phase touches exactly two files:

```
ferro-mcp/
├── src/
│   ├── tools/
│   │   └── stripe.rs      ← primary target: struct changes + logic changes + test updates
│   └── service.rs         ← secondary target: three description strings updated
Cargo.toml                 ← workspace version bump only
```

### Pattern 1: WalkDir for Source Tree Traversal

**What:** Recursively walk `src/` for `.rs` files, read content, apply regex.
**When to use:** Any `ferro-mcp` tool that must discover patterns across arbitrary app source files.
**Example:**
```rust
// Source: ferro-mcp/src/introspection/events.rs lines 113-137
for entry in WalkDir::new(project_root.join("src"))
    .into_iter()
    .filter_map(|e| e.ok())
    .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
{
    let content = match fs::read_to_string(entry.path()) {
        Ok(c) => c,
        Err(_) => continue,
    };
    let relative = entry
        .path()
        .strip_prefix(project_root)
        .unwrap_or(entry.path())
        .to_string_lossy()
        .to_string();
    // ... regex work on `content`
}
```
[VERIFIED: ferro-mcp/src/introspection/events.rs]

### Pattern 2: Regex Capture with Line Number

**What:** Compute 1-based line number from byte offset of a regex match.
**When to use:** Any scan that needs to report source location alongside matched content.
**Example:**
```rust
// Source: CONTEXT.md code_context section
let re = Regex::new(r"\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|").unwrap();
for cap in re.captures_iter(&content) {
    let byte_offset = cap.get(0).unwrap().start();
    let line = content[..byte_offset].lines().count() + 1;
    // cap[1] is the EventType name
}
```
[ASSUMED — line number computation formula; standard Rust pattern]

### Pattern 3: Boolean Field Additions to Existing Struct

**What:** Add new fields to a `#[derive(Serialize)]` struct without breaking existing serialization consumers.
**When to use:** Extending a response struct with additive information.
**Example:**
```rust
// New fields on StripeConfigStatus (D-07):
pub checkout_exists: bool,
pub refund_exists: bool,
pub account_exists: bool,
pub webhook_dir_exists: bool,
```
Existing fields (`configured`, `keys_present`, `keys_missing`, `scaffold_exists`, `scaffold_files`) are unchanged. JSON consumers that don't read the new fields are unaffected.
[VERIFIED: ferro-mcp/src/tools/stripe.rs current struct definition]

### Pattern 4: Capability-Axis Boolean Checks

**What:** Simple `Path::is_file()` / `Path::is_dir()` checks for scaffold file existence.
**Example:**
```rust
let stripe_dir = project_root.join("src/stripe");
let checkout_exists = stripe_dir.join("checkout.rs").is_file();
let refund_exists = stripe_dir.join("refund.rs").is_file();
let account_exists = stripe_dir.join("account.rs").is_file();
let webhook_dir_exists = stripe_dir.join("webhook").is_dir();
```
[VERIFIED: consistent with existing `scaffold_exists` check in stripe.rs lines 71-72]

### Anti-Patterns to Avoid

- **Hard-coding `src/stripe/listeners.rs`:** The old pattern only scanned a single file. The new walker must cover any `.rs` file under `src/` — handlers may be registered in `src/stripe/checkout.rs`, `src/stripe/webhook/mod.rs`, or anywhere the user wires the dispatcher.
- **Using `fs::read_dir` recursively:** Prefer `WalkDir` which is idiomatic in this codebase and handles nested directories (e.g., `src/stripe/webhook/`) transparently.
- **Unconsolidated regexes per call:** Compile `Regex` with `Regex::new(...).unwrap()` once per function call (no lazy_static needed for this use case — the hot path is I/O, not regex compilation). [ASSUMED]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recursive `.rs` file walk | Manual recursive `fs::read_dir` | `walkdir::WalkDir` | Already a dep; idiomatic in codebase; handles symlinks, depth limits |
| Line-number from byte offset | Custom iterator | `content[..offset].lines().count() + 1` | One-liner; no dep needed |
| Temp dir in tests | Manual `std::fs::create_dir_all` teardown | `tempfile::TempDir` | Already in dev-deps; auto-cleanup on drop |

---

## Common Pitfalls

### Pitfall 1: Regex Misses Turbofish Form

**What goes wrong:** The primary regex `\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|` only matches closure syntax. Apps using turbofish `.on::<StripeCheckoutCompleted, _, _>(handler)` are silently missed.
**Why it happens:** Two distinct call forms exist in the SyncDispatcher API.
**How to avoid:** Apply both regexes to every file. D-02 specifies the secondary pattern `\.on::<(\w+)` for turbofish.
**Warning signs:** Test D-17 must cover turbofish; if only D-16's closure fixture passes, turbofish is missing.

### Pitfall 2: Old Test References `listener` Field

**What goes wrong:** `test_webhook_events_serializes` constructs a `WebhookEventInfo` with `listener: "SyncSubscriptionPlan".to_string()`. After D-03 removes the field, this test fails to compile.
**Why it happens:** Hard-coded struct literal includes the removed field.
**How to avoid:** D-19 mandates removing or updating this test. Must be done in the same task as the struct change.

### Pitfall 3: `scaffold_files` Inadvertently Picks Up Webhook Subdir Files

**What goes wrong:** If `scaffold_files` listing is changed to recursive, it would also list `src/stripe/webhook/mod.rs` etc., changing the existing behavior.
**Why it happens:** Adding `webhook_dir_exists` might tempt a recursive scan.
**How to avoid:** D-08 explicitly mandates keeping `scaffold_files` flat (non-recursive `fs::read_dir` on `src/stripe/` only). Use `webhook_dir_exists` for the directory signal.

### Pitfall 4: Line Number Off-by-One

**What goes wrong:** Line numbers are reported as 0-based when users expect 1-based.
**Why it happens:** `str.lines().count()` on the prefix before the match returns the number of newlines, which gives a 0-based index.
**How to avoid:** Use `content[..byte_offset].lines().count() + 1` (the `+ 1` converts to 1-based).

### Pitfall 5: JSON Schema Not Regenerated

**What goes wrong:** rmcp proc-macro generates the JSON schema at compile time. Struct changes require a clean rebuild.
**Why it happens:** Incremental builds may not re-run proc-macros if only the struct layout changes.
**How to avoid:** Run `cargo build` after struct changes; CI will catch schema drift. Per CONTEXT.md code_context: "JSON schema is regenerated automatically when the rmcp proc-macro compiles the updated struct/method signatures."
[VERIFIED: ferro-mcp/src/service.rs MCP tool registration pattern]

---

## Code Examples

### stripe_webhook_events — Updated Function Skeleton

```rust
// Source: CONTEXT.md decisions D-01 through D-05 + code_context
pub fn stripe_webhook_events(project_root: &Path) -> StripeWebhookEvents {
    let src_dir = project_root.join("src");
    if !src_dir.is_dir() {
        return StripeWebhookEvents { events: Vec::new() };
    }

    // Primary: closure form  .on(|ident: EventType| ...)
    let re_closure = Regex::new(r"\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|").unwrap();
    // Secondary: turbofish form  .on::<EventType
    let re_turbofish = Regex::new(r"\.on::<(\w+)").unwrap();

    let mut events = Vec::new();

    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let relative = entry
            .path()
            .strip_prefix(project_root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .to_string();

        for cap in re_closure.captures_iter(&content) {
            let byte_offset = cap.get(0).unwrap().start();
            let line = (content[..byte_offset].lines().count() + 1) as u32;
            events.push(WebhookEventInfo {
                event_type: cap[1].to_string(),
                file: relative.clone(),
                line,
            });
        }

        for cap in re_turbofish.captures_iter(&content) {
            let byte_offset = cap.get(0).unwrap().start();
            let line = (content[..byte_offset].lines().count() + 1) as u32;
            events.push(WebhookEventInfo {
                event_type: cap[1].to_string(),
                file: relative.clone(),
                line,
            });
        }
    }

    StripeWebhookEvents { events }
}
```

### WebhookEventInfo — Updated Struct

```rust
// Source: CONTEXT.md D-03
#[derive(Debug, Serialize)]
pub struct WebhookEventInfo {
    /// The event type name (e.g., "StripeSubscriptionUpdated").
    pub event_type: String,
    /// Relative file path where the handler registration appears.
    pub file: String,
    /// 1-based line number of the `.on(...)` call.
    pub line: u32,
}
```

### stripe_config_status — Capability-Axis Fields

```rust
// Source: CONTEXT.md D-07 + D-08
#[derive(Debug, Serialize)]
pub struct StripeConfigStatus {
    pub configured: bool,
    pub keys_present: Vec<String>,
    pub keys_missing: Vec<String>,
    pub scaffold_exists: bool,
    pub scaffold_files: Vec<String>,
    // New: capability-axis layout detection
    pub checkout_exists: bool,
    pub refund_exists: bool,
    pub account_exists: bool,
    pub webhook_dir_exists: bool,
}

// In stripe_config_status() after existing scaffold_files computation:
let checkout_exists = scaffold_dir.join("checkout.rs").is_file();
let refund_exists = scaffold_dir.join("refund.rs").is_file();
let account_exists = scaffold_dir.join("account.rs").is_file();
let webhook_dir_exists = scaffold_dir.join("webhook").is_dir();
```

### Test Fixture: closure syntax (D-16)

```rust
// Replaces the old impl Listener<...> for ... fixture
let content = r#"
use ferro_stripe::{SyncDispatcher, StripeSubscriptionUpdated};
use std::sync::Arc;

let dispatcher = SyncDispatcher::new()
    .on(|event: StripeSubscriptionUpdated| async move {
        Ok(())
    });
"#;
// Write to any path under src/ (not necessarily src/stripe/listeners.rs)
fs::create_dir_all(tmp.path().join("src/stripe")).unwrap();
fs::write(tmp.path().join("src/stripe/mod.rs"), content).unwrap();

let result = stripe_webhook_events(tmp.path());
assert_eq!(result.events.len(), 1);
assert_eq!(result.events[0].event_type, "StripeSubscriptionUpdated");
assert!(result.events[0].line > 0);
```

### Test: turbofish pattern (D-17)

```rust
let content = r#"
dispatcher.on::<StripeCheckoutCompleted, _, _>(handler)
"#;
// ... write, scan, assert event_type == "StripeCheckoutCompleted"
```

### MCP Description Strings (D-13, D-14, D-15)

```rust
// stripe_webhook_events (D-13)
description = "Scan project source for SyncDispatcher webhook handler registrations. \
    Returns event types and file locations for all `.on(|event: EventType| ...)` \
    calls found in `src/`.\n\n\
    **When to use:** Understanding which Stripe events the app handles, \
    checking handler coverage, debugging missing event handling.\n\n\
    **Returns:** events array with event_type, file path, and line number.\n\n\
    **Combine with:** `stripe_config_status` to verify setup, \
    `list_jobs` to see ProcessStripeWebhook job."

// stripe_config_status (D-14) — add capability-axis mention
description = "Report Stripe configuration status for the current project.\n\n\
    **When to use:** Verifying Stripe is configured before running the app, \
    checking which env vars are set, confirming the scaffold and capability-axis \
    module layout (checkout, refund, account, webhook) exist.\n\n\
    **Returns:** configured (bool), keys_present, keys_missing, scaffold_exists, \
    scaffold_files, checkout_exists, refund_exists, account_exists, webhook_dir_exists.\n\n\
    **Combine with:** `stripe_webhook_events` to check event handlers, \
    `stripe_subscription_info` to inspect the billing table schema, \
    `get_config` to view Stripe env var values."

// stripe_subscription_info (D-15) — clarify app-level billing table
description = "Report the tenant_billing table schema parsed from app migration files.\n\n\
    **When to use:** Checking the app billing table structure, understanding column types \
    and nullability, verifying the migration was generated. Scans app migrations for \
    `tenant_billing` — not the ferro-stripe framework module.\n\n\
    **Returns:** table_exists, migration_file path, columns (name, sql_type, nullable, default), indexes.\n\n\
    **Combine with:** `list_migrations` to see migration status, \
    `db_schema` for live table introspection after migration."
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `impl Listener<EventType> for StructName` in `src/stripe/listeners.rs` | `SyncDispatcher::on(\|event: EventType\| ...)` anywhere in `src/` | Phase 141 (2026-04-20) | Scanner must walk all `.rs` files and match closure/turbofish forms |
| `ferro_events::Event` impls | `StripeEvent` marker trait with `from_raw` | Phase 141 | No named listener structs exist; `listener` field in MCP response is meaningless |
| `src/stripe/{listeners,events,subscriptions}.rs` layout | `src/stripe/{checkout,refund,account,webhook/}` capability-axis layout | Phase 141 | Config status must check for new file names |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Line number formula `content[..offset].lines().count() + 1` returns correct 1-based line | Code Examples | Off-by-one in reported line numbers; minor, caught by D-16 test assertions |
| A2 | Compiling `Regex` once per function call (not lazy_static) is acceptable for MCP tool latency | Architecture Patterns | Negligible; regex compilation is fast; all other tools in this file follow same pattern |

---

## Open Questions

1. **Deduplication of duplicate (event_type, file, line) tuples**
   - What we know: CONTEXT.md specifics section says "deduplicated (event_type, file, line) tuples".
   - What's unclear: Whether the two regex patterns (closure + turbofish) could match the same line.
   - Recommendation: Deduplicate after collecting all matches using a `HashSet<(String, String, u32)>` before building the final `Vec`. Low-risk addition.

2. **`scaffold_files` behavior when `webhook/` subdir exists**
   - What we know: D-08 says flat listing, non-recursive. `fs::read_dir` only lists direct children of `src/stripe/`.
   - What's unclear: Whether the existing `fs::read_dir` filter already excludes directories (it filters on `.extension() == "rs"`, so directories are excluded naturally).
   - Recommendation: Confirmed safe — directories have no `.rs` extension; no behavior change needed.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code-only changes to `ferro-mcp/src/tools/stripe.rs` and `ferro-mcp/src/service.rs`. No external tools, databases, or services involved.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`, `#[cfg(test)]`) |
| Config file | none — standard Cargo test runner |
| Quick run command | `cargo test --manifest-path ferro-mcp/Cargo.toml -- stripe` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-16 | Closure `.on(\|event: Type\| ...)` detected with correct `event_type` and `line` | unit | `cargo test --manifest-path ferro-mcp/Cargo.toml -- test_webhook_events_parses_listeners` | ✅ (update in place) |
| D-17 | Turbofish `.on::<EventType` detected | unit | `cargo test --manifest-path ferro-mcp/Cargo.toml -- test_webhook_events_turbofish` | ❌ Wave 0 |
| D-18 | `checkout_exists`, `refund_exists`, `account_exists`, `webhook_dir_exists` correct | unit | `cargo test --manifest-path ferro-mcp/Cargo.toml -- test_config_status_capability_axis_fields` | ❌ Wave 0 |
| D-19 | `test_webhook_events_serializes` compiles without `listener` field | unit | `cargo test --manifest-path ferro-mcp/Cargo.toml -- test_webhook_events_serializes` | ✅ (update in place) |
| SC-6 | Workspace CI green, version bumped | build | `cargo build --all-features` | ✅ |

### Sampling Rate

- **Per task commit:** `cargo test --manifest-path ferro-mcp/Cargo.toml -- stripe`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `test_webhook_events_turbofish` — covers D-17 (turbofish pattern detection)
- [ ] `test_config_status_capability_axis_fields` — covers D-18 (four new boolean fields)

*(All other tests are updates to existing tests in `ferro-mcp/src/tools/stripe.rs`.)*

---

## Security Domain

This phase performs static source analysis (read-only file scanning) within the MCP server. No authentication, cryptography, session management, or external network access is involved.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | no | Input is project source files on disk; no user-controlled input |
| V6 Cryptography | no | — |

---

## Sources

### Primary (HIGH confidence)
- `ferro-mcp/src/tools/stripe.rs` — full current implementation and test suite; baseline for all changes
- `ferro-mcp/src/service.rs` lines 1542–1595 — current MCP tool registrations and description strings
- `ferro-mcp/src/introspection/events.rs` — WalkDir usage pattern (idiomatic for this codebase)
- `ferro-mcp/Cargo.toml` — confirms `walkdir = "2"` and `regex = "1"` are already direct deps; `tempfile = "3"` in dev-deps
- `ferro-stripe/src/webhook/sync.rs` — canonical SyncDispatcher API and `.on(|event: Type| ...)` call form
- `ferro-stripe/src/webhook/events.rs` — all 10 StripeEvent types; confirms no `ferro_events::Event` impls remain
- `.planning/phases/142-ferro-mcp-parity/142-CONTEXT.md` — all locked decisions (D-01 through D-20)

### Secondary (MEDIUM confidence)
- `.planning/phases/141-protocol-uplift/141-CONTEXT.md` — Phase 141 output decisions confirming old listener model is gone
- `.planning/phases/141-protocol-uplift/141-PATTERNS.md` — implementation patterns for SyncDispatcher closure form

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all deps verified in Cargo.toml
- Architecture: HIGH — both target files read in full; patterns confirmed from 15+ existing usages
- Pitfalls: HIGH — derived directly from struct diffs between old and new shape
- Test strategy: HIGH — existing 10-test suite confirmed passing; wave 0 gaps identified precisely

**Research date:** 2026-04-20
**Valid until:** Stable (ferro-mcp and ferro-stripe are in the same workspace; no external registry volatility)
