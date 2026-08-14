# Phase 249: ferro-mcp Introspection + Docs — Pattern Map

**Mapped:** 2026-08-15
**Files analyzed:** 6 (4 Rust, 2 Markdown new/modified)
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-mcp/src/tools/list_services.rs` | tool / static parser | transform (text → struct) | `ferro-mcp/src/tools/list_services.rs` (existing body) | self — extension of existing file |
| `ferro-mcp/src/service.rs` (tool description edit only) | tool registration | request-response | `ferro-mcp/src/service.rs` (other tool descriptions in same file) | exact — same file, same pattern |
| `ferro-mcp/src/tools/generation_context.rs` (optional `offload` field) | tool / data assembly | transform | `ferro-mcp/src/tools/generation_context.rs` `live_projection` field | role-match — same struct, `&'static str` field pattern |
| `docs/src/features/offload.md` (NEW) | documentation page | n/a | `docs/src/features/queues.md` / `docs/src/features/caching.md` | role-match — same mdBook feature-page structure |
| `docs/src/features/queues.md` (pointer reduction) | documentation page | n/a | existing file being trimmed — no new analog needed | self |
| `docs/src/SUMMARY.md` (nav entry) | mdBook nav | n/a | `docs/src/SUMMARY.md` lines 21–59 (existing `# Features` entries) | exact — same pattern in same file |

---

## Pattern Assignments

### `ferro-mcp/src/tools/list_services.rs` — static parser extension

**Analog:** the same file, existing body (lines 1–183).

**Imports pattern** (lines 7–10):
```rust
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
```
No new imports are needed for the offload extension. `walkdir` and `std::fs` are already brought into scope inside `scan_services_from_files` via `use` statements at the function body level (lines 106–107).

**Existing `ServiceItem` struct** (lines 31–37) — the extension point:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceItem {
    /// Service name (trait or concrete type)
    pub name: String,
    /// Type of binding (trait_binding or singleton)
    pub binding_type: String,
}
```
Add `methods: Vec<OffloadableMethod>` with `#[serde(default, skip_serializing_if = "Vec::is_empty")]` so non-offload service output is byte-for-byte unchanged (D-02). The `Deserialize` derive stays because `fetch_runtime_services` maps through `RuntimeServiceInfo` first (lines 73–82) and constructs `ServiceItem` manually — but keeping `Deserialize` is correct practice and costs nothing.

**New structs to add immediately above `ServiceItem`:**
```rust
#[derive(Debug, Serialize, Clone)]
pub struct OffloadParam {
    pub name: String,
    pub rust_type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct OffloadableMethod {
    pub name: String,
    /// Queue declared in #[offload(queue = "...")] or "default" when omitted.
    pub queue: String,
    /// Non-self parameters, types as Rust strings (owned equivalents of borrow types).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<OffloadParam>,
}
```
Then extend `ServiceItem`:
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceItem {
    pub name: String,
    pub binding_type: String,
    /// Offloadable methods declared on this service trait.
    /// Absent (not serialized) for services with no #[offload] methods.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<OffloadableMethod>,
}
```

**Core parser pattern** (lines 104–183) — the model to extend and follow:
```rust
fn scan_services_from_files(project_root: &Path) -> Vec<ServiceItem> {
    use std::fs;
    use walkdir::WalkDir;

    let mut services = Vec::new();
    let src_dir = project_root.join("src");

    if !src_dir.exists() {
        return services;
    }

    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            for line in content.lines() {
                let trimmed = line.trim();
                // Match #[service(SomeType)]
                if trimmed.starts_with("#[service(") {
                    if let Some(start) = trimmed.find('(') {
                        if let Some(end) = trimmed.find(')') {
                            let impl_name = &trimmed[start + 1..end];
                            services.push(ServiceItem {
                                name: impl_name.trim().to_string(),
                                binding_type: "trait_binding".to_string(),
                            });
                        }
                    }
                }
                // ...
            }
        }
    }
    services
}
```
The offload augmentation runs as a second pass over the same `src_dir` after `scan_services_from_files` returns its `Vec<ServiceItem>`. This avoids the need to track `{` / `}` nesting depth for trait blocks in the existing pass (see Research Pitfall 3). The second-pass function signature:
```rust
fn scan_offload_methods_from_files(project_root: &Path, services: &mut Vec<ServiceItem>);
```
It detects `#[service(ConcreteType)]` → trait name → `#[offload]` methods belonging to that trait using a three-state machine (Idle → OffloadPending → FnCollecting), then matches discovered methods back to `ServiceItem` by name.

**Dual-path wiring in `execute()`** (lines 85–102) — the extension point for offload augmentation:
```rust
pub async fn execute(project_root: &Path) -> Result<ServicesInfo> {
    // Try runtime endpoint first
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        if let Some(services) = fetch_runtime_services(base_url).await {
            return Ok(ServicesInfo {
                services,
                source: ServiceSource::Runtime,
            });
        }
    }

    // Fall back to static analysis
    let services = scan_services_from_files(project_root);
    Ok(ServicesInfo {
        services,
        source: ServiceSource::StaticAnalysis,
    })
}
```
Per the Research recommendation for D-05: run `scan_offload_methods_from_files` in **both** branches so agents always see offload data regardless of whether the app is running. The runtime `/_ferro/services` endpoint is not modified (D-05 satisfied). Pattern:
```rust
pub async fn execute(project_root: &Path) -> Result<ServicesInfo> {
    for base_url in ["http://localhost:8080", "http://127.0.0.1:8080"] {
        if let Some(mut services) = fetch_runtime_services(base_url).await {
            scan_offload_methods_from_files(project_root, &mut services);
            return Ok(ServicesInfo { services, source: ServiceSource::Runtime });
        }
    }
    let mut services = scan_services_from_files(project_root);
    scan_offload_methods_from_files(project_root, &mut services);
    Ok(ServicesInfo { services, source: ServiceSource::StaticAnalysis })
}
```

**Inline test pattern** — copy exactly from `ferro-mcp/src/tools/route_dependencies.rs` lines 218–313:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let source = r#"
            // ... rust source snippet ...
        "#;
        let result = helper_fn(source);
        assert!(result.contains(...));
    }
}
```
Key: plain `#[test]` (not `#[tokio::test]`) because the parser helpers are synchronous. No separate test file — tests live inline at the bottom of `list_services.rs`, matching `route_dependencies.rs`.

Tests to write (all using inline `r#"..."#` source snippets):
1. `detect_offload_attr` — bare `#[offload]` → queue `"default"`.
2. `detect_offload_attr` — `#[offload(queue = "reports")]` → queue `"reports"`.
3. `extract_method_params` — single simple param `tenant_id: i64` → `[{name:"tenant_id", rust_type:"i64"}]`.
4. `extract_method_params` — generic param `HashMap<K, V>` — bracket-aware split, not split on inner comma.
5. `extract_method_params` — `&str` param → `rust_type: "String"` (owned substitution).
6. Full integration: file with one `#[service]` trait, two `#[offload]` methods, one non-offload method — correct `ServiceItem.methods`.
7. Non-offload service: output unchanged (`methods` field absent in JSON).

---

### `ferro-mcp/src/service.rs` — tool description update

**Analog:** the same file, other `#[tool(...)]` description blocks (e.g. `list_services` at lines 600–616, `request_metrics` at lines 618–630).

**Current `list_services` tool description** (lines 601–608 — read verbatim):
```rust
#[tool(
    name = "list_services",
    description = "List all registered dependency injection container services.\n\n\
        **When to use:** Understanding available services, checking DI bindings, \
        planning new service dependencies, debugging resolution errors.\n\n\
        **Returns:** Singleton registrations, trait-to-concrete bindings, scopes.\n\n\
        **Combine with:** `get_handler` to see service usage, `application_info` for service overview."
)]
pub async fn list_services(&self) -> String {
```

**Target description (D-03)** — minimal diff replacing only `**When to use:**` and `**Returns:**` lines:
```rust
#[tool(
    name = "list_services",
    description = "List all registered dependency injection container services.\n\n\
        **When to use:** Understanding available services, checking DI bindings, \
        planning new service dependencies, debugging resolution errors, or \
        discovering which service methods are offloadable.\n\n\
        **Returns:** Singleton registrations, trait-to-concrete bindings. \
        Service entries with `#[offload]`-marked methods include a `methods` array \
        listing each method's name, declared queue, and typed parameter list \
        (`[{ name, rust_type }]`). Plain services omit the `methods` field.\n\n\
        **Combine with:** `get_handler` to see service usage, `application_info` for service overview."
)]
pub async fn list_services(&self) -> String {
```

---

### `ferro-mcp/src/tools/generation_context.rs` — optional `offload` field

**Analog:** `LiveProjectionGuidance` struct and its `memoize` field (lines 135–153), plus its construction in `execute()` (lines 455–477).

**The `&'static str` field pattern** (lines 145–148):
```rust
/// (c) `#[memoize]` — when to annotate, request-scoped dedup, coalescing, error caching,
/// graceful no-op outside request scope, complement to eager_loading/BatchLoad.
pub memoize: &'static str,
```
And its population in `execute()`:
```rust
memoize: "Annotate an async fn or #[service] method with #[memoize] (use ferro::memoize) \
    when N intents over one key call it during a render pass: it runs the body at most \
    once per (callsite, args) per request and coalesces concurrent callers onto one \
    shared computation (errors cached). It is request-scoped (dropped with the request), \
    a graceful no-op outside request scope, and COMPLEMENTS eager_loading/BatchLoad — it \
    is NOT cross-request caching (that stays ferro-cache).",
```

**Pattern to copy for `offload`:** Add a new `offload: &'static str` field to `GenerationContext` (not inside a sub-struct — keep it flat, same level as `live_projection`). Content template (one sentence on what `#[offload]` does, one on queue defaulting, one pointer):
```rust
/// Work distribution: offloadable service methods and the deployable worker model.
/// Read-only summary; see docs/src/features/offload.md for the full authoring surface.
pub offload: &'static str,
```
Populated in `execute()` as a `&'static str` literal — no new struct, no drift guard needed (the content is a stable descriptive sentence, not derived from a registry). Approximate content:
```
"Mark a #[service] trait method with #[offload] to derive a ferro-queue Job from its signature — \
the trait method keeps its in-process contract; #[offload] adds an .offload() enqueue entrypoint \
returning OffloadHandle<T>. Queue defaults to \"default\"; override with #[offload(queue = \"name\")]. \
Deploy workers as: <app-bin> worker --queue <name> (N replicas). See docs/src/features/offload.md."
```

**Struct addition location:** Add the `offload` field to `GenerationContext` immediately after `live_projection` (line 21 in the struct definition). Add its construction to `execute()` immediately after the `live_projection` struct literal (after line 477).

---

### `docs/src/features/offload.md` (NEW) — canonical documentation page

**Analog:** `docs/src/features/queues.md` (overall page structure) and `docs/src/features/caching.md` (opening paragraph style).

**Opening paragraph pattern** from `caching.md` (lines 1–3):
```markdown
# Caching

Ferro provides a unified caching API with support for multiple backends, cache tags for bulk
invalidation, and the convenient "remember" pattern for lazy caching.
```
Apply the same register: one-sentence factual description of what the feature provides, no marketing framing.

**Code block pattern** from `queues.md` lines 196–208 (which will be relocated here):
```markdown
```rust
use ferro::prelude::*;

#[service(impl = ReportBuilder)]
#[async_trait]
pub trait ReportsService: Send + Sync {
    #[offload]
    async fn build_monthly(&self, tenant_id: i64, month: Month) -> Report;
}
```
```

**Page structure to use** (from Research `offload.md` document structure, confirmed against `queues.md` heading depth):
```markdown
# Work Distribution (Offload)

<one-sentence intro>

## Authoring an offloadable method
  ### Authoring surface
  ### Typed handle
  ### Success-type contract
  ### Serializable contract

## Result path
  ### Enqueue and mark pending
  ### Server-side consumer
  ### Client-side read-back
  ### Delta payload and redaction
  ### Migration

## Scaling model
  ### Deploy recipe
  ### Worker classes and fault isolation
  ### Honest limitations

## Non-goals (v2.0 direction)
```

Content sources for each section:
- `## Authoring` and `## Result path` — relocate from `docs/src/features/queues.md` lines 188–end (currently confirmed as the "Offloading Service Methods" and "Subscribe and await" sections). Read those lines during implementation to relocate exact prose.
- `## Scaling model` — new prose from Research §Finding 9 facts (Phase 248 decided surface) + spec `2026-06-24-offload-work-distribution-design.md` §Scaling model. Four ASSUMED facts (A1–A4 in Research) must be grepped before writing code examples — all low-risk.
- `## Non-goals` — from spec §Future direction; framed as future work in neutral public voice (repository-docs discipline applies).

**`## Honest limitations` subsection content** (from Research §Finding 9, D-10):
- `DB_MAX_CONNECTIONS` × N replicas vs Postgres ceiling (~100); recommend PgBouncer.
- No built-in metrics or OTel export in generated manifests; monitoring requires a separately provisioned observability stack.
- Result latency is worker-scheduling-bound; unsuited for sub-second interactive computation.

---

### `docs/src/features/queues.md` — pointer reduction

**No new analog needed** — this is a content removal and replacement. After relocating the `## Offloading Service Methods` section (lines 188–end), replace those lines with a short cross-link paragraph:

```markdown
## Offloading Service Methods

For full documentation of `#[offload]`, the result-handle/streaming pattern, the deployable
worker scaling model, and the non-goals, see the dedicated page:
[Work Distribution (Offload)](offload.md).
```

Before making the edit: grep for inbound anchor links (`grep -r "queues.md#offload" docs/src/`) — Research tags this as ASSUMED A4 (no inbound anchors found, but verify during implementation).

---

### `docs/src/SUMMARY.md` — nav entry

**Analog:** existing `# Features` entries (lines 21–59 of SUMMARY.md).

**Current block** (lines 23–25):
```markdown
- [Queues & Background Jobs](features/queues.md)
- [Notifications](features/notifications.md)
```

**Target after insertion:**
```markdown
- [Queues & Background Jobs](features/queues.md)
- [Work Distribution (Offload)](features/offload.md)
- [Notifications](features/notifications.md)
```

The entry goes immediately after `queues.md` because `offload.md` is a direct companion page and the natural reading order is queue mechanics → offload distribution → notifications.

---

## Shared Patterns

### `#[serde(default, skip_serializing_if = ...)]` for additive optional Vec fields

**Source:** Research §Finding 4 (design reasoning confirmed against codebase conventions). The serde enum `ServiceSource` already uses `#[serde(rename_all = "snake_case")]` (lines 23–29). `ServiceItem` does not use `rename_all` (it is a plain struct), so no rename annotation is needed on the new fields.

**Pattern:**
```rust
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub methods: Vec<OffloadableMethod>,
```
`default` ensures deserialization of existing runtime payloads (which carry no `methods` key) continues to succeed. `skip_serializing_if = "Vec::is_empty"` ensures plain services serialize without the field.

### Bracket-aware comma split for generic param types

**Apply to:** `extract_method_params` helper in `list_services.rs`.

Pattern (from Research §Finding 3):
```
Maintain a depth counter (starts at 0).
Increment on '<' and '['.
Decrement on '>' and ']'.
Only split on ',' at depth 0.
```
This handles `HashMap<K, V>` (inner comma at depth 1 is not a param separator) and `Vec<String>` without false splits.

### `owned_type` text substitution

**Apply to:** type strings recovered by `extract_method_params`.

Rules (from Research §Finding 2, mirroring `ferro-macros/src/offload.rs owned_type`):
- `&str` → `String`
- `&[T]` → `Vec<T>` (T = content between `[` and `]`)
- `&T` → `T` (strip leading `&`)
- all other types → verbatim

### Neutral public voice for docs

**Apply to:** all prose in `docs/src/features/offload.md`.

Per the repository-documents convention (CLAUDE.md): neutral architectural documentation voice. Trigger phrases to avoid: "killer feature", "the bet", "load-bearing", "we accept that", "our risk". The Honest Limitations and Non-goals sections must describe constraints plainly without apologetic framing — state the fact and the mitigation.

---

## No Analog Found

All files in scope have clear analogs. No file requires falling back to external reference patterns.

---

## Implementation Notes for Planner

**Pitfall 3 — service-to-method correlation** (Research §Pitfall 3): The second-pass approach (`scan_offload_methods_from_files` as a separate function) is the correct architecture. The first pass produces `Vec<ServiceItem>` with `name` = the concrete impl type (e.g. `"ReportBuilder"` from `#[service(ReportBuilder)]`). The second pass detects `#[service(...)]` → trait name → opening brace, then accumulates `#[offload]` methods inside that trait block. The trait name (from `pub trait TraitName`) correlates methods to the owning service. Match back to `ServiceItem` by either the concrete name or trait name — the planner should confirm which name the first pass stores (`ServiceItem.name` = `impl_name` from `#[service(ConcreteType)]`).

**ASSUMED facts requiring one-line verification before implementation** (from Research §Assumptions Log A1–A4):
- A1: `ferro::offload::enqueue_and_mark_pending` re-export path — `grep -r "enqueue_and_mark_pending" framework/src/lib.rs`
- A2: `CreateProjectionSnapshotsTable` crate — `grep -r "CreateProjectionSnapshotsTable" ferro-projection/src/`
- A3: `--no-worker` flag in `app/src/main.rs` — `grep "no.worker\|no_worker" app/src/main.rs`
- A4: No inbound anchor links to `queues.md#offload` — `grep -r "queues.md#" docs/src/`

All four are low-risk single-grep checks; none block planning.

**Test run command:** `cargo test -p ferro-mcp` (per-task gate). Full gate before phase close: `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`.

---

## Metadata

**Analog search scope:** `ferro-mcp/src/tools/`, `ferro-mcp/src/service.rs`, `docs/src/features/`, `docs/src/SUMMARY.md`
**Files read:** 8 source files
**Pattern extraction date:** 2026-08-15
